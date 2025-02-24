use crate::error::*;
use crate::instructions::VAULT_SEED;
use crate::state::*;
use anchor_lang::prelude::*;
use anchor_lang::system_program::{transfer, Transfer};

#[derive(Accounts)]
pub struct ClaimRewards<'info> {
    #[account(mut)]
    pub referral_program: Account<'info, ReferralProgram>,
    #[account(
        mut,
        seeds = [
            b"participant",
            referral_program.key().as_ref(),
            user.key().as_ref()
        ],
        bump
    )]
    pub participant: Account<'info, Participant>,
    #[account(
        mut,
        seeds = [b"vault", referral_program.key().as_ref()],
        bump
    )]
    pub vault: SystemAccount<'info>,
    #[account(mut)]
    pub user: Signer<'info>,
    pub system_program: Program<'info, System>,
}

pub fn process_claim_rewards(ctx: Context<ClaimRewards>) -> Result<()> {
    let referral_program = &mut ctx.accounts.referral_program;
    let participant = &mut ctx.accounts.participant;
    
    // Verify program is active
    require!(referral_program.is_active, ReferralError::ProgramInactive);
    
    // Calculate rewards amount
    let reward_amount = calculate_reward_share(
        participant.total_referrals,
        referral_program.total_participants,
        referral_program.total_available
    );

    // Transfer from vault using seeds signing
    let binding = referral_program.key();
    let seeds = &[
        VAULT_SEED,
        binding.as_ref(),
        &[referral_program.vault_bump], // Use the vault_bump from the referral program
    ];
    let signer = &[&seeds[..]];
    
    // Transfer rewards to participant
    let transfer_ctx = CpiContext::new_with_signer(
        ctx.accounts.system_program.to_account_info(),
        Transfer {
            from: ctx.accounts.vault.to_account_info(),
            to: ctx.accounts.user.to_account_info(),
        },
        signer,
    );
    
    transfer(transfer_ctx, reward_amount)?;
    
    // Update participant state
    participant.total_rewards = participant.total_rewards
        .checked_add(reward_amount)
        .ok_or(ReferralError::NumericOverflow)?;

    referral_program.total_available = referral_program.total_available
        .checked_sub(reward_amount)
        .ok_or(ReferralError::InsufficientFunds)?;
    
    referral_program.total_rewards_distributed = referral_program.total_rewards_distributed
        .checked_add(reward_amount)
        .ok_or(ReferralError::NumericOverflow)?;
    
    Ok(())
}

fn calculate_reward_share(participant_referrals: u64, total_participants: u64, total_available: u64) -> u64 {
    // Implement reward distribution formula here
    // Example: proportional distribution based on referral count
    if total_participants == 0 {
        return 0;
    }
    (participant_referrals * total_available) / total_participants
}
