use anchor_lang::prelude::*;

/// Represents a participant in the referral program.
/// 
/// This struct stores information about a participant including their:
/// - Referral link for sharing with others
/// - Total number of successful referrals
/// - Total rewards earned
/// - Optional referrer if they joined through someone's link
#[account]
pub struct Participant {
    /// The owner of this participant account
    pub owner: Pubkey,
    /// The referral program this participant belongs to
    pub program: Pubkey,
    /// When this participant joined the program
    pub join_time: i64,
    /// Number of successful referrals made
    pub total_referrals: u64,
    /// Total rewards earned from referrals
    pub total_rewards: u64,
    /// Who referred this participant (if any)
    pub referrer: Option<Pubkey>,
    /// Unique referral link for this participant
    pub referral_link: [u8; 100],
}

impl Default for Participant {
    fn default() -> Self {
        Self {
            owner: Pubkey::default(),
            program: Pubkey::default(),
            join_time: 0,
            total_referrals: 0,
            total_rewards: 0,
            referrer: None,
            referral_link: [0u8; 100],
        }
    }
}
