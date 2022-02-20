extern crate proc_macro;

use anchor_syn::parser;
use heck::SnakeCase;
use quote::quote;
use syn::parse_macro_input;

/// The `#[interface]` attribute allows one to define an external program
/// dependency, without having any knowledge about the program, other than
/// the fact that it implements the given trait.
///
/// Additionally, the attribute generates a client that can be used to perform
/// CPI to these external dependencies.
///
/// # Example
///
/// In the following example, we have a counter program, where the count
/// can only be set if the configured external program authorizes it.
///
/// ## Defining an `#[interface]`
///
/// First we define the program that depends on an external interface.
///
/// ```ignore
/// use anchor_lang::prelude::*;
///
/// #[interface]
/// pub trait Auth<'info, T: Accounts<'info>> {
///     fn is_authorized(ctx: Context<T>, current: u64, new: u64) -> anchor_lang::Result<()>;
/// }
///
/// #[program]
/// pub mod counter {
///     use super::*;
///
///     #[state]
///     pub struct Counter {
///         pub count: u64,
///         pub auth_program: Pubkey,
///     }
///
///     impl Counter {
///         pub fn new(_ctx: Context<Empty>, auth_program: Pubkey) -> Result<Self> {
///             Ok(Self {
///                 count: 0,
///                 auth_program,
///             })
///         }
///
///         #[access_control(SetCount::accounts(&self, &ctx))]
///         pub fn set_count(&mut self, ctx: Context<SetCount>, new_count: u64) -> Result<()> {
///             // Ask the auth program if we should approve the transaction.
///             let cpi_program = ctx.accounts.auth_program.clone();
///             let cpi_ctx = CpiContext::new(cpi_program, Empty {});
///
///             // This is the client generated by the `#[interface]` attribute.
///             auth::is_authorized(cpi_ctx, self.count, new_count)?;
///
///             // Approved, so update.
///             self.count = new_count;
///             Ok(())
///         }
///     }
/// }
///
/// #[derive(Accounts)]
/// pub struct Empty {}
///
/// #[derive(Accounts)]
/// pub struct SetCount<'info> {
///     auth_program: AccountInfo<'info>,
/// }
///
/// impl<'info> SetCount<'info> {
///     pub fn accounts(counter: &Counter, ctx: &Context<SetCount>) -> Result<()> {
///         if ctx.accounts.auth_program.key != &counter.auth_program {
///             return Err(ErrorCode::InvalidAuthProgram.into());
///         }
///         Ok(())
///     }
/// }
///
/// #[error]
/// pub enum ErrorCode {
///     #[msg("Invalid auth program.")]
///     InvalidAuthProgram,
/// }
///```
///
/// ## Defining an implementation
///
/// Now we define the program that implements the interface, which the above
/// program will call.
///
/// ```ignore
/// use anchor_lang::prelude::*;
/// use counter::Auth;
///
/// #[program]
/// pub mod counter_auth {
///     use super::*;
///
///     #[state]
///     pub struct CounterAuth;
///
///     impl<'info> Auth<'info, Empty> for CounterAuth {
///         fn is_authorized(_ctx: Context<Empty>, current: u64, new: u64) -> Result<()> {
///             if current % 2 == 0 {
///                 if new % 2 == 0 {
///                     return Err(ProgramError::Custom(50).into()); // Arbitrary error code.
///                 }
///             } else {
///                 if new % 2 == 1 {
///                     return Err(ProgramError::Custom(60).into()); // Arbitrary error code.
///                 }
///             }
///             Ok(())
///         }
///     }
/// }
/// #[derive(Accounts)]
/// pub struct Empty {}
/// ```
///
/// # Returning Values Across CPI
///
/// The caller above uses a `Result` to act as a boolean. However, in order
/// for this feature to be maximally useful, we need a way to return values from
/// interfaces. For now, one can do this by writing to a shared account, e.g.,
/// with the SPL's [Shared Memory Program](https://github.com/solana-labs/solana-program-library/tree/master/shared-memory).
/// In the future, Anchor will add the ability to return values across CPI
/// without having to worry about the details of shared memory accounts.
#[proc_macro_attribute]
pub fn interface(
    _args: proc_macro::TokenStream,
    input: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let item_trait = parse_macro_input!(input as syn::ItemTrait);

    let trait_name = item_trait.ident.to_string();
    let mod_name: proc_macro2::TokenStream = item_trait
        .ident
        .to_string()
        .to_snake_case()
        .parse()
        .unwrap();

    let methods: Vec<proc_macro2::TokenStream> = item_trait
        .items
        .iter()
        .filter_map(|trait_item: &syn::TraitItem| match trait_item {
            syn::TraitItem::Method(m) => Some(m),
            _ => None,
        })
        .map(|method: &syn::TraitItemMethod| {
            let method_name = &method.sig.ident;
            let args: Vec<&syn::PatType> = method
                .sig
                .inputs
                .iter()
                .filter_map(|arg: &syn::FnArg| match arg {
                    syn::FnArg::Typed(pat_ty) => Some(pat_ty),
                    // TODO: just map this to None once we allow this feature.
                    _ => panic!("Invalid syntax. No self allowed."),
                })
                .filter(|pat_ty| {
                    let mut ty = parser::tts_to_string(&pat_ty.ty);
                    ty.retain(|s| !s.is_whitespace());
                    !ty.starts_with("Context<")
                })
                .collect();
            let args_no_tys: Vec<&Box<syn::Pat>> = args
                .iter()
                .map(|arg| {
                    &arg.pat
                })
                .collect();
            let args_struct = {
                if args.is_empty() {
                    quote! {
                        use anchor_lang::prelude::borsh;
                        #[derive(anchor_lang::AnchorSerialize, anchor_lang::AnchorDeserialize)]
                        struct Args;
                    }
                } else {
                    quote! {
                        use anchor_lang::prelude::borsh;
                        #[derive(anchor_lang::AnchorSerialize, anchor_lang::AnchorDeserialize)]
                        struct Args {
                            #(#args),*
                        }
                    }
                }
            };

            let sighash_arr = anchor_syn::codegen::program::common::sighash(&trait_name, &method_name.to_string());
            let sighash_tts: proc_macro2::TokenStream =
                format!("{:?}", sighash_arr).parse().unwrap();
            quote! {
                pub fn #method_name<'a,'b, 'c, 'info, T: anchor_lang::Accounts<'info> + anchor_lang::ToAccountMetas + anchor_lang::ToAccountInfos<'info>>(
                    ctx: anchor_lang::context::CpiContext<'a, 'b, 'c, 'info, T>,
                    #(#args),*
                ) -> anchor_lang::Result<()> {
                    #args_struct

                    let ix = {
                        let ix = Args {
                            #(#args_no_tys),*
                        };
                        let mut ix_data = anchor_lang::AnchorSerialize::try_to_vec(&ix)
                            .map_err(|_| anchor_lang::anchor_attribute_error::error_without_origin!(anchor_lang::error::ErrorCode::InstructionDidNotSerialize))?;
                        let mut data = #sighash_tts.to_vec();
                        data.append(&mut ix_data);
                        let accounts = ctx.to_account_metas(None);
                        anchor_lang::solana_program::instruction::Instruction {
                            program_id: *ctx.program.key,
                            accounts,
                            data,
                        }
                    };
                    let mut acc_infos = ctx.to_account_infos();
                    acc_infos.push(ctx.program.clone());
                    anchor_lang::solana_program::program::invoke_signed(
                        &ix,
                        &acc_infos,
                        ctx.signer_seeds,
                    ).map_err(|pe| pe.into())
                }
            }
        })
        .collect();

    proc_macro::TokenStream::from(quote! {
        #item_trait

        /// Anchor generated module for invoking programs implementing an
        /// `#[interface]` via CPI.
        mod #mod_name {
            use super::*;
            #(#methods)*
        }
    })
}
