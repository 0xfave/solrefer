use anchor_client::solana_sdk::{pubkey::Pubkey, signer::Signer, system_program};
use anchor_spl::token::spl_token;
use solrefer::{state::{ReferralProgram, EligibilityCriteria}, instructions::ProgramSettings};

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

    // Create SOL referral program
    let (referral_program_pubkey, _) = create_sol_referral_program(
        &owner,
        &client,
        program_id,
        fixed_reward_amount,    // 1 SOL fixed reward
        locked_period,          // 7 days locked period
        early_redemption_fee,   // 10% early redemption fee
        None,            // 0.05 SOL base reward
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
    assert_eq!(referral_program.total_referrals, 0);
    assert_eq!(referral_program.total_rewards_distributed, 0);
    assert!(referral_program.is_active);

    // Find PDA for vault
    let (vault, _) = Pubkey::find_program_address(&[b"vault", referral_program_pubkey.as_ref()], &program_id);

    // Test depositing SOL
    let deposit_amount = 500_000_000; // 0.5 SOL
    let tx = deposit_sol(deposit_amount, referral_program_pubkey, &owner, &client, program_id, vault);

    println!("Deposited SOL. Transaction signature: {}", tx);

    // Verify the vault balance
    let vault_balance =
        client.program(program_id).unwrap().rpc().get_balance(&vault).expect("Failed to get vault balance");

    assert_eq!(vault_balance, deposit_amount, "Vault balance should match deposit amount");
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
        1_000_000,             // 0.001 SOL fixed reward
        60,                    // 1 minute locked period
        2500,                  // 25% early redemption fee
        None         // No end time
    );

    // Create a token mint and account to test invalid deposits
    let mint = create_mint(&owner, &client, program_id);
    let owner_token_account = create_token_account(&owner, &mint.pubkey(), &client, program_id);
    mint_tokens(&mint, &owner_token_account, &owner, 1_000_000_000, &client, program_id);

    // Test case 1: Try to deposit 0 SOL (should fail)
    let result =
        std::panic::catch_unwind(|| deposit_sol(0, referral_program_pubkey, &owner, &client, program_id, vault));
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

#[test]
fn test_update_program_settings_success() {
    let (owner, _, _, program_id, client) = setup();

    // Create a SOL referral program
    let (referral_program_pubkey, _) = create_sol_referral_program(
        &owner,
        &client,
        program_id,
        1_000_000,             // 0.001 SOL fixed reward
        60,                    // 1 minute locked period
        2500,                  // 25% early redemption fee
        None     // No end time
    );

    // Find eligibility criteria PDA
    let (eligibility_criteria_pubkey, _) = Pubkey::find_program_address(
        &[b"eligibility_criteria", referral_program_pubkey.as_ref()],
        &program_id,
    );

    // New settings to update
    let new_settings = ProgramSettings {
        fixed_reward_amount: 2_000_000,     // 0.002 SOL fixed reward
        locked_period: 86400,              // 1 day locked period (minimum allowed)
        program_end_time: Some(i64::MAX),   // Set end time to max
        base_reward: 75_000_000,            // 0.075 SOL base reward
        max_reward_cap: 1_000_000_000,      // 1 SOL max reward cap
    };

    // Update program settings
    let tx = client
        .program(program_id)
        .unwrap()
        .request()
        .accounts(solrefer::accounts::UpdateProgramSettings {
            referral_program: referral_program_pubkey,
            eligibility_criteria: eligibility_criteria_pubkey,
            authority: owner.pubkey(),
            system_program: system_program::ID,
        })
        .args(solrefer::instruction::UpdateProgramSettings {
            new_settings: new_settings.clone(),
        })
        .signer(&owner)
        .send()
        .expect("Failed to update program settings");

    println!("Updated program settings. Transaction signature: {}", tx);

    // Verify the updated settings
    let referral_program: ReferralProgram = client
        .program(program_id)
        .unwrap()
        .account(referral_program_pubkey)
        .expect("Failed to fetch referral program account");

    assert_eq!(referral_program.fixed_reward_amount, new_settings.fixed_reward_amount);
    assert_eq!(referral_program.locked_period, new_settings.locked_period);
    // Verify eligibility criteria updates
    let eligibility_criteria: EligibilityCriteria = client
        .program(program_id)
        .unwrap()
        .account(eligibility_criteria_pubkey)
        .expect("Failed to fetch eligibility criteria account");

    assert_eq!(eligibility_criteria.base_reward, new_settings.base_reward);
    assert_eq!(eligibility_criteria.max_reward_cap, new_settings.max_reward_cap);
    assert_eq!(eligibility_criteria.program_end_time, new_settings.clone().program_end_time);
}

#[test]
fn test_update_program_settings_invalid_reward_amount() {
    let (owner, _, _, program_id, client) = setup();

    // Create a SOL referral program with valid settings
    let (referral_program_pubkey, _) = create_sol_referral_program(
        &owner,
        &client,
        program_id,
        1_000_000,             // 0.001 SOL fixed reward
        86400,                 // 1 day locked period
        2500,                  // 25% early redemption fee
        None
    );

    // Find eligibility criteria PDA
    let (eligibility_criteria_pubkey, _) = Pubkey::find_program_address(
        &[b"eligibility_criteria", referral_program_pubkey.as_ref()],
        &program_id,
    );

    // Test case 1: Zero fixed reward amount
    let invalid_settings_1 = ProgramSettings {
        fixed_reward_amount: 0,            // Invalid: Zero reward
        locked_period: 86400,              // 1 day
        program_end_time: None,
        base_reward: 50_000_000,           // 0.05 SOL
        max_reward_cap: 1_000_000_000,     // 1 SOL
    };

    let result = client
        .program(program_id)
        .unwrap()
        .request()
        .accounts(solrefer::accounts::UpdateProgramSettings {
            referral_program: referral_program_pubkey,
            eligibility_criteria: eligibility_criteria_pubkey,
            authority: owner.pubkey(),
            system_program: system_program::ID,
        })
        .args(solrefer::instruction::UpdateProgramSettings {
            new_settings: invalid_settings_1.clone(),
        })
        .signer(&owner)
        .send();

    assert!(result.is_err(), "Expected error for zero reward amount");

    // Test case 2: Base reward greater than max reward cap
    let invalid_settings_2 = ProgramSettings {
        fixed_reward_amount: 1_000_000,     // 0.001 SOL
        locked_period: 86400,               // 1 day
        program_end_time: None,
        base_reward: 2_000_000_000,         // Invalid: 2 SOL base reward > 1 SOL max cap
        max_reward_cap: 1_000_000_000,      // 1 SOL
    };

    let result = client
        .program(program_id)
        .unwrap()
        .request()
        .accounts(solrefer::accounts::UpdateProgramSettings {
            referral_program: referral_program_pubkey,
            eligibility_criteria: eligibility_criteria_pubkey,
            authority: owner.pubkey(),
            system_program: system_program::ID,
        })
        .args(solrefer::instruction::UpdateProgramSettings {
            new_settings: invalid_settings_2.clone(),
        })
        .signer(&owner)
        .send();

    assert!(result.is_err(), "Expected error for base reward > max reward cap");
}

#[test]
fn test_update_program_settings_invalid_end_time() {
    let (owner, _, _, program_id, client) = setup();

    // Create a SOL referral program with valid settings
    let (referral_program_pubkey, _) = create_sol_referral_program(
        &owner,
        &client,
        program_id,
        1_000_000,             // 0.001 SOL fixed reward
        86400,                 // 1 day locked period
        2500,                  // 25% early redemption fee
        None
    );

    // Find eligibility criteria PDA
    let (eligibility_criteria_pubkey, _) = Pubkey::find_program_address(
        &[b"eligibility_criteria", referral_program_pubkey.as_ref()],
        &program_id,
    );

    // Get current time
    let current_time = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64;

    // Test case 1: End time in the past
    let invalid_settings_1 = ProgramSettings {
        fixed_reward_amount: 1_000_000,     // 0.001 SOL
        locked_period: 86400,               // 1 day
        program_end_time: Some(current_time - 1), // Invalid: End time in the past
        base_reward: 50_000_000,            // 0.05 SOL
        max_reward_cap: 1_000_000_000,      // 1 SOL
    };

    let result = client
        .program(program_id)
        .unwrap()
        .request()
        .accounts(solrefer::accounts::UpdateProgramSettings {
            referral_program: referral_program_pubkey,
            eligibility_criteria: eligibility_criteria_pubkey,
            authority: owner.pubkey(),
            system_program: system_program::ID,
        })
        .args(solrefer::instruction::UpdateProgramSettings {
            new_settings: invalid_settings_1.clone(),
        })
        .signer(&owner)
        .send();

    assert!(result.is_err(), "Expected error for end time in the past");

    // Test case 2: End time before locked period ends
    let invalid_settings_2 = ProgramSettings {
        fixed_reward_amount: 1_000_000,     // 0.001 SOL
        locked_period: 86400,               // 1 day
        program_end_time: Some(current_time + 3600), // Invalid: End time only 1 hour in future (less than locked period)
        base_reward: 50_000_000,            // 0.05 SOL
        max_reward_cap: 1_000_000_000,      // 1 SOL
    };

    let result = client
        .program(program_id)
        .unwrap()
        .request()
        .accounts(solrefer::accounts::UpdateProgramSettings {
            referral_program: referral_program_pubkey,
            eligibility_criteria: eligibility_criteria_pubkey,
            authority: owner.pubkey(),
            system_program: system_program::ID,
        })
        .args(solrefer::instruction::UpdateProgramSettings {
            new_settings: invalid_settings_2.clone(),
        })
        .signer(&owner)
        .send();

    assert!(result.is_err(), "Expected error for end time before locked period ends");
}

#[test]
fn test_update_program_settings_invalid_locked_period() {
    let (owner, _, _, program_id, client) = setup();

    // Create a SOL referral program with valid settings
    let (referral_program_pubkey, _) = create_sol_referral_program(
        &owner,
        &client,
        program_id,
        1_000_000,             // 0.001 SOL fixed reward
        86400,                 // 1 day locked period
        2500,                  // 25% early redemption fee
        None
    );

    // Find eligibility criteria PDA
    let (eligibility_criteria_pubkey, _) = Pubkey::find_program_address(
        &[b"eligibility_criteria", referral_program_pubkey.as_ref()],
        &program_id,
    );

    // Test case 1: Locked period too short (less than 1 day)
    let invalid_settings_1 = ProgramSettings {
        fixed_reward_amount: 1_000_000,     // 0.001 SOL
        locked_period: 3600,                // Invalid: Only 1 hour (minimum is 1 day)
        program_end_time: None,
        base_reward: 50_000_000,            // 0.05 SOL
        max_reward_cap: 1_000_000_000,      // 1 SOL
    };

    let result = client
        .program(program_id)
        .unwrap()
        .request()
        .accounts(solrefer::accounts::UpdateProgramSettings {
            referral_program: referral_program_pubkey,
            eligibility_criteria: eligibility_criteria_pubkey,
            authority: owner.pubkey(),
            system_program: system_program::ID,
        })
        .args(solrefer::instruction::UpdateProgramSettings {
            new_settings: invalid_settings_1.clone(),
        })
        .signer(&owner)
        .send();

    assert!(result.is_err(), "Expected error for locked period less than 1 day");

    // Test case 2: Locked period too long (more than 365 days)
    let invalid_settings_2 = ProgramSettings {
        fixed_reward_amount: 1_000_000,     // 0.001 SOL
        locked_period: 31536000 + 86400,    // Invalid: 366 days (maximum is 365 days)
        program_end_time: None,
        base_reward: 50_000_000,            // 0.05 SOL
        max_reward_cap: 1_000_000_000,      // 1 SOL
    };

    let result = client
        .program(program_id)
        .unwrap()
        .request()
        .accounts(solrefer::accounts::UpdateProgramSettings {
            referral_program: referral_program_pubkey,
            eligibility_criteria: eligibility_criteria_pubkey,
            authority: owner.pubkey(),
            system_program: system_program::ID,
        })
        .args(solrefer::instruction::UpdateProgramSettings {
            new_settings: invalid_settings_2.clone(),
        })
        .signer(&owner)
        .send();

    assert!(result.is_err(), "Expected error for locked period more than 365 days");
}
