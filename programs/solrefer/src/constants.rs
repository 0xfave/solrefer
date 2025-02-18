/// The seed used for the referral program's Pubkey.
pub const REFERRAL_PROGRAM_SEED: &[u8] = b"referral_program";
/// The minimum reward amount for the referral program.
pub const MIN_REWARD_AMOUNT: u64 = 1;
/// The maximum fee percentage allowed for the referral program, expressed in basis points (1/100th of a percent).
pub const MAX_FEE_PERCENTAGE: u64 = 5000; // 50% in basis points

/// The maximum early redemption fee allowed, expressed in basis points (1/100th of a percent).
pub const MAX_EARLY_REDEMPTION_FEE: u64 = 3000; // 30% in basis points

/// The minimum locked period for rewards in seconds (1 day).
pub const MIN_LOCKED_PERIOD: i64 = 86400;

/// The maximum locked period for rewards in seconds (365 days).
pub const MAX_LOCKED_PERIOD: i64 = 31536000;
