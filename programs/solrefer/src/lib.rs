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

    /// Updates the settings of an existing referral program.
    ///
    /// This function allows the program authority to update various settings of the referral program,
    /// such as reward amounts, locked periods, and fees. It validates the new settings to ensure they
    /// meet the program's requirements.
    ///
    /// # Arguments
    /// * `ctx` - The context for the UpdateProgramSettings instruction
    /// * `new_settings` - The new settings to apply to the program
    pub fn update_program_settings(
        ctx: Context<UpdateProgramSettings>,
        new_settings: ProgramSettings,
    ) -> Result<()> {
        instructions::referral_program::update_program_settings(ctx, new_settings)
    }

    /// Allows a user to join a referral program as someone who wants to refer others.
    ///
    /// This instruction creates a new participant account for the user and generates
    /// their unique referral link that they can share with others. The user joins
    /// directly (not through a referral).
    ///
    /// # Arguments
    /// * `ctx` - The context containing:
    ///   - referral_program: The program account (must be active)
    ///   - participant: The new participant account to create
    ///   - user: The user joining the program (signer)
    ///   - system_program: The system program
    ///   - rent: The rent sysvar
    ///
    /// # Errors
    /// * `ProgramInactive` - If the referral program is not active
    pub fn join_referral_program(
        ctx: Context<JoinReferralProgram>,
    ) -> Result<()> {
        instructions::join_referral_program(ctx)
    }

    /// Join a referral program through someone's referral link.
    ///
    /// This instruction creates a new participant account for the user,
    /// credits the referrer, and generates a new referral link for the user
    /// to share with others.
    ///
    /// # Arguments
    /// * `ctx` - The context containing:
    ///   - referral_program: The program account (must be active)
    ///   - participant: The new participant account to create
    ///   - referrer: The referrer's participant account
    ///   - user: The user joining through the referral (signer)
    ///   - system_program: The system program
    ///   - rent: The rent sysvar
    ///
    /// # Errors
    /// * `ProgramInactive` - If the referral program is not active
    /// * `InvalidReferrer` - If the referrer is not part of this program
    pub fn join_through_referral(
        ctx: Context<JoinThroughReferral>,
    ) -> Result<()> {
        instructions::join_through_referral(ctx)
    }
}
