use anchor_lang::prelude::*;

/// Represents various error conditions that can occur in the referral system.
///
/// These errors are used to provide meaningful error messages to users and developers
/// when certain invalid conditions are encountered, such as invalid reward amounts,
/// fee amounts, locked periods, minimum stake amounts, tier reward amounts, or
/// tier thresholds.
#[error_code]
pub enum ReferralError {
    #[msg("Invalid reward amount - must be greater than MIN_REWARD_AMOUNT")]
    InvalidRewardAmount,
    #[msg("Invalid fee amount - must be less than MAX_FEE_PERCENTAGE")]
    InvalidFeeAmount,
    #[msg("Invalid locked period - must be between MIN_LOCKED_PERIOD and MAX_LOCKED_PERIOD")]
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
    #[msg("Invalid mint fee - must be less than or equal to MAX_MINT_FEE")]
    InvalidMintFee,
    #[msg("Invalid early redemption fee - must be less than or equal to MAX_EARLY_REDEMPTION_FEE")]
    InvalidEarlyRedemptionFee,
    #[msg("Invalid program end time - must be in the future and after locked period")]
    InvalidProgramEndTime,
    #[msg("Invalid reward cap - must be greater than or equal to fixed and base rewards")]
    InvalidRewardCap,
    #[msg("Invalid minimum token amount - must be greater than 0 for token-based programs")]
    InvalidMinTokenAmount,
    #[msg("Invalid referrer provided")]
    InvalidReferrer,
    #[msg("No rewards available to claim")]
    NoRewardsAvailable,
    #[msg("Rewards are still locked")]
    RewardsLocked,
    #[msg("Insufficient vault balance")]
    InsufficientVaultBalance,
    #[msg("Invalid time period")]
    InvalidEndTime,
    #[msg("Overflow when calculating rewards")]
    NumericOverflow,
    #[msg("Insufficient funds")]
    InsufficientFunds,
    #[msg("Lock period has not elapsed yet")]
    LockPeriodNotElapsed,
}
