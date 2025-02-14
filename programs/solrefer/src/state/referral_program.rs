use anchor_lang::prelude::*;

#[account]
pub struct ReferralProgram {
    pub authority: Pubkey,            // 32
    pub token_mint: Pubkey,          // 32 (Optional, if None/zero pubkey then use SOL)
    pub fixed_reward_amount: u64,     // 8
    pub locked_period: i64,           // 8
    pub early_redemption_fee: u64,    // 8
    pub mint_fee: u64,               // 8
    pub min_stake_amount: u64,       // 8
    pub total_referrals: u64,        // 8
    pub total_rewards_distributed: u64, // 8
    pub is_active: bool,             // 1
    pub bump: u8,                    // 1
}
