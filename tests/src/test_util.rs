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
    },
    Client, Cluster
};
use std::process::Command;

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

pub fn request_airdrop_with_retries(rpc_client: &RpcClient, pubkey: &Pubkey, amount: u64) -> Result<(), String> {
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

pub fn setup() -> (Keypair, Keypair, Keypair, Pubkey, Client<Arc<Keypair>>) {
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
