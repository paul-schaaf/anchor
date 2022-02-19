//! Data structures that are used to provide non-argument inputs to program endpoints

use crate::{Accounts, ToAccountInfos, ToAccountMetas};
use solana_program::account_info::AccountInfo;
use solana_program::instruction::AccountMeta;
use solana_program::pubkey::Pubkey;
use std::collections::BTreeMap;
use std::fmt;

/// Provides non-argument inputs to the program.
///
/// # Example
/// ```ignore
/// pub fn set_data(ctx: Context<SetData>, age: u64, other_data: u32) -> anchor_lang::Result<()> {
///     // Set account data like this
///     (*ctx.accounts.my_account).age = age;
///     (*ctx.accounts.my_account).other_data = other_data;
///     // or like this
///     let my_account = &mut ctx.account.my_account;
///     my_account.age = age;
///     my_account.other_data = other_data;
///     Ok(())
/// }
/// ```
pub struct Context<'a, 'b, 'c, 'info, T> {
    /// Currently executing program id.
    pub program_id: &'a Pubkey,
    /// Deserialized accounts.
    pub accounts: &'b mut T,
    /// Remaining accounts given but not deserialized or validated.
    /// Be very careful when using this directly.
    pub remaining_accounts: &'c [AccountInfo<'info>],
    /// Bump seeds found during constraint validation. This is provided as a
    /// convenience so that handlers don't have to recalculate bump seeds or
    /// pass them in as arguments.
    pub bumps: BTreeMap<String, u8>,
}

impl<'a, 'b, 'c, 'info, T: fmt::Debug> fmt::Debug for Context<'a, 'b, 'c, 'info, T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Context")
            .field("program_id", &self.program_id)
            .field("accounts", &self.accounts)
            .field("remaining_accounts", &self.remaining_accounts)
            .field("bumps", &self.bumps)
            .finish()
    }
}

impl<'a, 'b, 'c, 'info, T: Accounts<'info>> Context<'a, 'b, 'c, 'info, T> {
    pub fn new(
        program_id: &'a Pubkey,
        accounts: &'b mut T,
        remaining_accounts: &'c [AccountInfo<'info>],
        bumps: BTreeMap<String, u8>,
    ) -> Self {
        Self {
            program_id,
            accounts,
            remaining_accounts,
            bumps,
        }
    }
}

/// Context specifying non-argument inputs for cross-program-invocations.
///
/// # Example with and without PDA signature
/// ```ignore
/// // Callee Program
///
/// use anchor_lang::prelude::*;
///
/// declare_id!("Fg6PaFpoGXkYsidMpWTK6W2BeZ7FEfcYkg476zPFsLnS");
///
/// #[program]
/// pub mod callee {
///     use super::*;
///     pub fn init(ctx: Context<Init>) -> anchor_lang::Result<()> {
///         (*ctx.accounts.data).authority = ctx.accounts.authority.key();
///         Ok(())
///     }
///
///     pub fn set_data(ctx: Context<SetData>, data: u64) -> anchor_lang::Result<()> {
///         (*ctx.accounts.data_acc).data = data;
///         Ok(())
///     }
/// }
///
/// #[account]
/// #[derive(Default)]
/// pub struct Data {
///     data: u64,
///     authority: Pubkey,
/// }
///
/// #[derive(Accounts)]
/// pub struct Init<'info> {
///     #[account(init, payer = payer)]
///     pub data: Account<'info, Data>,
///     pub payer: Signer<'info>,
///     pub authority: UncheckedAccount<'info>,
///     pub system_program: Program<'info, System>
/// }
///
/// #[derive(Accounts)]
/// pub struct SetData<'info> {
///     #[account(mut, has_one = authority)]
///     pub data_acc: Account<'info, Data>,
///     pub authority: Signer<'info>,
/// }
///
/// // Caller Program
///
/// use anchor_lang::prelude::*;
/// use callee::{self, program::Callee};
///
/// declare_id!("Sxg7dBh5VLT8S1o6BqncZCPq9nhHHukjfVd6ohQJeAk");
///
/// #[program]
/// pub mod caller {
///     use super::*;
///     pub fn do_cpi(ctx: Context<DoCpi>, data: u64) -> anchor_lang::Result<()> {
///         let callee_id = ctx.accounts.callee.to_account_info();
///         let callee_accounts = callee::cpi::accounts::SetData {
///             data_acc: ctx.accounts.data_acc.to_account_info(),
///             authority: ctx.accounts.callee_authority.to_account_info(),
///         };
///         let cpi_ctx = CpiContext::new(callee_id, callee_accounts);
///         callee::cpi::set_data(cpi_ctx, data)
///     }
///
///     pub fn do_cpi_with_pda_authority(ctx: Context<DoCpiWithPDAAuthority>, bump: u8, data: u64) -> anchor_lang::Result<()> {
///         let seeds = &[&[b"example_seed", bytemuck::bytes_of(&bump)][..]];
///         let callee_id = ctx.accounts.callee.to_account_info();
///         let callee_accounts = callee::cpi::accounts::SetData {
///             data_acc: ctx.accounts.data_acc.to_account_info(),
///             authority: ctx.accounts.callee_authority.to_account_info(),
///         };
///         let cpi_ctx = CpiContext::new_with_signer(callee_id, callee_accounts, seeds);
///         callee::cpi::set_data(cpi_ctx, data)
///     }
/// }
///
/// // We can use "UncheckedAccount"s here because
/// // the callee program does the checks.
/// // We use "mut" so the autogenerated clients know
/// // that this account should be mutable.
/// #[derive(Accounts)]
/// pub struct DoCpi<'info> {
///     #[account(mut)]
///     pub data_acc: UncheckedAccount<'info>,
///     pub callee_authority: UncheckedAccount<'info>,
///     pub callee: Program<'info, Callee>,
/// }
///
/// #[derive(Accounts)]
/// pub struct DoCpiWithPDAAuthority<'info> {
///     #[account(mut)]
///     pub data_acc: UncheckedAccount<'info>,
///     pub callee_authority: UncheckedAccount<'info>,
///     pub callee: Program<'info, Callee>,
/// }
/// ```
pub struct CpiContext<'a, 'b, 'c, 'info, T>
where
    T: ToAccountMetas + ToAccountInfos<'info>,
{
    pub accounts: T,
    pub remaining_accounts: Vec<AccountInfo<'info>>,
    pub program: AccountInfo<'info>,
    pub signer_seeds: &'a [&'b [&'c [u8]]],
}

impl<'a, 'b, 'c, 'info, T> CpiContext<'a, 'b, 'c, 'info, T>
where
    T: ToAccountMetas + ToAccountInfos<'info>,
{
    pub fn new(program: AccountInfo<'info>, accounts: T) -> Self {
        Self {
            accounts,
            program,
            remaining_accounts: Vec::new(),
            signer_seeds: &[],
        }
    }

    #[must_use]
    pub fn new_with_signer(
        program: AccountInfo<'info>,
        accounts: T,
        signer_seeds: &'a [&'b [&'c [u8]]],
    ) -> Self {
        Self {
            accounts,
            program,
            signer_seeds,
            remaining_accounts: Vec::new(),
        }
    }

    #[must_use]
    pub fn with_signer(mut self, signer_seeds: &'a [&'b [&'c [u8]]]) -> Self {
        self.signer_seeds = signer_seeds;
        self
    }

    #[must_use]
    pub fn with_remaining_accounts(mut self, ra: Vec<AccountInfo<'info>>) -> Self {
        self.remaining_accounts = ra;
        self
    }
}

impl<'info, T: ToAccountInfos<'info> + ToAccountMetas> ToAccountInfos<'info>
    for CpiContext<'_, '_, '_, 'info, T>
{
    fn to_account_infos(&self) -> Vec<AccountInfo<'info>> {
        let mut infos = self.accounts.to_account_infos();
        infos.extend_from_slice(&self.remaining_accounts);
        infos.push(self.program.clone());
        infos
    }
}

impl<'info, T: ToAccountInfos<'info> + ToAccountMetas> ToAccountMetas
    for CpiContext<'_, '_, '_, 'info, T>
{
    fn to_account_metas(&self, is_signer: Option<bool>) -> Vec<AccountMeta> {
        let mut metas = self.accounts.to_account_metas(is_signer);
        metas.append(
            &mut self
                .remaining_accounts
                .iter()
                .map(|acc| match acc.is_writable {
                    false => AccountMeta::new_readonly(*acc.key, acc.is_signer),
                    true => AccountMeta::new(*acc.key, acc.is_signer),
                })
                .collect(),
        );
        metas
    }
}

/// Context specifying non-argument inputs for cross-program-invocations
/// targeted at program state instructions.
#[doc(hidden)]
#[deprecated]
pub struct CpiStateContext<'a, 'b, 'c, 'info, T: Accounts<'info>> {
    state: AccountInfo<'info>,
    cpi_ctx: CpiContext<'a, 'b, 'c, 'info, T>,
}

#[allow(deprecated)]
impl<'a, 'b, 'c, 'info, T: Accounts<'info>> CpiStateContext<'a, 'b, 'c, 'info, T> {
    pub fn new(program: AccountInfo<'info>, state: AccountInfo<'info>, accounts: T) -> Self {
        Self {
            state,
            cpi_ctx: CpiContext {
                accounts,
                program,
                signer_seeds: &[],
                remaining_accounts: Vec::new(),
            },
        }
    }

    pub fn new_with_signer(
        program: AccountInfo<'info>,
        state: AccountInfo<'info>,
        accounts: T,
        signer_seeds: &'a [&'b [&'c [u8]]],
    ) -> Self {
        Self {
            state,
            cpi_ctx: CpiContext {
                accounts,
                program,
                signer_seeds,
                remaining_accounts: Vec::new(),
            },
        }
    }

    #[must_use]
    pub fn with_signer(mut self, signer_seeds: &'a [&'b [&'c [u8]]]) -> Self {
        self.cpi_ctx = self.cpi_ctx.with_signer(signer_seeds);
        self
    }

    pub fn program(&self) -> &AccountInfo<'info> {
        &self.cpi_ctx.program
    }

    pub fn signer_seeds(&self) -> &[&[&[u8]]] {
        self.cpi_ctx.signer_seeds
    }
}

#[allow(deprecated)]
impl<'a, 'b, 'c, 'info, T: Accounts<'info>> ToAccountMetas
    for CpiStateContext<'a, 'b, 'c, 'info, T>
{
    fn to_account_metas(&self, is_signer: Option<bool>) -> Vec<AccountMeta> {
        // State account is always first for state instructions.
        let mut metas = vec![match self.state.is_writable {
            false => AccountMeta::new_readonly(*self.state.key, false),
            true => AccountMeta::new(*self.state.key, false),
        }];
        metas.append(&mut self.cpi_ctx.accounts.to_account_metas(is_signer));
        metas
    }
}

#[allow(deprecated)]
impl<'a, 'b, 'c, 'info, T: Accounts<'info>> ToAccountInfos<'info>
    for CpiStateContext<'a, 'b, 'c, 'info, T>
{
    fn to_account_infos(&self) -> Vec<AccountInfo<'info>> {
        let mut infos = self.cpi_ctx.accounts.to_account_infos();
        infos.push(self.state.clone());
        infos.push(self.cpi_ctx.program.clone());
        infos
    }
}
