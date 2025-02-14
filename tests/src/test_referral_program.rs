use std::str::FromStr;
use std::sync::Arc;
use anchor_client::{
    solana_client::rpc_client::RpcClient,
    solana_sdk::{
        commitment_config::CommitmentConfig,
        native_token::LAMPORTS_PER_SOL,
        pubkey::Pubkey,
        signature::{read_keypair_file, Keypair},
        signer::Signer,
        system_program,
        system_instruction,
    },
    Client, Cluster
};
use anchor_spl::token::spl_token;
use solrefer::state::ReferralProgram;
use std::process::Command;

fn ensure_test_validator() -> RpcClient {
    let rpc_url = "http://localhost:8899";
    let rpc_client = RpcClient::new(rpc_url);

    // Try to connect to validator
    if let Err(_) = rpc_client.get_version() {
        println!("No validator detected, attempting to start one...");
        // Kill any existing validator process
        Command::new("pkill")
            .args(&["-f", "solana-test-validator"])
            .output()
            .ok();

        // Start new validator
        Command::new("solana-test-validator")
            .arg("--quiet")
            .spawn()
            .expect("Failed to start validator");

        // Wait for validator to start
        let mut attempts = 0;
        while attempts < 30 {
            if rpc_client.get_version().is_ok() {
                println!("Validator started successfully");
                std::thread::sleep(std::time::Duration::from_secs(2));
                break;
            }
            std::thread::sleep(std::time::Duration::from_secs(1));
            attempts += 1;
        }
        if attempts >= 30 {
            panic!("Failed to start validator after 30 seconds");
        }
    }
    rpc_client
}

fn request_airdrop_with_retries(rpc_client: &RpcClient, pubkey: &Pubkey, amount: u64) -> Result<(), String> {
    let max_retries = 5;
    let mut current_try = 0;

    while current_try < max_retries {
        match rpc_client.request_airdrop(pubkey, amount) {
            Ok(sig) => {
                let mut confirmed = false;
                for _ in 0..30 {
                    if let Ok(true) = rpc_client.confirm_transaction(&sig) {
                        confirmed = true;
                        break;
                    }
                    std::thread::sleep(std::time::Duration::from_millis(500));
                }
                if confirmed {
                    // Verify the balance actually increased
                    std::thread::sleep(std::time::Duration::from_secs(1));
                    if let Ok(balance) = rpc_client.get_balance(pubkey) {
                        if balance >= amount {
                            println!("Successfully airdropped {} SOL", amount as f64 / LAMPORTS_PER_SOL as f64);
                            return Ok(());
                        }
                    }
                }
            }
            Err(e) => println!("Airdrop failed: {}", e),
        }
        current_try += 1;
        if current_try < max_retries {
            println!("Retrying airdrop... (attempt {}/{})", current_try + 1, max_retries);
            std::thread::sleep(std::time::Duration::from_secs(2));
        }
    }
    Err(format!("Failed to airdrop after {} attempts", max_retries))
}

fn setup() -> (Keypair, Keypair, Keypair, Pubkey, Client<Arc<Keypair>>) {
    let program_id = "DvdCTkZBHpUpPYAccKkN3DQtu69GCEre3gsPJ7r33W35"; // Your program ID
    let anchor_wallet = std::env::var("ANCHOR_WALLET").unwrap();
    let payer = Arc::new(read_keypair_file(&anchor_wallet).unwrap());

    let client = Client::new_with_options(
        Cluster::Localnet,
        payer.clone(),
        CommitmentConfig::confirmed()
    );
    let program_id = Pubkey::from_str(program_id).unwrap();

    // Create wallets for owner, alice and bob
    let owner = Keypair::new();
    let alice = Keypair::new();
    let bob = Keypair::new();

    // Ensure validator is running and get client
    let rpc_client = ensure_test_validator();
    
    // Fund accounts with smaller amounts and multiple retries
    let fund_amount = LAMPORTS_PER_SOL * 2;
    for (name, kp) in [("owner", &owner), ("alice", &alice), ("bob", &bob)] {
        if let Err(e) = request_airdrop_with_retries(&rpc_client, &kp.pubkey(), fund_amount) {
            panic!("Failed to fund {}: {}", name, e);
        }
    }

    // Return the vault keypair, wallets, program ID, and client for reuse
    (owner, alice, bob, program_id, client)
}

#[test]
fn test_create_sol_referral_program() {
    let (owner, _, _, program_id, client) = setup();

    // Test parameters
    let fixed_reward_amount = 1000000; // 1 SOL
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

    // Create SOL referral program
    let tx = client
        .program(program_id)
        .unwrap()
        .request()
        .accounts(solrefer::accounts::CreateReferralProgram {
            referral_program: referral_program_pubkey,
            eligibility_criteria,
            authority: owner.pubkey(),
            token_mint_info: None,
            system_program: system_program::ID,
            token_program: None,
        })
        .args(solrefer::instruction::CreateReferralProgram {
            token_mint: None,
            fixed_reward_amount,
            locked_period,
            early_redemption_fee,
            mint_fee,
            base_reward: 50_000_000, // 0.05 SOL base reward
            tier1_threshold: 5, // 5 referrals for tier 1
            tier1_reward: 75_000_000, // 0.075 SOL tier 1 reward
            tier2_threshold: 10, // 10 referrals for tier 2
            tier2_reward: 100_000_000, // 0.1 SOL tier 2 reward
            max_reward_cap: 1_000_000_000, // 1 SOL max rewards
            revenue_share_percent: 500, // 5% revenue share
            required_token: None,
            min_token_amount: 0,
            program_end_time: None,
        })
        .signer(&owner)
        .send()
        .expect("Failed to create SOL referral program");

    println!("Created SOL referral program. Transaction signature: {}", tx);

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
}

#[test]
fn test_create_referral_program_with_token_mint() {
    let (owner, _, _, program_id, client) = setup();

    // Create new token mint
    let mint = Keypair::new();
    let mint_authority = &owner;
    
    // Create mint account
    let rpc_client = client.program(program_id).unwrap().rpc();
    let rent = rpc_client.get_minimum_balance_for_rent_exemption(82).unwrap();
    let ix = system_instruction::create_account(
        &owner.pubkey(),
        &mint.pubkey(),
        rent,
        82,
        &spl_token::id(),
    );
    
    let tx = client
        .program(program_id)
        .unwrap()
        .request()
        .instruction(ix)
        .signer(&owner)
        .signer(&mint)
        .send()
        .expect("Failed to create mint account");
    println!("Created mint account. Transaction signature: {}", tx);

    // Initialize mint
    let ix = spl_token::instruction::initialize_mint(
        &spl_token::id(),
        &mint.pubkey(),
        &mint_authority.pubkey(),
        Some(&mint_authority.pubkey()),
        9,
    ).unwrap();

    let tx = client
        .program(program_id)
        .unwrap()
        .request()
        .instruction(ix)
        .send()
        .expect("Failed to initialize mint");
    println!("Initialized mint. Transaction signature: {}", tx);

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
            base_reward: 50_000_000, // 0.05 SOL base reward
            tier1_threshold: 5, // 5 referrals for tier 1
            tier1_reward: 75_000_000, // 0.075 SOL tier 1 reward
            tier2_threshold: 10, // 10 referrals for tier 2
            tier2_reward: 100_000_000, // 0.1 SOL tier 2 reward
            max_reward_cap: 1_000_000_000, // 1 SOL max rewards
            revenue_share_percent: 500, // 5% revenue share
            required_token: None,
            min_token_amount: 0,
            program_end_time: None,
        })
        .signer(&owner)
        .send()
        .expect("Failed to create token referral program");

    println!("Created token referral program. Transaction signature: {}", tx);

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
}
