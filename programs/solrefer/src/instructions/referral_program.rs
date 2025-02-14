use anchor_lang::prelude::*;
use anchor_spl::token::{ Mint, Token };
use crate::state::*;
use crate::constants::*;
use crate::error::*;

/// Accounts for creating a new referral program.
///
/// This struct defines the accounts required for the `create_referral_program` instruction.
/// It includes the following accounts:
///
/// - `referral_program`: The account that will store the referral program data.
/// - `eligibility_criteria`: The account that will store the eligibility criteria for the referral program.
/// - `token_mint_info`: An optional account for the token mint to be used for payments. If not provided, the program will use native SOL.
/// - `authority`: The signer account that will create the referral program.
/// - `system_program`: The system program account.
/// - `token_program`: An optional token program account.
#[derive(Accounts)]
#[instruction(token_mint: Option<Pubkey>)]
pub struct CreateReferralProgram<'info> {
    #[account(
        init,
        payer = authority,
        space = 8 + ReferralProgram::SIZE,
        seeds = [b"referral_program", authority.key().as_ref()],
        bump
    )]
    pub referral_program: Account<'info, ReferralProgram>,

    #[account(
        init,
        payer = authority,
        space = 8 + EligibilityCriteria::SIZE,
        seeds = [b"eligibility_criteria", referral_program.key().as_ref()],
        bump
    )]
    pub eligibility_criteria: Account<'info, EligibilityCriteria>,

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

/// Creates a new referral program with the specified parameters.
///
/// This function sets up a new referral program, including the referral program account and the eligibility criteria account.
/// It validates the input parameters and sets the initial values for the referral program and eligibility criteria.
///
/// # Parameters
/// - `ctx`: The context for the `CreateReferralProgram` accounts.
/// - `token_mint`: An optional token mint account to be used for payments. If not provided, the program will use native SOL.
/// - `fixed_reward_amount`: The fixed reward amount for referrals.
/// - `locked_period`: The locked period for referral rewards.
/// - `early_redemption_fee`: The fee for early redemption of referral rewards.
/// - `mint_fee`: The fee for minting referral rewards.
/// - `base_reward`: The base reward amount for referrals.
/// - `tier1_threshold`: The threshold for the first tier of referral rewards.
/// - `tier1_reward`: The reward amount for the first tier of referrals.
/// - `tier2_threshold`: The threshold for the second tier of referral rewards.
/// - `tier2_reward`: The reward amount for the second tier of referrals.
/// - `max_reward_cap`: The maximum total reward cap for the referral program.
/// - `revenue_share_percent`: The percentage of revenue to be shared with referrers.
/// - `required_token`: An optional token required for eligibility.
/// - `min_token_amount`: The minimum amount of the required token needed for eligibility.
/// - `program_end_time`: An optional end time for the referral program.
///
/// # Returns
/// A `Result` indicating whether the referral program was created successfully.
pub fn create_referral_program(
    ctx: Context<CreateReferralProgram>,
    token_mint: Option<Pubkey>,
    fixed_reward_amount: u64,
    locked_period: i64,
    early_redemption_fee: u64,
    mint_fee: u64,
    // Eligibility criteria parameters
    base_reward: u64,
    tier1_threshold: u64,
    tier1_reward: u64,
    tier2_threshold: u64,
    tier2_reward: u64,
    max_reward_cap: u64,
    revenue_share_percent: u64,
    required_token: Option<Pubkey>,
    min_token_amount: u64,
    program_end_time: Option<i64>,
) -> Result<()> {
    // Validate base parameters
    require!(
        fixed_reward_amount >= MIN_REWARD_AMOUNT,
        ReferralError::InvalidRewardAmount
    );
    require!(
        early_redemption_fee <= MAX_FEE_PERCENTAGE,
        ReferralError::InvalidFeeAmount
    );
    
    // Validate eligibility parameters
    require!(
        base_reward >= MIN_REWARD_AMOUNT,
        ReferralError::InvalidRewardAmount
    );
    require!(
        tier1_reward >= base_reward,
        ReferralError::InvalidTierReward
    );
    require!(
        tier2_reward >= tier1_reward,
        ReferralError::InvalidTierReward
    );
    require!(
        tier2_threshold > tier1_threshold,
        ReferralError::InvalidTierThreshold
    );
    require!(
        revenue_share_percent <= MAX_FEE_PERCENTAGE,
        ReferralError::InvalidFeeAmount
    );
    
    // Set up referral program
    let referral_program = &mut ctx.accounts.referral_program;
    referral_program.authority = ctx.accounts.authority.key();
    referral_program.token_mint = token_mint.unwrap_or(Pubkey::default());
    referral_program.fixed_reward_amount = fixed_reward_amount;
    referral_program.locked_period = locked_period;
    referral_program.early_redemption_fee = early_redemption_fee;
    referral_program.mint_fee = mint_fee;
    referral_program.is_active = true;
    referral_program.bump = ctx.bumps.referral_program;

    // Set up eligibility criteria
    let criteria = &mut ctx.accounts.eligibility_criteria;
    let clock = Clock::get()?;
    
    criteria.base_reward = base_reward;
    criteria.tier1_threshold = tier1_threshold;
    criteria.tier1_reward = tier1_reward;
    criteria.tier2_threshold = tier2_threshold;
    criteria.tier2_reward = tier2_reward;
    criteria.max_reward_cap = max_reward_cap;
    criteria.revenue_share_percent = revenue_share_percent;
    
    criteria.required_token = required_token;
    criteria.min_token_amount = min_token_amount;
    
    criteria.program_start_time = clock.unix_timestamp;
    criteria.program_end_time = program_end_time;
    
    criteria.is_active = true;
    criteria.last_updated = clock.unix_timestamp;
    
    msg!("Created referral program with authority: {:?}", referral_program.authority);
    Ok(())
}

/// Accounts required for the `SetEligibilityCriteria` instruction.
///
/// - `eligibility_criteria`: The account that stores the eligibility criteria for the referral program.
/// - `referral_program`: The referral program account, which must have the same authority as the signer.
/// - `authority`: The signer account that has authority over the referral program.
/// - `system_program`: The system program account.
#[derive(Accounts)]
pub struct SetEligibilityCriteria<'info> {
    #[account(
        mut,
        seeds = [b"eligibility_criteria", referral_program.key().as_ref()],
        bump
    )]
    pub eligibility_criteria: Account<'info, EligibilityCriteria>,
    
    #[account(
        mut,
        constraint = referral_program.authority == authority.key()
    )]
    pub referral_program: Account<'info, ReferralProgram>,
    
    #[account(mut)]
    pub authority: Signer<'info>,
    
    pub system_program: Program<'info, System>,
}

/// Sets the eligibility criteria for a referral program.
///
/// This function updates the eligibility criteria for a referral program, including the reward structure,
/// requirements, and time parameters. The function validates the input parameters to ensure they meet the
/// program's requirements.
///
/// # Arguments
/// * `ctx` - The context for the `SetEligibilityCriteria` instruction.
/// * `base_reward` - The base reward amount for the referral program.
/// * `tier1_threshold` - The threshold for the first tier of the referral program.
/// * `tier1_reward` - The reward amount for the first tier of the referral program.
/// * `tier2_threshold` - The threshold for the second tier of the referral program.
/// * `tier2_reward` - The reward amount for the second tier of the referral program.
/// * `max_reward_cap` - The maximum reward cap for the referral program.
/// * `revenue_share_percent` - The revenue share percentage for the referral program.
/// * `required_token` - The token required for participation in the referral program.
/// * `min_token_amount` - The minimum token amount required for participation in the referral program.
/// * `program_end_time` - The end time for the referral program.
///
/// # Returns
/// A `Result` indicating whether the operation was successful.
pub fn set_eligibility_criteria(
    ctx: Context<SetEligibilityCriteria>,
    base_reward: u64,
    tier1_threshold: u64,
    tier1_reward: u64,
    tier2_threshold: u64,
    tier2_reward: u64,
    max_reward_cap: u64,
    revenue_share_percent: u64,
    required_token: Option<Pubkey>,
    min_token_amount: u64,
    program_end_time: Option<i64>,
) -> Result<()> {
    let criteria = &mut ctx.accounts.eligibility_criteria;
    let clock = Clock::get()?;
    
    // Validate parameters
    require!(
        base_reward >= MIN_REWARD_AMOUNT,
        ReferralError::InvalidRewardAmount
    );
    require!(
        tier1_reward >= base_reward,
        ReferralError::InvalidTierReward
    );
    require!(
        tier2_reward >= tier1_reward,
        ReferralError::InvalidTierReward
    );
    require!(
        tier2_threshold > tier1_threshold,
        ReferralError::InvalidTierThreshold
    );
    require!(
        revenue_share_percent <= MAX_FEE_PERCENTAGE,
        ReferralError::InvalidFeeAmount
    );
    
    // Set reward structure
    criteria.base_reward = base_reward;
    criteria.tier1_threshold = tier1_threshold;
    criteria.tier1_reward = tier1_reward;
    criteria.tier2_threshold = tier2_threshold;
    criteria.tier2_reward = tier2_reward;
    criteria.max_reward_cap = max_reward_cap;
    criteria.revenue_share_percent = revenue_share_percent;
    
    // Set requirements
    criteria.required_token = required_token;
    criteria.min_token_amount = min_token_amount;
    
    // Set time parameters
    criteria.program_start_time = clock.unix_timestamp;
    criteria.program_end_time = program_end_time;
    
    // Update status
    criteria.is_active = true;
    criteria.last_updated = clock.unix_timestamp;
    
    Ok(())
}
