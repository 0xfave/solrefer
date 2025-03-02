use crate::test_util::{create_sol_referral_program, deposit_sol, setup};
use anchor_client::solana_sdk::{pubkey::Pubkey, signer::Signer, system_program};
use solrefer::{instructions::VAULT_SEED, state::{Participant, ReferralProgram}};

#[test]
fn test_reward_claim() {
    // Setup test environment
    let (owner, referrer, referee, program_id, client) = setup();
    
    // Create referral program with rewards
    let fixed_reward_amount = 1_000_000_000; // 1 SOL
    
    let (referral_program_pubkey, _) = create_sol_referral_program(
        &owner,
        &client,
        program_id,
        fixed_reward_amount,    // 1 SOL fixed reward
        i64::MAX,            // Program end time
    );

    // Find PDA for vault
    let (vault, _) = Pubkey::find_program_address(&[VAULT_SEED, referral_program_pubkey.as_ref()], &program_id);

    // Fund vault
    let deposit_amount = 1_000_000_000; // 1 SOL
    deposit_sol(
        deposit_amount,
        referral_program_pubkey,
        &owner,
        &client,
        program_id,
        vault
    );

    // Join program and create referrals
    // Calculate PDA for participant account
    let (referrer_participant_pubkey, _) = Pubkey::find_program_address(
        &[b"participant", referral_program_pubkey.as_ref(), referrer.pubkey().as_ref()],
        &program_id,
    );

    let program = client.program(program_id).unwrap();
    program
        .request()
        .accounts(solrefer::accounts::JoinReferralProgram {
            referral_program: referral_program_pubkey,
            participant: referrer_participant_pubkey,
            user: referrer.pubkey(),
            system_program: system_program::ID,
            rent: anchor_client::solana_sdk::sysvar::rent::ID,
        })
        .args(solrefer::instruction::JoinReferralProgram {})
        .signer(&referrer)
        .send()
        .unwrap();

    // referrer refers referee
    // Calculate PDA for referee's participant account
    let (referee_participant_pubkey, _) = Pubkey::find_program_address(
        &[b"participant", referral_program_pubkey.as_ref(), referee.pubkey().as_ref()],
        &program_id,
    );

    // referee joins through Alice's referral
    program
        .request()
        .accounts(solrefer::accounts::JoinThroughReferral {
            referral_program: referral_program_pubkey,
            participant: referee_participant_pubkey,
            referrer: referrer_participant_pubkey,
            user: referee.pubkey(),
            system_program: system_program::ID,
            rent: anchor_client::solana_sdk::sysvar::rent::ID,
        })
        .args(solrefer::instruction::JoinThroughReferral {})
        .signer(&referee)
        .send()
        .unwrap();

    // Get vault balance before claiming
    let vault_balance_before = client.program(program_id).unwrap().rpc().get_balance(&vault).unwrap();

    // Get referrer's balance before claiming
    let referrer_balance_before = client.program(program_id).unwrap().rpc().get_balance(&referrer.pubkey()).unwrap();

    // Claim rewards
    let tx = program
        .request()
        .accounts(solrefer::accounts::ClaimRewards {
            referral_program: referral_program_pubkey,
            participant: referrer_participant_pubkey,
            vault,
            user: referrer.pubkey(),
            system_program: system_program::ID,
        })
        .args(solrefer::instruction::ClaimRewards {})
        .signer(&referrer)
        .send()
        .unwrap();

    // Get vault balance after claiming
    let vault_balance_after = client.program(program_id).unwrap().rpc().get_balance(&vault).unwrap();

    // Get referrer's balance after claiming
    let referrer_balance_after = client.program(program_id).unwrap().rpc().get_balance(&referrer.pubkey()).unwrap();

    // Verify reward distribution
    let participant: Participant = client.program(program_id).unwrap().account(referrer_participant_pubkey).unwrap();

    // Debug logs
    println!("Vault balance before claim: {}", vault_balance_before);
    println!("Referrer balance before claim: {}", referrer_balance_before);
    println!("Vault balance after claim: {}", vault_balance_after);
    println!("Referrer balance after claim: {}", referrer_balance_after);

    // Verify actual SOL transfer
    assert_eq!(referrer_balance_after - referrer_balance_before, fixed_reward_amount);

    assert_eq!(participant.total_rewards, fixed_reward_amount);

    let program_state: ReferralProgram = client.program(program_id).unwrap().account(referral_program_pubkey).unwrap();

    assert_eq!(program_state.total_rewards_distributed, fixed_reward_amount);
    assert_eq!(program_state.total_available, deposit_amount - fixed_reward_amount);
}
