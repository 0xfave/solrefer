use anchor_lang::prelude::*;

/// Represents various error conditions that can occur in the referral system.
///
/// These errors are used to provide meaningful error messages to users and developers
/// when certain invalid conditions are encountered, such as invalid reward amounts,
/// fee amounts, locked periods, minimum stake amounts, tier reward amounts, or
/// tier thresholds.
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
    #[msg("Invalid tier reward amount")]
    InvalidTierReward,
    #[msg("Invalid tier threshold")]
    InvalidTierThreshold,
    #[msg("Program is not active")]
    ProgramInactive,
    #[msg("Invalid authority")]
    InvalidAuthority,
    #[msg("Invalid token accounts provided")]
    InvalidTokenAccounts,
    #[msg("Insufficient deposit amount")]
    InsufficientDeposit,
    #[msg("Invalid token mint")]
    InvalidTokenMint,
    #[msg("Invalid token program")]
    InvalidTokenProgram,
    #[msg("Cannot deposit tokens to a SOL-based referral program")]
    TokenDepositToSolProgram,
    #[msg("Cannot deposit SOL to a token-based referral program")]
    SolDepositToTokenProgram,
}
