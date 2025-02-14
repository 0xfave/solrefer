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

    /// Deposits funds into the referral program's vault.
    ///
    /// This instruction allows the program authority to deposit funds that will be used
    /// to pay out referral rewards. The type of deposit (SOL or SPL token) must match
    /// the referral program's configuration.
    ///
    /// # Arguments
    /// * `ctx` - The context containing all required accounts
    /// * `is_token_deposit` - Flag indicating if this is a token deposit (true) or SOL deposit (false)
    /// * `amount` - Amount to deposit (in lamports for SOL, or token amount for SPL tokens)
    ///
    /// The deposit will be validated to ensure:
    /// - The depositor is the program authority
    /// - The program is active
    /// - The deposit amount is greater than 0
    /// - For token deposits, the correct token mint and program are used
    pub fn deposit(
        ctx: Context<Deposit>,
        is_token_deposit: bool,
        amount: u64,
    ) -> Result<()> {
        instructions::deposit::deposit(ctx, is_token_deposit, amount)
    }
}
