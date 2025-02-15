use anchor_lang::prelude::*;

#[account]
/// Represents the state of a referral program.
///
/// This struct contains the core configuration and state of a referral program,
/// including the program authority, token mint, reward amounts, time parameters,
/// and program status.
pub struct ReferralProgram {
    pub authority: Pubkey,              // 32
    pub token_mint: Pubkey,             // 32 (Optional, if None/zero pubkey then use SOL)
    pub fixed_reward_amount: u64,       // 8
    pub locked_period: i64,             // 8
    pub early_redemption_fee: u64,      // 8
    pub mint_fee: u64,                  // 8
    pub min_stake_amount: u64,          // 8
    pub total_referrals: u64,           // 8
    pub total_rewards_distributed: u64, // 8
    pub is_active: bool,                // 1
    pub bump: u8,                       // 1
}

/// The size of the `ReferralProgram` account in bytes.
///
/// This constant defines the total size of the `ReferralProgram` account, including
/// the discriminator, all the fields, and any padding required by the Solana
/// runtime.
impl ReferralProgram {
    pub const SIZE: usize = 8 + // discriminator
        32 + // authority
        32 + // token_mint
        8 + // fixed_reward_amount
        8 + // locked_period
        8 + // early_redemption_fee
        8 + // mint_fee
        8 + // min_stake_amount
        8 + // total_referrals
        8 + // total_rewards_distributed
        1 + // is_active
        1; // bump
}

/// Represents the eligibility criteria for a referral program.
///
/// This struct contains the configuration for the reward structure, token
/// requirements, time parameters, and program status for a referral program.
/// The fields in this struct define the rules and conditions that determine
/// whether a user is eligible to receive rewards from the referral program.
#[account]
#[derive(Default)]
pub struct EligibilityCriteria {
    // Core Reward Structure
    pub base_reward: u64,           // 8
    pub tier1_threshold: u64,       // 8
    pub tier1_reward: u64,          // 8
    pub tier2_threshold: u64,       // 8
    pub tier2_reward: u64,          // 8
    pub max_reward_cap: u64,        // 8
    pub revenue_share_percent: u64, // 8

    // Optional Token Requirement
    pub required_token: Option<Pubkey>, // 32 + 1
    pub min_token_amount: u64,          // 8

    // Time Parameters
    pub program_start_time: i64,       // 8
    pub program_end_time: Option<i64>, // 8 + 1

    // Status
    pub is_active: bool,   // 1
    pub last_updated: i64, // 8
    pub bump: u8,          // 1
}

/// Defines the total size of the `EligibilityCriteria` account, including the
/// discriminator, all the fields, and any padding required by the Solana runtime.
impl EligibilityCriteria {
    pub const SIZE: usize = 8 + // discriminator
        8 * 7 + // reward structure (u64s)
        (32 + 1) + // required_token (Option<Pubkey>)
        8 + // min_token_amount
        8 + // program_start_time
        (8 + 1) + // program_end_time (Option<i64>)
        1 + // is_active
        8 + // last_updated
        1; // bump
}
