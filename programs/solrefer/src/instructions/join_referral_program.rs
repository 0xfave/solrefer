use crate::{error::ReferralError, state::{referral_program::*, participant::*}};
use anchor_lang::{
    prelude::*,
    system_program::System,
};
use std::mem::size_of;

/// Join a referral program as a new participant who wants to refer others.
/// This creates their participant account and generates their unique referral link
/// that they can share with others.
pub fn join_referral_program(
    ctx: Context<JoinReferralProgram>,
) -> Result<()> {
    // 1. Verify program is active
    require!(
        ctx.accounts.referral_program.is_active,
        ReferralError::ProgramInactive
    );

    // 2. Create participant account
    let participant = &mut ctx.accounts.participant;
    participant.owner = ctx.accounts.user.key();
    participant.program = ctx.accounts.referral_program.key();
    participant.join_time = Clock::get()?.unix_timestamp;
    participant.total_referrals = 0;
    participant.total_rewards = 0;
    participant.referrer = None; // They are joining directly, not through a referral

    // Create referral link
    let referral_link = format!("https://solrefer.io/ref/{}", ctx.accounts.user.key());
    let mut referral_link_bytes = [0u8; 100];
    let bytes = referral_link.as_bytes();
    referral_link_bytes[..bytes.len()].copy_from_slice(bytes);
    participant.referral_link = referral_link_bytes;

    // Log the referral link for frontend to pick up
    msg!("referral_link:{}", referral_link);

    Ok(())
}

#[derive(Accounts)]
pub struct JoinReferralProgram<'info> {
    #[account(mut)]
    pub referral_program: Account<'info, ReferralProgram>,

    #[account(
        init,
        payer = user,
        space = 8 + size_of::<Participant>(),
        seeds = [
            b"participant",
            referral_program.key().as_ref(),
            user.key().as_ref()
        ],
        bump
    )]
    pub participant: Account<'info, Participant>,

    #[account(mut)]
    pub user: Signer<'info>,

    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}
