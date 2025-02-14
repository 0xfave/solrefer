use anchor_lang::prelude::*;

#[error_code]
pub enum ReferralError {
    #[msg("Invalid reward amount")]
    InvalidRewardAmount,
    #[msg("Invalid fee amount")]
    InvalidFeeAmount,
    #[msg("Invalid locked period")]
    InvalidLockedPeriod,
    #[msg("Invalid minimum stake amount")]
    InvalidMinStakeAmount,
}
