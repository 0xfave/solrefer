use anchor_client::solana_sdk::{pubkey::Pubkey, signer::Signer};
use anchor_spl::token::spl_token;
use solrefer::state::ReferralProgram;

use crate::test_util::{
    create_mint, create_sol_referral_program, create_token_account, deposit_sol, mint_tokens, setup,
};

#[test]
fn test_create_sol_referral_program() {
    let (owner, _, _, program_id, client) = setup();

    // Test parameters
    let fixed_reward_amount = 1000000; // 1 SOL
    let locked_period = 7 * 24 * 60 * 60; // 7 days in seconds
    let early_redemption_fee = 1000; // 10% in basis points
    let mint_fee = 500; // 5% in basis points

    // Create SOL referral program
    let (referral_program_pubkey, _) = create_sol_referral_program(
        &owner,
        &client,
        program_id,
        1_000_000,            // 0.001 SOL fixed reward
        locked_period,        // 1 minute locked period
        early_redemption_fee, // 25% early redemption fee in basis points
        mint_fee,             // 10% mint fee in basis points
        50_000_000,           // 0.05 SOL base reward
        5,                    // 5 referrals for tier 1
        75_000_000,           // 0.075 SOL tier1 reward (> base_reward)
        10,                   // 10 referrals for tier 2
        100_000_000,          // 0.1 SOL tier2 reward (> tier1_reward)
        1_000_000_000,        // 1 SOL max reward cap
        500,                  // 5% revenue share
        None,                 // No required token
        0,                    // No min token amount
        None,                 // No end time
    );

    // Verify the created program
    let referral_program: ReferralProgram = client
        .program(program_id)
        .unwrap()
        .account(referral_program_pubkey)
        .expect("Failed to fetch referral program account");

    assert_eq!(referral_program.authority, owner.pubkey());
    assert_eq!(referral_program.token_mint, Pubkey::default()); // Default pubkey means SOL
    assert_eq!(referral_program.fixed_reward_amount, fixed_reward_amount);
    assert_eq!(referral_program.locked_period, locked_period);
    assert_eq!(referral_program.early_redemption_fee, early_redemption_fee);
    assert_eq!(referral_program.mint_fee, mint_fee);
    assert_eq!(referral_program.total_referrals, 0);
    assert_eq!(referral_program.total_rewards_distributed, 0);
    assert!(referral_program.is_active);

    // Find PDA for vault
    let (vault, _) =
        Pubkey::find_program_address(&[b"vault", referral_program_pubkey.as_ref()], &program_id);

    // Test depositing SOL
    let deposit_amount = 500_000_000; // 0.5 SOL
    let tx = deposit_sol(
        deposit_amount,
        referral_program_pubkey,
        &owner,
        &client,
        program_id,
        vault,
    );

    println!("Deposited SOL. Transaction signature: {}", tx);

    // Verify the vault balance
    let vault_balance = client
        .program(program_id)
        .unwrap()
        .rpc()
        .get_balance(&vault)
        .expect("Failed to get vault balance");

    assert_eq!(
        vault_balance, deposit_amount,
        "Vault balance should match deposit amount"
    );
}

#[test]
#[should_panic(expected = "TokenDepositToSolProgram")]
fn test_sol_referral_program_not_sol_deposit() {
    let (owner, _, _, program_id, client) = setup();

    // Create a SOL referral program
    let (referral_program_pubkey, vault) = create_sol_referral_program(
        &owner,
        &client,
        program_id,
        1_000_000,     // 0.001 SOL fixed reward
        60,            // 1 minute locked period
        2500,          // 25% early redemption fee in basis points
        1000,          // 10% mint fee in basis points
        50_000_000,    // 0.05 SOL base reward
        5,             // 5 referrals for tier 1
        75_000_000,    // 0.075 SOL tier1 reward (> base_reward)
        10,            // 10 referrals for tier 2
        100_000_000,   // 0.1 SOL tier2 reward (> tier1_reward)
        1_000_000_000, // 1 SOL max reward cap
        500,           // 5% revenue share
        None,          // No required token
        0,             // No min token amount
        None,          // No end time
    );

    // Create a token mint and account to test invalid deposits
    let mint = create_mint(&owner, &client, program_id);
    let owner_token_account = create_token_account(&owner, &mint.pubkey(), &client, program_id);
    mint_tokens(
        &mint,
        &owner_token_account,
        &owner,
        1_000_000_000,
        &client,
        program_id,
    );

    // Test case 1: Try to deposit 0 SOL (should fail)
    let result = std::panic::catch_unwind(|| {
        deposit_sol(
            0,
            referral_program_pubkey,
            &owner,
            &client,
            program_id,
            vault,
        )
    });
    assert!(result.is_err(), "Should fail when depositing 0 SOL");

    // Test case 2: Try to deposit tokens to SOL program (should fail)
    // This will trigger the TokenDepositToSolProgram error
    let _ = client
        .program(program_id)
        .unwrap()
        .request()
        .accounts(solrefer::accounts::DepositToken {
            referral_program: referral_program_pubkey,
            token_vault: vault, // Using SOL vault as token vault (should fail)
            token_mint: mint.pubkey(),
            depositor_token_account: owner_token_account,
            authority: owner.pubkey(),
            token_program: spl_token::id(),
        })
        .args(solrefer::instruction::DepositToken { amount: 1_000_000 })
        .signer(&owner)
        .send()
        .expect("Transaction failed but not with TokenDepositToSolProgram error");
}
