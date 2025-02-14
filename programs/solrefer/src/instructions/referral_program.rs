use anchor_lang::prelude::*;
use anchor_spl::token::{ Mint, Token };
use crate::state::*;
use crate::constants::*;
use crate::error::*;

#[derive(Accounts)]
#[instruction(token_mint: Option<Pubkey>)]
pub struct CreateReferralProgram<'info> {
    #[account(
        init,
        payer = authority,
        space = 8 + 32 + 32 + 8 + 8 + 8 + 8 + 8 + 8 + 8 + 1 + 1,
        seeds = [REFERRAL_PROGRAM_SEED, authority.key().as_ref()],
        bump
    )]
    pub referral_program: Account<'info, ReferralProgram>,
    
    /// Optional token mint account. If provided, the program will use this token for payments
    /// If not provided (None), the program will use native SOL
    #[account(
        mut,
        constraint = token_mint.map_or(true, |mint| mint == token_mint_info.key())
    )]
    pub token_mint_info: Option<Account<'info, Mint>>,
    
    #[account(mut)]
    pub authority: Signer<'info>,
    
    pub system_program: Program<'info, System>,
    pub token_program: Option<Program<'info, Token>>,
}

pub fn create_referral_program(
    ctx: Context<CreateReferralProgram>,
    token_mint: Option<Pubkey>,
    fixed_reward_amount: u64,
    locked_period: i64,
    early_redemption_fee: u64,
    mint_fee: u64,
    min_stake_amount: u64,
) -> Result<()> {
    require!(
        fixed_reward_amount >= MIN_REWARD_AMOUNT,
        ReferralError::InvalidRewardAmount
    );
    require!(
        early_redemption_fee <= MAX_FEE_PERCENTAGE,
        ReferralError::InvalidFeeAmount
    );
    
    let referral_program = &mut ctx.accounts.referral_program;
    
    referral_program.authority = ctx.accounts.authority.key();
    referral_program.token_mint = token_mint.unwrap_or(Pubkey::default());
    referral_program.fixed_reward_amount = fixed_reward_amount;
    referral_program.locked_period = locked_period;
    referral_program.early_redemption_fee = early_redemption_fee;
    referral_program.mint_fee = mint_fee;
    referral_program.min_stake_amount = min_stake_amount;
    referral_program.total_referrals = 0;
    referral_program.total_rewards_distributed = 0;
    referral_program.is_active = true;
    referral_program.bump = ctx.bumps.referral_program;

    msg!("Created referral program with authority: {:?}", referral_program.authority);
    Ok(())
}
