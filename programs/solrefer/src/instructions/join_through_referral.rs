use crate::{error::ReferralError, state::{referral_program::*, participant::*}};
use anchor_lang::{
    prelude::*,
    system_program::System,
};
use std::mem::size_of;

pub fn join_through_referral(
    ctx: Context<JoinThroughReferral>,
) -> Result<()> {
    // 1. Verify program is active
    require!(
        ctx.accounts.referral_program.is_active,
        ReferralError::ProgramInactive
    );

    // 2. Verify referrer exists and is valid
    require!(
        ctx.accounts.referrer.program == ctx.accounts.referral_program.key(),
        ReferralError::InvalidReferrer
    );

    // 3. Create participant account
    let participant = &mut ctx.accounts.participant;
    participant.owner = ctx.accounts.user.key();
    participant.program = ctx.accounts.referral_program.key();
    participant.join_time = Clock::get()?.unix_timestamp;
    participant.total_referrals = 0;
    participant.total_rewards = 0;
    participant.referrer = Some(ctx.accounts.referrer.key());

    // Create referral link
    let referral_link = format!("https://solrefer.io/ref/{}", ctx.accounts.user.key());
    let mut referral_link_bytes = [0u8; 100];
    let bytes = referral_link.as_bytes();
    referral_link_bytes[..bytes.len()].copy_from_slice(bytes);
    participant.referral_link = referral_link_bytes;

    // 4. Update referrer's stats
    let referrer = &mut ctx.accounts.referrer;
    referrer.total_referrals = referrer.total_referrals.checked_add(1).unwrap();

    // Log the referral link for frontend to pick up
    msg!("referral_link:{}", referral_link);

    Ok(())
}

#[derive(Accounts)]
pub struct JoinThroughReferral<'info> {
    #[account(mut)]
    pub referral_program: Account<'info, ReferralProgram>,

    #[account(
        init,
        payer = user,
        space = 8 + size_of::<Participant>(),
        seeds = [
            b"participant",
            referral_program.key().as_ref(),
            user.key().as_ref(),
        ],
        bump
    )]
    pub participant: Account<'info, Participant>,

    #[account(mut)]
    pub referrer: Account<'info, Participant>,

    #[account(mut)]
    pub user: Signer<'info>,

    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}
