use anchor_client::{
    anchor_lang,
    solana_sdk::{pubkey::Pubkey, signer::Signer, system_program},
};
use anchor_spl::token::spl_token;
use solrefer::state::ReferralProgram;

use crate::test_util::{create_mint, create_token_account, deposit_tokens, mint_tokens, setup};
#[test]
fn test_create_referral_program_with_token_mint() {
    let (owner, _, _, program_id, client) = setup();

    // Create new token mint
    let mint = create_mint(&owner, &client, program_id);

    // Test parameters
    let fixed_reward_amount = 1_000_000_000; // 1 token
    let locked_period = 7 * 24 * 60 * 60; // 7 days in seconds
    let early_redemption_fee = 1000; // 10% in basis points
    let mint_fee = 500; // 5% in basis points

    // Find PDA for referral program
    let binding = owner.pubkey();
    let seeds = [b"referral_program".as_ref(), binding.as_ref()];
    let (referral_program_pubkey, _) = Pubkey::find_program_address(&seeds, &program_id);

    // Find PDA for eligibility criteria
    let (eligibility_criteria, _bump) = Pubkey::find_program_address(
        &[b"eligibility_criteria", referral_program_pubkey.as_ref()],
        &program_id,
    );

    // Create token referral program
    let tx = client
        .program(program_id)
        .unwrap()
        .request()
        .accounts(solrefer::accounts::CreateReferralProgram {
            referral_program: referral_program_pubkey,
            eligibility_criteria,
            authority: owner.pubkey(),
            token_mint_info: Some(mint.pubkey()),
            system_program: system_program::ID,
            token_program: Some(spl_token::id()),
        })
        .args(solrefer::instruction::CreateReferralProgram {
            token_mint: Some(mint.pubkey()),
            fixed_reward_amount,
            locked_period,
            early_redemption_fee,
            mint_fee,
            base_reward: 50_000_000,       // 0.05 SOL base reward
            tier1_threshold: 5,            // 5 referrals for tier 1
            tier1_reward: 75_000_000,      // 0.075 SOL tier 1 reward
            tier2_threshold: 10,           // 10 referrals for tier 2
            tier2_reward: 100_000_000,     // 0.1 SOL tier 2 reward
            max_reward_cap: 1_000_000_000, // 1 SOL max rewards
            revenue_share_percent: 500,    // 5% revenue share
            required_token: None,
            min_token_amount: 0,
            program_end_time: None,
        })
        .signer(&owner)
        .send()
        .expect("Failed to create token referral program");

    println!(
        "Created token referral program. Transaction signature: {}",
        tx
    );

    // Verify the created program
    let referral_program: ReferralProgram = client
        .program(program_id)
        .unwrap()
        .account(referral_program_pubkey)
        .expect("Failed to fetch referral program account");

    assert_eq!(referral_program.authority, owner.pubkey());
    assert_eq!(referral_program.token_mint, mint.pubkey());
    assert_eq!(referral_program.fixed_reward_amount, fixed_reward_amount);
    assert_eq!(referral_program.locked_period, locked_period);
    assert_eq!(referral_program.early_redemption_fee, early_redemption_fee);
    assert_eq!(referral_program.mint_fee, mint_fee);
    assert_eq!(referral_program.total_referrals, 0);
    assert_eq!(referral_program.total_rewards_distributed, 0);
    assert!(referral_program.is_active);

    // Find PDA for token vault
    let (token_vault, _) = Pubkey::find_program_address(
        &[b"token_vault", referral_program_pubkey.as_ref()],
        &program_id,
    );

    // Initialize token vault
    let tx = client
        .program(program_id)
        .unwrap()
        .request()
        .accounts(solrefer::accounts::InitializeTokenVault {
            referral_program: referral_program_pubkey,
            token_vault,
            token_mint: mint.pubkey(),
            authority: owner.pubkey(),
            system_program: system_program::ID,
            token_program: spl_token::id(),
            rent: anchor_lang::solana_program::sysvar::rent::ID,
        })
        .args(solrefer::instruction::InitializeTokenVault)
        .signer(&owner)
        .send()
        .expect("Failed to initialize token vault");

    println!("Initialized token vault. Transaction signature: {}", tx);

    // Create token account for owner
    let owner_token_account = create_token_account(&owner, &mint.pubkey(), &client, program_id);

    // Mint some tokens to owner's account
    let initial_token_amount = 10_000_000_000; // 10 tokens
    mint_tokens(
        &mint,
        &owner_token_account,
        &owner,
        initial_token_amount,
        &client,
        program_id,
    );

    // Test depositing tokens
    let deposit_amount = 500_000_000; // 0.5 tokens
    let tx = deposit_tokens(
        deposit_amount,
        referral_program_pubkey,
        token_vault,
        mint.pubkey(),
        owner_token_account,
        &owner,
        &client,
        program_id,
    );

    println!("Deposited tokens. Transaction signature: {}", tx);

    // Verify the token vault balance
    let vault_balance = client
        .program(program_id)
        .unwrap()
        .rpc()
        .get_token_account_balance(&token_vault)
        .expect("Failed to get token vault balance")
        .amount
        .parse::<u64>()
        .unwrap();

    assert_eq!(
        vault_balance, deposit_amount,
        "Token vault balance should match deposit amount"
    );

    // Verify owner's token account balance was reduced
    let owner_balance = client
        .program(program_id)
        .unwrap()
        .rpc()
        .get_token_account_balance(&owner_token_account)
        .expect("Failed to get owner token balance")
        .amount
        .parse::<u64>()
        .unwrap();

    assert_eq!(
        owner_balance,
        initial_token_amount - deposit_amount,
        "Owner token balance should be reduced by deposit amount"
    );
}
