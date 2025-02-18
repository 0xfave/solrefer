use anchor_client::solana_sdk::{pubkey::Pubkey, signer::Signer, system_program, system_instruction, signature::Keypair};
use solrefer::state::Participant;
use std::str;

use crate::test_util::{create_sol_referral_program, setup};

#[test]
fn test_join_referral_program_sucesss() {
    let (owner, alice, _, program_id, client) = setup();

    // Create a SOL referral program
    let (referral_program_pubkey, _) = create_sol_referral_program(
        &owner,
        &client,
        program_id,
        1_000_000,             // 1 SOL fixed reward
        7 * 24 * 60 * 60,      // 7 days locked period
        1_000_000_000,         // 1 SOL max reward cap
        None,                  // No end time
    );

    // Calculate PDA for participant account
    let (participant_pubkey, _) = Pubkey::find_program_address(
        &[
            b"participant",
            referral_program_pubkey.as_ref(),
            alice.pubkey().as_ref(),
        ],
        &program_id,
    );

    // Join the referral program
    let program = client.program(program_id).unwrap();
    program
        .request()
        .accounts(solrefer::accounts::JoinReferralProgram {
            referral_program: referral_program_pubkey,
            participant: participant_pubkey,
            user: alice.pubkey(),
            system_program: system_program::ID,
            rent: anchor_client::solana_sdk::sysvar::rent::ID,
        })
        .args(solrefer::instruction::JoinReferralProgram {})
        .signer(&alice)
        .send()
        .unwrap();

    // Verify participant account was created correctly
    let participant_account: Participant = program
        .account(participant_pubkey)
        .unwrap();
    assert_eq!(participant_account.owner, alice.pubkey());
    assert_eq!(participant_account.program, referral_program_pubkey);
    assert_eq!(participant_account.total_referrals, 0);
    assert_eq!(participant_account.total_rewards, 0);
    assert_eq!(participant_account.referrer, None);

    // Convert bytes to string, trimming null bytes
    let referral_link = str::from_utf8(&participant_account.referral_link)
        .unwrap()
        .trim_matches(char::from(0));
    assert_eq!(
        referral_link,
        format!("https://solrefer.io/ref/{}", alice.pubkey())
    );
}

#[test]
fn test_join_through_referral_success() {
    let (owner, alice, bob, program_id, client) = setup();

    // Create a SOL referral program
    let (referral_program_pubkey, _) = create_sol_referral_program(
        &owner,
        &client,
        program_id,
        1_000_000,             // 1 SOL fixed reward
        7 * 24 * 60 * 60,      // 7 days locked period
        1_000_000_000,         // 1 SOL max reward cap
        None,                  // No end time
    );

    // Calculate PDA for referrer's participant account
    let (referrer_participant_pubkey, _) = Pubkey::find_program_address(
        &[
            b"participant",
            referral_program_pubkey.as_ref(),
            alice.pubkey().as_ref(),
        ],
        &program_id,
    );

    // Alice joins normally first
    let program = client.program(program_id).unwrap();
    program
        .request()
        .accounts(solrefer::accounts::JoinReferralProgram {
            referral_program: referral_program_pubkey,
            participant: referrer_participant_pubkey,
            user: alice.pubkey(),
            system_program: system_program::ID,
            rent: anchor_client::solana_sdk::sysvar::rent::ID,
        })
        .args(solrefer::instruction::JoinReferralProgram {})
        .signer(&alice)
        .send()
        .unwrap();

    // Calculate PDA for Bob's participant account
    let (participant_pubkey, _) = Pubkey::find_program_address(
        &[
            b"participant",
            referral_program_pubkey.as_ref(),
            bob.pubkey().as_ref(),
        ],
        &program_id,
    );

    // Bob joins through Alice's referral
    program
        .request()
        .accounts(solrefer::accounts::JoinThroughReferral {
            referral_program: referral_program_pubkey,
            participant: participant_pubkey,
            referrer: referrer_participant_pubkey,
            user: bob.pubkey(),
            system_program: system_program::ID,
            rent: anchor_client::solana_sdk::sysvar::rent::ID,
        })
        .args(solrefer::instruction::JoinThroughReferral {})
        .signer(&bob)
        .send()
        .unwrap();

    // Verify Bob's participant account was created correctly
    let participant_account: Participant = program
        .account(participant_pubkey)
        .unwrap();
    assert_eq!(participant_account.owner, bob.pubkey());
    assert_eq!(participant_account.program, referral_program_pubkey);
    assert_eq!(participant_account.total_referrals, 0);
    assert_eq!(participant_account.total_rewards, 0);
    assert_eq!(participant_account.referrer, Some(referrer_participant_pubkey));

    // Convert bytes to string, trimming null bytes
    let referral_link = str::from_utf8(&participant_account.referral_link)
        .unwrap()
        .trim_matches(char::from(0));
    assert_eq!(
        referral_link,
        format!("https://solrefer.io/ref/{}", bob.pubkey())
    );

    // Verify Alice's stats were updated
    let referrer_account: Participant = program
        .account(referrer_participant_pubkey)
        .unwrap();
    assert_eq!(referrer_account.total_referrals, 1);
}

#[test]
#[should_panic(expected = "InvalidReferrer")]
fn test_join_through_invalid_referral() {
    let (owner, _, bob, program_id, client) = setup();

    // Create a SOL referral program
    let (referral_program_pubkey, _) = create_sol_referral_program(
        &owner,
        &client,
        program_id,
        1_000_000,             // 1 SOL fixed reward
        7 * 24 * 60 * 60,      // 7 days locked period
        1_000_000_000,         // 1 SOL max reward cap
        None,                  // No end time
    );

    // Create a keypair for the invalid account
    let invalid_account = Keypair::new();

    // Create a regular account at a normal address (not a PDA)
    let program = client.program(program_id).unwrap();
    program
        .request()
        .instruction(system_instruction::create_account(
            &bob.pubkey(),
            &invalid_account.pubkey(),
            10_000_000, // 0.01 SOL
            0,          // No data
            &system_program::ID, // Owned by system program
        ))
        .signer(&bob)
        .signer(&invalid_account)
        .send()
        .unwrap();

    // Calculate PDA for Bob's participant account
    let (participant_pubkey, _) = Pubkey::find_program_address(
        &[
            b"participant",
            referral_program_pubkey.as_ref(),
            bob.pubkey().as_ref(),
        ],
        &program_id,
    );

    // Try to join through invalid referral - should fail with InvalidReferrer
    let err = program
        .request()
        .accounts(solrefer::accounts::JoinThroughReferral {
            referral_program: referral_program_pubkey,
            participant: participant_pubkey,
            referrer: invalid_account.pubkey(),
            user: bob.pubkey(),
            system_program: system_program::ID,
            rent: anchor_client::solana_sdk::sysvar::rent::ID,
        })
        .args(solrefer::instruction::JoinThroughReferral {})
        .signer(&bob)
        .send()
        .unwrap_err();

    assert!(err.to_string().contains("InvalidReferrer"));
}
