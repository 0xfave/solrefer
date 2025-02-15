use anchor_client::{
    anchor_lang::system_program,
    solana_client::rpc_client::RpcClient,
    solana_sdk::{
        commitment_config::CommitmentConfig,
        native_token::LAMPORTS_PER_SOL,
        pubkey::Pubkey,
        signature::{read_keypair_file, Keypair},
        signer::Signer,
        system_instruction,
    },
    Client, Cluster,
};
use anchor_spl::token::spl_token;
use solrefer::{accounts, instruction};
use std::process::Command;
use std::str::FromStr;
use std::sync::Arc;

pub fn ensure_test_validator() -> RpcClient {
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

pub fn request_airdrop_with_retries(
    rpc_client: &RpcClient,
    pubkey: &Pubkey,
    amount: u64,
) -> Result<(), String> {
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
                            println!(
                                "Successfully airdropped {} SOL",
                                amount as f64 / LAMPORTS_PER_SOL as f64
                            );
                            return Ok(());
                        }
                    }
                }
            }
            Err(e) => println!("Airdrop failed: {}", e),
        }
        current_try += 1;
        if current_try < max_retries {
            println!(
                "Retrying airdrop... (attempt {}/{})",
                current_try + 1,
                max_retries
            );
            std::thread::sleep(std::time::Duration::from_secs(2));
        }
    }
    Err(format!("Failed to airdrop after {} attempts", max_retries))
}

pub fn setup() -> (Keypair, Keypair, Keypair, Pubkey, Client<Arc<Keypair>>) {
    let program_id = "DvdCTkZBHpUpPYAccKkN3DQtu69GCEre3gsPJ7r33W35"; // Your program ID
    let anchor_wallet = std::env::var("ANCHOR_WALLET").unwrap();
    let payer = Arc::new(read_keypair_file(&anchor_wallet).unwrap());

    let client = Client::new_with_options(
        Cluster::Localnet,
        payer.clone(),
        CommitmentConfig::confirmed(),
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

pub fn create_mint(owner: &Keypair, client: &Client<Arc<Keypair>>, program_id: Pubkey) -> Keypair {
    // Create new token mint
    let mint = Keypair::new();
    let mint_authority = &owner;

    // Create mint account
    let rpc_client = client.program(program_id).unwrap().rpc();
    let rent = rpc_client
        .get_minimum_balance_for_rent_exemption(82)
        .unwrap();
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
    )
    .unwrap();

    let tx = client
        .program(program_id)
        .unwrap()
        .request()
        .instruction(ix)
        .send()
        .expect("Failed to initialize mint");
    println!("Initialized mint. Transaction signature: {}", tx);

    mint
}

pub fn create_token_account(
    owner: &Keypair,
    mint: &Pubkey,
    client: &Client<Arc<Keypair>>,
    program_id: Pubkey,
) -> Pubkey {
    let rpc_client = client.program(program_id).unwrap().rpc();

    // Create token account
    let account = Keypair::new();
    let rent = rpc_client
        .get_minimum_balance_for_rent_exemption(165)
        .unwrap();

    let create_account_ix = system_instruction::create_account(
        &owner.pubkey(),
        &account.pubkey(),
        rent,
        165,
        &spl_token::id(),
    );

    let init_account_ix = spl_token::instruction::initialize_account(
        &spl_token::id(),
        &account.pubkey(),
        mint,
        &owner.pubkey(),
    )
    .unwrap();

    let tx = client
        .program(program_id)
        .unwrap()
        .request()
        .instruction(create_account_ix)
        .instruction(init_account_ix)
        .signer(&owner)
        .signer(&account)
        .send()
        .expect("Failed to create token account");
    println!("Created token account. Transaction signature: {}", tx);

    account.pubkey()
}

pub fn mint_tokens(
    mint: &Keypair,
    token_account: &Pubkey,
    owner: &Keypair,
    amount: u64,
    client: &Client<Arc<Keypair>>,
    program_id: Pubkey,
) {
    let ix = spl_token::instruction::mint_to(
        &spl_token::id(),
        &mint.pubkey(),
        token_account,
        &owner.pubkey(),
        &[&owner.pubkey()],
        amount,
    )
    .unwrap();

    let tx = client
        .program(program_id)
        .unwrap()
        .request()
        .instruction(ix)
        .signer(&owner)
        .send()
        .expect("Failed to mint tokens");
    println!("Minted tokens. Transaction signature: {}", tx);
}

/// Deposits SOL into a referral program
pub fn deposit_sol(
    amount: u64,
    referral_program_pubkey: Pubkey,
    authority: &Keypair,
    client: &Client<Arc<Keypair>>,
    program_id: Pubkey,
    vault: Pubkey,
) -> String {
    let tx = client
        .program(program_id)
        .unwrap()
        .request()
        .accounts(accounts::DepositSol {
            referral_program: referral_program_pubkey,
            vault: vault,
            authority: authority.pubkey(),
            system_program: system_program::ID,
        })
        .args(instruction::DepositSol { amount })
        .signer(authority)
        .send()
        .expect("Failed to deposit SOL");

    println!(
        "Deposited {} SOL. Transaction signature: {}",
        amount as f64 / LAMPORTS_PER_SOL as f64,
        tx
    );
    tx.to_string()
}

/// Deposits tokens into a referral program
pub fn deposit_tokens(
    amount: u64,
    referral_program_pubkey: Pubkey,
    token_vault: Pubkey,
    token_mint: Pubkey,
    depositor_token_account: Pubkey,
    authority: &Keypair,
    client: &Client<Arc<Keypair>>,
    program_id: Pubkey,
) -> String {
    let tx = client
        .program(program_id)
        .unwrap()
        .request()
        .accounts(accounts::DepositToken {
            referral_program: referral_program_pubkey,
            token_vault,
            token_mint,
            depositor_token_account,
            authority: authority.pubkey(),
            token_program: spl_token::id(),
        })
        .args(instruction::DepositToken { amount })
        .signer(authority)
        .send()
        .expect("Failed to deposit tokens");

    println!("Deposited {} tokens. Transaction signature: {}", amount, tx);
    tx.to_string()
}

// Helper function to create a SOL referral program for tests
pub fn create_sol_referral_program(
    owner: &Keypair,
    client: &Client<Arc<Keypair>>,
    program_id: Pubkey,
    fixed_reward_amount: u64,
    locked_period: i64,
    early_redemption_fee: u64,
    mint_fee: u64,
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
) -> (Pubkey, Pubkey) {
    // Find the PDA for referral program
    let (referral_program, _) =
        Pubkey::find_program_address(&[b"referral_program", owner.pubkey().as_ref()], &program_id);

    let (vault, _) =
        Pubkey::find_program_address(&[b"vault", referral_program.as_ref()], &program_id);

    let tx = client
        .program(program_id)
        .unwrap()
        .request()
        .accounts(solrefer::accounts::CreateReferralProgram {
            referral_program,
            eligibility_criteria: get_eligibility_criteria_pda(referral_program, program_id),
            authority: owner.pubkey(),
            token_mint_info: None,
            token_program: None,
            system_program: system_program::ID,
        })
        .args(solrefer::instruction::CreateReferralProgram {
            token_mint: None,
            fixed_reward_amount,
            locked_period,
            early_redemption_fee,
            mint_fee,
            base_reward,
            tier1_threshold,
            tier1_reward,
            tier2_threshold,
            tier2_reward,
            max_reward_cap,
            revenue_share_percent,
            required_token,
            min_token_amount,
            program_end_time,
        })
        .signer(&owner)
        .send()
        .expect("Failed to create SOL referral program");

    println!(
        "Created SOL referral program. Transaction signature: {}",
        tx
    );
    (referral_program, vault)
}

// Helper function to get eligibility criteria PDA
pub fn get_eligibility_criteria_pda(referral_program: Pubkey, program_id: Pubkey) -> Pubkey {
    let (pda, _) = Pubkey::find_program_address(
        &[b"eligibility_criteria", referral_program.as_ref()],
        &program_id,
    );
    pda
}
