use anchor_lang::{
    prelude::*,
    system_program::{self, System, Transfer},
};
use anchor_spl::token::{self, Token, TokenAccount, Mint};
use crate::{
    state::referral_program::*,
    error::ReferralError,
};

/// The seed used for deriving the vault PDA that holds SOL deposits
pub const VAULT_SEED: &[u8] = b"vault";
/// The seed used for deriving the token vault PDA that holds token deposits
pub const TOKEN_VAULT_SEED: &[u8] = b"token_vault";

/// Accounts required for depositing funds into the referral program.
/// The structure adapts based on whether it's a SOL or token deposit.
///
/// PDAs:
/// 1. Vault (SOL deposits):
///    - Seeds: ["vault", referral_program.key()]
///    - Owner: System Program
///    - Contains: Native SOL for rewards
///
/// 2. Token Vault (SPL Token deposits):
///    - Seeds: ["token_vault", referral_program.key()]
///    - Owner: Token Program
///    - Contains: SPL Tokens for rewards
///    - Authority: Referral Program PDA
#[derive(Accounts)]
#[instruction(is_token_deposit: bool)]
pub struct Deposit<'info> {
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

    /// Token account vault that holds deposited tokens
    /// PDA with seeds: ["token_vault", referral_program.key()]
    #[account(
        mut,
        seeds = [TOKEN_VAULT_SEED, referral_program.key().as_ref()],
        bump,
        token::mint = token_mint,
        token::authority = referral_program,
        constraint = token_vault.owner == token_program.key() @ ReferralError::InvalidTokenProgram,
    )]
    pub token_vault: Account<'info, TokenAccount>,

    /// The mint of the token for deposits
    #[account(
        constraint = !is_token_deposit || token_mint.key() == referral_program.token_mint @ ReferralError::InvalidTokenMint
    )]
    pub token_mint: Account<'info, Mint>,

    /// The depositor's token account
    #[account(
        mut,
        constraint = !is_token_deposit || (
            depositor_token_account.owner == authority.key() &&
            depositor_token_account.mint == token_mint.key() &&
            depositor_token_account.owner == token_program.key()
        ) @ ReferralError::InvalidTokenAccounts
    )]
    pub depositor_token_account: Account<'info, TokenAccount>,

    /// The authority/owner of the referral program
    #[account(mut)]
    pub authority: Signer<'info>,

    pub system_program: Program<'info, System>,
    
    #[account(
        constraint = token_program.key() == token::ID @ ReferralError::InvalidTokenProgram
    )]
    pub token_program: Program<'info, Token>,
}

/// Deposits funds into the referral program.
/// Handles both SOL and token deposits based on the referral program configuration.
///
/// # Arguments
/// * `ctx` - The deposit context
/// * `is_token_deposit` - Whether this is a token deposit
/// * `amount` - The amount to deposit (in lamports for SOL, or token amount for SPL tokens)
///
/// # Errors
/// * `ProgramInactive` - If the referral program is not active
/// * `InvalidAuthority` - If the signer is not the program authority
/// * `InvalidTokenProgram` - If the token program is incorrect
/// * `InvalidTokenMint` - If the token mint doesn't match the program's configuration
/// * `InvalidTokenAccounts` - If the token accounts are invalid
pub fn deposit(
    ctx: Context<Deposit>,
    is_token_deposit: bool,
    amount: u64,
) -> Result<()> {
    require!(amount > 0, ReferralError::InsufficientDeposit);
    
    let referral_program = &ctx.accounts.referral_program;
    
    // Validate deposit type matches program configuration
    let is_token_program = referral_program.token_mint != Pubkey::default();
    if is_token_deposit && !is_token_program {
        return err!(ReferralError::TokenDepositToSolProgram);
    }
    if !is_token_deposit && is_token_program {
        return err!(ReferralError::SolDepositToTokenProgram);
    }
    
    if is_token_deposit {
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
    } else {
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
    }

    msg!("Deposited {} to referral program", amount);
    Ok(())
}
