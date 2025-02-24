use crate::{constants::*, error::*, state::*};
use anchor_lang::prelude::*;
use anchor_spl::token::{Mint, Token, TokenAccount};

/// Accounts for creating a new referral program.
///
/// This struct defines the accounts required for the `create_referral_program` instruction.
/// It includes the following accounts:
///
/// - `referral_program`: The account that will store the referral program data.
/// - `eligibility_criteria`: The account that will store the eligibility criteria for the referral program.
/// - `token_mint_info`: An optional account for the token mint to be used for payments. If not provided, the program
///   will use native SOL.
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
/// This function sets up a new referral program, including the referral program account and the eligibility criteria
/// account. It validates the input parameters and sets the initial values for the referral program and eligibility
/// criteria.
///
/// # Parameters
/// - `ctx`: The context for the `CreateReferralProgram` accounts.
/// - `token_mint`: An optional token mint account to be used for payments. If not provided, the program will use native
///   SOL.
/// - `fixed_reward_amount`: The fixed reward amount for referrals.
/// - `locked_period`: The locked period for referral rewards.
/// - `early_redemption_fee`: The fee for early redemption of referral rewards.
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
#[allow(clippy::too_many_arguments)]
pub fn create_referral_program(
    ctx: Context<CreateReferralProgram>,
    token_mint: Option<Pubkey>,
    fixed_reward_amount: u64,
    program_end_time: i64,
) -> Result<()> {
    // Validate base parameters
    require!(fixed_reward_amount >= MIN_REWARD_AMOUNT, ReferralError::InvalidRewardAmount);

    let current_time = Clock::get()?.unix_timestamp;
    require!(program_end_time > current_time, ReferralError::InvalidEndTime);

    // Set up referral program
    let referral_program = &mut ctx.accounts.referral_program;
    referral_program.authority = ctx.accounts.authority.key();
    referral_program.token_mint = token_mint.unwrap_or_default();
    referral_program.fixed_reward_amount = fixed_reward_amount;
    referral_program.is_active = true;
    referral_program.bump = ctx.bumps.referral_program;

    // Set up eligibility criteria
    let criteria = &mut ctx.accounts.eligibility_criteria;
    let clock = Clock::get()?;


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
#[allow(clippy::too_many_arguments)]
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
    program_end_time: i64,
) -> Result<()> {
    let criteria = &mut ctx.accounts.eligibility_criteria;
    let clock = Clock::get()?;

    // Validate parameters
    require!(base_reward >= MIN_REWARD_AMOUNT, ReferralError::InvalidRewardAmount);
    require!(tier1_reward >= base_reward, ReferralError::InvalidTierReward);
    require!(tier2_reward >= tier1_reward, ReferralError::InvalidTierReward);
    require!(tier2_threshold > tier1_threshold, ReferralError::InvalidTierThreshold);
    require!(revenue_share_percent <= MAX_FEE_PERCENTAGE, ReferralError::InvalidFeeAmount);

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

/// Accounts required for initializing the token vault for a referral program.
///
/// This struct defines the accounts and constraints required to initialize a PDA token account
/// that will serve as the vault for storing deposited tokens in a token-based referral program.
/// The vault is a Program Derived Address (PDA) with seeds ["token_vault", referral_program.key()].
///
/// Required accounts:
/// - `referral_program`: The referral program account that must be active and token-based
/// - `token_vault`: The PDA token account that will be initialized to store deposited tokens
/// - `token_mint`: The mint of the token that matches the referral program's configuration
/// - `authority`: The signer with authority over the referral program
/// - `system_program`: Required for account creation
/// - `token_program`: Required for token account initialization
/// - `rent`: Required for rent-exempt account creation
#[derive(Accounts)]
pub struct InitializeTokenVault<'info> {
    #[account(
        mut,
        constraint = referral_program.is_active @ ReferralError::ProgramInactive,
        has_one = authority @ ReferralError::InvalidAuthority,
        constraint = referral_program.token_mint != Pubkey::default() @ ReferralError::InvalidTokenMint,
    )]
    pub referral_program: Account<'info, ReferralProgram>,

    /// Token account vault that will hold deposited tokens
    /// PDA with seeds: ["token_vault", referral_program.key()]
    #[account(
        init,
        payer = authority,
        seeds = [b"token_vault", referral_program.key().as_ref()],
        bump,
        token::mint = token_mint,
        token::authority = referral_program,
    )]
    pub token_vault: Account<'info, TokenAccount>,

    /// The mint of the token for deposits
    #[account(
        constraint = token_mint.key() == referral_program.token_mint @ ReferralError::InvalidTokenMint
    )]
    pub token_mint: Account<'info, Mint>,

    #[account(mut)]
    pub authority: Signer<'info>,

    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub rent: Sysvar<'info, Rent>,
}

/// Initializes the token vault for a token-based referral program.
///
/// This instruction is a crucial step in setting up a token-based referral program:
/// 1. It must be called after creating a referral program with a token mint
/// 2. It creates and initializes a PDA token account that will hold all deposited tokens
/// 3. It must be completed before any token deposits can be made to the program
///
/// The initialization process:
/// - Creates a new token account as a PDA (Program Derived Address)
/// - Sets the referral program as the authority over the vault
/// - Configures the vault to accept only the correct token type
///
/// # Arguments
/// * `ctx` - Contains all required accounts including:
///   - The referral program that must be active and configured for tokens
///   - The token vault PDA that will be initialized
///   - The token mint that must match the program's configuration
///   - The authority who must be the program's authority
///
/// # Errors
/// * `ProgramInactive` - If the referral program is not active
/// * `InvalidAuthority` - If the signer is not the program authority
/// * `InvalidTokenMint` - If the referral program is not configured for tokens
///
/// # Example Flow
/// ```ignore
/// 1. Create referral program with token_mint
/// 2. Call initialize_token_vault to create the vault
/// 3. Users can then deposit tokens to the program
/// ```
pub fn initialize_token_vault(ctx: Context<InitializeTokenVault>) -> Result<()> {
    msg!("Initialized token vault for referral program {}", ctx.accounts.referral_program.key());
    Ok(())
}

/// Settings that can be updated for a referral program
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct ProgramSettings {
    /// The fixed reward amount for referrals
    pub fixed_reward_amount: u64,
    /// The locked period for referral rewards
    pub locked_period: i64,
    /// Optional end time for the referral program
    pub program_end_time: i64,
    /// The base reward amount for referrals
    pub base_reward: u64,
    /// The maximum reward cap
    pub max_reward_cap: u64,
}

/// Accounts required for updating program settings
#[derive(Accounts)]
pub struct UpdateProgramSettings<'info> {
    #[account(
        mut,
        constraint = referral_program.authority == authority.key(),
        constraint = referral_program.is_active @ ReferralError::ProgramInactive,
    )]
    pub referral_program: Account<'info, ReferralProgram>,

    #[account(
        mut,
        seeds = [b"eligibility_criteria", referral_program.key().as_ref()],
        bump
    )]
    pub eligibility_criteria: Account<'info, EligibilityCriteria>,

    #[account(mut)]
    pub authority: Signer<'info>,

    pub system_program: Program<'info, System>,
}

/// Updates the settings of an existing referral program
///
/// This function allows the program authority to update various settings of the referral program,
/// such as reward amounts, locked periods, and fees. It validates the new settings to ensure they
/// meet the program's requirements.
///
/// # Arguments
/// * `ctx` - The context for the UpdateProgramSettings instruction
/// * `new_settings` - The new settings to apply to the program
///
/// # Returns
/// * `Result<()>` - Returns Ok(()) if successful, or an error if validation fails
pub fn update_program_settings(
    ctx: Context<UpdateProgramSettings>,
    new_settings: ProgramSettings,
) -> Result<()> {
    let current_time = Clock::get()?.unix_timestamp;

    // Core reward amount validations
    require!(
        new_settings.fixed_reward_amount >= MIN_REWARD_AMOUNT,
        ReferralError::InvalidRewardAmount
    );
    require!(
        new_settings.base_reward >= MIN_REWARD_AMOUNT,
        ReferralError::InvalidRewardAmount
    );
    require!(
        new_settings.max_reward_cap >= new_settings.fixed_reward_amount 
        && new_settings.max_reward_cap >= new_settings.base_reward,
        ReferralError::InvalidRewardCap
    );

    // Time period validations
    require!(
        new_settings.locked_period >= MIN_LOCKED_PERIOD && new_settings.locked_period <= MAX_LOCKED_PERIOD,
        ReferralError::InvalidLockedPeriod
    );
    let end_time = new_settings.program_end_time;
    require!(
        end_time > current_time,
        ReferralError::InvalidProgramEndTime
    );
    // Ensure end time is after locked period
    require!(
        end_time > current_time + new_settings.locked_period,
        ReferralError::InvalidProgramEndTime
    );

    // Update core program settings
    let program = &mut ctx.accounts.referral_program;
    program.fixed_reward_amount = new_settings.fixed_reward_amount;
    program.locked_period = new_settings.locked_period;

    // Update eligibility criteria
    let criteria = &mut ctx.accounts.eligibility_criteria;
    criteria.program_end_time = new_settings.program_end_time;
    criteria.base_reward = new_settings.base_reward;
    criteria.max_reward_cap = new_settings.max_reward_cap;
    criteria.last_updated = current_time;

    Ok(())
}
