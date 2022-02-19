//! Misc example is a catchall program for testing unrelated features.
//! It's not too instructive/coherent by itself, so please see other examples.

use anchor_lang::prelude::*;
use context::*;
use event::*;
use misc2::Auth;

mod account;
mod context;
mod event;

declare_id!("Fg6PaFpoGXkYsidMpWTK6W2BeZ7FEfcYkg476zPFsLnS");

#[constant]
pub const BASE: u128 = 1_000_000;
#[constant]
pub const DECIMALS: u8 = 6;
pub const NO_IDL: u16 = 55;

#[program]
pub mod misc {
    use super::*;

    pub const SIZE: u64 = 99;

    #[state(SIZE)]
    pub struct MyState {
        pub v: Vec<u8>,
    }

    impl MyState {
        pub fn new(_ctx: Context<Ctor>) -> Result<Self, ProgramError> {
            Ok(Self { v: vec![] })
        }

        pub fn remaining_accounts(&mut self, ctx: Context<RemainingAccounts>) -> anchor_lang::Result<()> {
            if ctx.remaining_accounts.len() != 1 {
                return Err(ProgramError::Custom(1).into()); // Arbitrary error.
            }
            Ok(())
        }
    }

    pub fn initialize(ctx: Context<Initialize>, udata: u128, idata: i128) -> anchor_lang::Result<()> {
        ctx.accounts.data.udata = udata;
        ctx.accounts.data.idata = idata;
        Ok(())
    }

    pub fn initialize_no_rent_exempt(ctx: Context<InitializeNoRentExempt>) -> anchor_lang::Result<()> {
        Ok(())
    }

    pub fn initialize_skip_rent_exempt(ctx: Context<InitializeSkipRentExempt>) -> anchor_lang::Result<()> {
        Ok(())
    }

    pub fn test_owner(_ctx: Context<TestOwner>) -> anchor_lang::Result<()> {
        Ok(())
    }

    pub fn test_executable(_ctx: Context<TestExecutable>) -> anchor_lang::Result<()> {
        Ok(())
    }

    pub fn test_state_cpi(ctx: Context<TestStateCpi>, data: u64) -> anchor_lang::Result<()> {
        let cpi_program = ctx.accounts.misc2_program.clone();
        let cpi_accounts = Auth {
            authority: ctx.accounts.authority.clone(),
        };
        let ctx = ctx.accounts.cpi_state.context(cpi_program, cpi_accounts);
        misc2::cpi::state::set_data(ctx, data)
    }

    pub fn test_u16(ctx: Context<TestU16>, data: u16) -> anchor_lang::Result<()> {
        ctx.accounts.my_account.data = data;
        Ok(())
    }

    pub fn test_simulate(_ctx: Context<TestSimulate>, data: u32) -> anchor_lang::Result<()> {
        emit!(E1 { data });
        emit!(E2 { data: 1234 });
        emit!(E3 { data: 9 });
        Ok(())
    }

    pub fn test_i8(ctx: Context<TestI8>, data: i8) -> anchor_lang::Result<()> {
        ctx.accounts.data.data = data;
        Ok(())
    }

    pub fn test_i16(ctx: Context<TestI16>, data: i16) -> anchor_lang::Result<()> {
        ctx.accounts.data.data = data;
        Ok(())
    }

    pub fn test_const_array_size(ctx: Context<TestConstArraySize>, data: u8) -> anchor_lang::Result<()> {
        ctx.accounts.data.data[0] = data;
        Ok(())
    }

    pub fn test_close(_ctx: Context<TestClose>) -> anchor_lang::Result<()> {
        Ok(())
    }

    pub fn test_instruction_constraint(
        _ctx: Context<TestInstructionConstraint>,
        _nonce: u8,
    ) -> anchor_lang::Result<()> {
        Ok(())
    }

    pub fn test_pda_init(
        ctx: Context<TestPdaInit>,
        _domain: String,
        _seed: Vec<u8>,
        _bump: u8,
    ) -> anchor_lang::Result<()> {
        ctx.accounts.my_pda.data = 6;
        Ok(())
    }

    pub fn test_pda_init_zero_copy(ctx: Context<TestPdaInitZeroCopy>) -> anchor_lang::Result<()> {
        let mut acc = ctx.accounts.my_pda.load_init()?;
        acc.data = 9;
        acc.bump = *ctx.bumps.get("my_pda").unwrap();
        Ok(())
    }

    pub fn test_pda_mut_zero_copy(ctx: Context<TestPdaMutZeroCopy>) -> anchor_lang::Result<()> {
        let mut acc = ctx.accounts.my_pda.load_mut()?;
        acc.data = 1234;
        Ok(())
    }

    pub fn test_token_seeds_init(_ctx: Context<TestTokenSeedsInit>) -> anchor_lang::Result<()> {
        Ok(())
    }

    pub fn default<'info>(
        _program_id: &Pubkey,
        _accounts: &[AccountInfo<'info>],
        _data: &[u8],
    ) -> anchor_lang::Result<()> {
        Err(ProgramError::Custom(1234).into())
    }

    pub fn test_init(ctx: Context<TestInit>) -> anchor_lang::Result<()> {
        ctx.accounts.data.data = 3;
        Ok(())
    }

    pub fn test_init_zero_copy(ctx: Context<TestInitZeroCopy>) -> anchor_lang::Result<()> {
        let mut data = ctx.accounts.data.load_init()?;
        data.data = 10;
        data.bump = 2;
        Ok(())
    }

    pub fn test_init_mint(ctx: Context<TestInitMint>) -> anchor_lang::Result<()> {
        assert!(ctx.accounts.mint.decimals == 6);
        Ok(())
    }

    pub fn test_init_token(ctx: Context<TestInitToken>) -> anchor_lang::Result<()> {
        assert!(ctx.accounts.token.mint == ctx.accounts.mint.key());
        Ok(())
    }

    pub fn test_composite_payer(ctx: Context<TestCompositePayer>) -> anchor_lang::Result<()> {
        ctx.accounts.composite.data.data = 1;
        ctx.accounts.data.udata = 2;
        ctx.accounts.data.idata = 3;
        Ok(())
    }

    pub fn test_init_associated_token(ctx: Context<TestInitAssociatedToken>) -> anchor_lang::Result<()> {
        assert!(ctx.accounts.token.mint == ctx.accounts.mint.key());
        Ok(())
    }

    pub fn test_validate_associated_token(
        _ctx: Context<TestValidateAssociatedToken>,
    ) -> anchor_lang::Result<()> {
        Ok(())
    }

    pub fn test_fetch_all(ctx: Context<TestFetchAll>, filterable: Pubkey) -> anchor_lang::Result<()> {
        ctx.accounts.data.authority = ctx.accounts.authority.key();
        ctx.accounts.data.filterable = filterable;
        Ok(())
    }

    pub fn test_init_with_empty_seeds(ctx: Context<TestInitWithEmptySeeds>) -> anchor_lang::Result<()> {
        Ok(())
    }

    pub fn test_empty_seeds_constraint(ctx: Context<TestEmptySeedsConstraint>) -> anchor_lang::Result<()> {
        Ok(())
    }

    pub fn test_init_if_needed(ctx: Context<TestInitIfNeeded>, data: u16) -> anchor_lang::Result<()> {
        ctx.accounts.data.data = data;
        Ok(())
    }

    pub fn test_init_if_needed_checks_owner(
        ctx: Context<TestInitIfNeededChecksOwner>,
    ) -> anchor_lang::Result<()> {
        Ok(())
    }

    pub fn test_init_if_needed_checks_seeds(
        ctx: Context<TestInitIfNeededChecksSeeds>,
        seed_data: String,
    ) -> anchor_lang::Result<()> {
        Ok(())
    }

    pub fn test_init_mint_if_needed(
        ctx: Context<TestInitMintIfNeeded>,
        decimals: u8,
    ) -> anchor_lang::Result<()> {
        Ok(())
    }

    pub fn test_init_token_if_needed(ctx: Context<TestInitTokenIfNeeded>) -> anchor_lang::Result<()> {
        Ok(())
    }

    pub fn test_init_associated_token_if_needed(
        ctx: Context<TestInitAssociatedTokenIfNeeded>,
    ) -> anchor_lang::Result<()> {
        Ok(())
    }

    pub fn init_with_space(ctx: Context<InitWithSpace>, data: u16) -> anchor_lang::Result<()> {
        Ok(())
    }

    pub fn test_multidimensional_array(
        ctx: Context<TestMultidimensionalArray>,
        data: [[u8; 10]; 10],
    ) -> anchor_lang::Result<()> {
        ctx.accounts.data.data = data;
        Ok(())
    }

    pub fn test_no_rent_exempt(ctx: Context<NoRentExempt>) -> anchor_lang::Result<()> {
        Ok(())
    }

    pub fn test_enforce_rent_exempt(ctx: Context<EnforceRentExempt>) -> anchor_lang::Result<()> {
        Ok(())
    }

    pub fn init_decrease_lamports(ctx: Context<InitDecreaseLamports>) -> anchor_lang::Result<()> {
        **ctx.accounts.data.try_borrow_mut_lamports()? -= 1;
        **ctx.accounts.user.try_borrow_mut_lamports()? += 1;
        Ok(())
    }

    pub fn init_if_needed_checks_rent_exemption(
        _ctx: Context<InitIfNeededChecksRentExemption>,
    ) -> anchor_lang::Result<()> {
        Ok(())
    }

    pub fn test_program_id_constraint(
        _ctx: Context<TestProgramIdConstraint>,
        _bump: u8,
        _second_bump: u8,
    ) -> anchor_lang::Result<()> {
        Ok(())
    }

    pub fn test_program_id_constraint_find_pda(
        _ctx: Context<TestProgramIdConstraintUsingFindPda>,
    ) -> anchor_lang::Result<()> {
        Ok(())
    }
}
