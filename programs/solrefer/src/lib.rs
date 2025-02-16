pub mod constants;
pub mod error;
pub mod instructions;
pub mod state;

use anchor_lang::prelude::*;
use instructions::*;

declare_id!("DvdCTkZBHpUpPYAccKkN3DQtu69GCEre3gsPJ7r33W35");

#[program]
pub mod solrefer {
    use super::*;

    /// Creates a new referral program with the specified parameters.
    ///
    /// This function sets up a new referral program with the provided configuration options.
    /// The referral program allows users to earn rewards for referring others to the program.
    /// The program can have various tiers and thresholds for earning rewards, as well as
    /// a fixed reward amount, locked period, early redemption fee, mint fee, and more.
    ///
    /// # Arguments
    ///
    /// * `ctx` - The context for the create referral program instruction.
    /// * `token_mint` - The optional token mint for the referral program rewards.
    /// * `fixed_reward_amount` - The fixed amount of rewards for each referral.
    /// * `locked_period` - The period of time the rewards are locked before they can be redeemed.
    /// * `early_redemption_fee` - The fee charged for redeeming rewards early.
    /// * `mint_fee` - The fee charged for minting new rewards.
    /// * `base_reward` - The base reward amount.
    /// * `tier1_threshold` - The threshold for the first tier of rewards.
    /// * `tier1_reward` - The reward amount for the first tier.
    /// * `tier2_threshold` - The threshold for the second tier of rewards.
    /// * `tier2_reward` - The reward amount for the second tier.
    /// * `max_reward_cap` - The maximum total reward amount that can be earned.
    /// * `revenue_share_percent` - The percentage of revenue shared with referrers.
    /// * `required_token` - The optional token required to participate in the referral program.
    /// * `min_token_amount` - The minimum token amount required to participate.
    /// * `program_end_time` - The optional end time for the referral program.
    #[allow(clippy::too_many_arguments)]
    pub fn create_referral_program(
        ctx: Context<CreateReferralProgram>,
        token_mint: Option<Pubkey>,
        fixed_reward_amount: u64,
        locked_period: i64,
        early_redemption_fee: u64,
        mint_fee: u64,
        base_reward: u64,
        tier1_threshold: u64,
        tier1_reward: u64,
        tier2_threshold: u64,
        tier2_reward: u64,
        max_reward_cap: u64,
        revenue_share_percent: u64,
        required_token: Option<Pubkey>,
        min_token_amount: u64,
        program_end_time: Option<i64>,
    ) -> Result<()> {
        instructions::referral_program::create_referral_program(
            ctx,
            token_mint,
            fixed_reward_amount,
            locked_period,
            early_redemption_fee,
            mint_fee,
            base_reward,
            tier1_threshold,
            tier1_reward,
            tier2_threshold,
            tier2_reward,
            max_reward_cap,
            revenue_share_percent,
            required_token,
            min_token_amount,
            program_end_time,
        )
    }

    /// Initializes the token vault for a token-based referral program.
    ///
    /// This instruction creates and initializes the token vault account that will hold
    /// deposited tokens for the referral program. This must be called after creating
    /// a token-based referral program and before any token deposits can be made.
    ///
    /// # Arguments
    /// * `ctx` - The context containing:
    ///   - referral_program: The program account (must be active)
    ///   - token_vault: The token vault PDA to initialize
    ///   - token_mint: The token mint (must match program config)
    ///   - authority: The program authority (signer)
    ///   - system_program: The system program
    ///   - token_program: The token program
    ///   - rent: The rent sysvar
    ///
    /// # Errors
    /// * `ProgramInactive` - If the referral program is not active
    /// * `InvalidAuthority` - If the signer is not the program authority
    /// * `InvalidTokenMint` - If the referral program is not configured for tokens
    pub fn initialize_token_vault(ctx: Context<InitializeTokenVault>) -> Result<()> {
        instructions::referral_program::initialize_token_vault(ctx)
    }

    /// Deposits SOL into the referral program's vault.
    ///
    /// This instruction allows the program authority to deposit SOL that will be used
    /// to pay out referral rewards. The program must be configured for SOL deposits.
    ///
    /// # Arguments
    /// * `ctx` - The deposit context containing:
    ///   - referral_program: The program account (must be active)
    ///   - vault: The SOL vault PDA
    ///   - authority: The program authority (signer)
    ///   - system_program: The system program
    /// * `amount` - Amount to deposit in lamports
    ///
    /// # Errors
    /// * `ProgramInactive` - If the referral program is not active
    /// * `InvalidAuthority` - If the signer is not the program authority
    /// * `InsufficientDeposit` - If the deposit amount is zero
    /// * `SolDepositToTokenProgram` - If attempting SOL deposit to a token program
    pub fn deposit_sol(ctx: Context<DepositSol>, amount: u64) -> Result<()> {
        instructions::deposit::deposit_sol(ctx, amount)
    }

    /// Deposits tokens into the referral program's vault.
    ///
    /// This instruction allows the program authority to deposit SPL tokens that will be used
    /// to pay out referral rewards. The program must be configured for token deposits.
    ///
    /// # Arguments
    /// * `ctx` - The deposit context containing:
    ///   - referral_program: The program account (must be active)
    ///   - token_vault: The token vault PDA
    ///   - token_mint: The token mint (must match program config)
    ///   - depositor_token_account: The authority's token account
    ///   - authority: The program authority (signer)
    ///   - token_program: The token program
    /// * `amount` - Amount to deposit in token units
    ///
    /// # Errors
    /// * `ProgramInactive` - If the referral program is not active
    /// * `InvalidAuthority` - If the signer is not the program authority
    /// * `InvalidTokenProgram` - If the token program is incorrect
    /// * `InvalidTokenMint` - If the token mint doesn't match program's configuration
    /// * `InvalidTokenAccounts` - If the token accounts are invalid
    /// * `InsufficientDeposit` - If the deposit amount is zero
    /// * `TokenDepositToSolProgram` - If attempting token deposit to a SOL program
    pub fn deposit_token(ctx: Context<DepositToken>, amount: u64) -> Result<()> {
        instructions::deposit::deposit_token(ctx, amount)
    }
}
