pub mod constants;
pub mod error;
pub mod instructions;
pub mod state;

use anchor_lang::prelude::*;
use instructions::*;

declare_id!("DvdCTkZBHpUpPYAccKkN3DQtu69GCEre3gsPJ7r33W35");

#[program]
pub mod solrefer {
    use super::*;

    pub fn create_referral_program(
        ctx: Context<CreateReferralProgram>,
        token_mint: Option<Pubkey>,
        fixed_reward_amount: u64,
        locked_period: i64,
        early_redemption_fee: u64,
        mint_fee: u64,
        min_stake_amount: u64,
    ) -> Result<()> {
        instructions::referral_program::create_referral_program(
            ctx,
            token_mint,
            fixed_reward_amount,
            locked_period,
            early_redemption_fee,
            mint_fee,
            min_stake_amount,
        )
    }
}
