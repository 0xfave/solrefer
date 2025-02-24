use crate::{error::ReferralError, state::referral_program::*};
use anchor_lang::{
    prelude::*,
    system_program::{self, System, Transfer},
};
use anchor_spl::token::{self, Mint, Token, TokenAccount};

/// The seed used for deriving the vault PDA that holds SOL deposits
pub const VAULT_SEED: &[u8] = b"vault";

/// The seed used for deriving the token vault PDA that holds token deposits
pub const TOKEN_VAULT_SEED: &[u8] = b"token_vault";

/// Accounts required for depositing SOL into the referral program.
#[derive(Accounts)]
pub struct DepositSol<'info> {
    #[account(
        mut,
        constraint = referral_program.is_active @ ReferralError::ProgramInactive,
        has_one = authority @ ReferralError::InvalidAuthority,
    )]
    pub referral_program: Account<'info, ReferralProgram>,

    /// The vault that will hold the deposited SOL
    /// PDA with seeds: ["vault", referral_program.key()]
    #[account(
        mut,
        seeds = [VAULT_SEED, referral_program.key().as_ref()],
        bump,
    )]
    pub vault: SystemAccount<'info>,

    /// The authority/owner of the referral program
    #[account(mut)]
    pub authority: Signer<'info>,

    pub system_program: Program<'info, System>,
}

/// Deposits SOL into the referral program.
///
/// # Arguments
/// * `ctx` - The deposit context
/// * `amount` - The amount to deposit in lamports
///
/// # Errors
/// * `ProgramInactive` - If the referral program is not active
/// * `InvalidAuthority` - If the signer is not the program authority
/// * `InsufficientDeposit` - If the deposit amount is zero
pub fn deposit_sol(ctx: Context<DepositSol>, amount: u64) -> Result<()> {
    require!(amount > 0, ReferralError::InsufficientDeposit);

    let referral_program = &mut ctx.accounts.referral_program;

    // Validate that the program is not a token program
    if referral_program.token_mint != Pubkey::default() {
        return err!(ReferralError::SolDepositToTokenProgram);
    }

    // SOL deposit
    system_program::transfer(
        CpiContext::new(
            ctx.accounts.system_program.to_account_info(),
            Transfer {
                from: ctx.accounts.authority.to_account_info(),
                to: ctx.accounts.vault.to_account_info(),
            },
        ),
        amount,
    )?;

    referral_program.reload()?;

    // Update total available rewards
    referral_program.total_available =
        referral_program.total_available.checked_add(amount).ok_or(ReferralError::NumericOverflow)?;

    msg!("Deposited {} lamports to referral program", amount);
    Ok(())
}

/// Accounts required for depositing tokens into the referral program.
#[derive(Accounts)]
pub struct DepositToken<'info> {
    #[account(
        mut,
        constraint = referral_program.is_active @ ReferralError::ProgramInactive,
        has_one = authority @ ReferralError::InvalidAuthority,
    )]
    pub referral_program: Account<'info, ReferralProgram>,

    /// Token account vault that holds deposited tokens
    /// PDA with seeds: ["token_vault", referral_program.key()]
    #[account(
        mut,
        seeds = [TOKEN_VAULT_SEED, referral_program.key().as_ref()],
        bump,
        token::mint = token_mint,
        token::authority = referral_program,
    )]
    pub token_vault: Account<'info, TokenAccount>,

    /// The mint of the token for deposits
    #[account(
        constraint = token_mint.key() == referral_program.token_mint @ ReferralError::InvalidTokenMint
    )]
    pub token_mint: Account<'info, Mint>,

    /// The depositor's token account
    #[account(
        mut,
        constraint = depositor_token_account.mint == token_mint.key() &&
                     depositor_token_account.owner == authority.key() @ ReferralError::InvalidTokenAccounts
    )]
    pub depositor_token_account: Account<'info, TokenAccount>,

    /// The authority/owner of the referral program
    #[account(mut)]
    pub authority: Signer<'info>,

    pub token_program: Program<'info, Token>,
}

/// Deposits tokens into the referral program.
///
/// # Arguments
/// * `ctx` - The deposit context
/// * `amount` - The amount to deposit in token units
///
/// # Errors
/// * `ProgramInactive` - If the referral program is not active
/// * `InvalidAuthority` - If the signer is not the program authority
/// * `InvalidTokenProgram` - If the token program is incorrect
/// * `InvalidTokenMint` - If the token mint doesn't match the program's configuration
/// * `InvalidTokenAccounts` - If the token accounts are invalid
/// * `InsufficientDeposit` - If the deposit amount is zero
pub fn deposit_token(ctx: Context<DepositToken>, amount: u64) -> Result<()> {
    require!(amount > 0, ReferralError::InsufficientDeposit);

    let referral_program = &mut ctx.accounts.referral_program;

    // Validate that the program is a token program
    if referral_program.token_mint == Pubkey::default() {
        return err!(ReferralError::TokenDepositToSolProgram);
    }

    // Token deposit
    token::transfer(
        CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            token::Transfer {
                from: ctx.accounts.depositor_token_account.to_account_info(),
                to: ctx.accounts.token_vault.to_account_info(),
                authority: ctx.accounts.authority.to_account_info(),
            },
        ),
        amount,
    )?;

    referral_program.reload()?;

    // Update total available rewards
    referral_program.total_available =
        referral_program.total_available.checked_add(amount).ok_or(ReferralError::NumericOverflow)?;

    msg!("Deposited {} tokens to referral program", amount);
    Ok(())
}
