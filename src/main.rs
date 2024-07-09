// src/main.rs

mod blockchain;
mod consensus;
mod transaction_validator;
mod democracy;
mod did;
mod currency;

use crate::blockchain::{Blockchain, Transaction};
use crate::consensus::CurrencyType;
use crate::democracy::{ProposalCategory, ProposalType};
use crate::did::{DecentralizedIdentity, DidManager};
use crate::currency::{CurrencySystem, Wallet};
use chrono::Duration;
use rand::Rng;
use std::collections::HashMap;

fn main() {
    let mut blockchain = Blockchain::new();
    let mut rng = rand::thread_rng();

    println!("Initializing blockchain...");

    // Initialize the DiD Manager
    let mut did_manager = DidManager::new();

    // Create some example DiDs
    println!("Creating Decentralized Identities...");
    for name in &["Alice", "Bob", "Charlie", "Dave"] {
        let mut attributes = HashMap::new();
        attributes.insert("name".to_string(), name.to_string());
        let (did, _) = DecentralizedIdentity::new(attributes);
        did_manager.register_did(did).expect("Failed to register DiD");
        println!("Created DiD for {}: {}", name, did.id);
    }

    // Initialize the Currency System
    let mut currency_system = CurrencySystem::new();

    // Create a custom currency
    currency_system.create_custom_currency("CoopCoin".to_string(), 100_000.0, 0.005)
        .expect("Failed to create custom currency");

    // Initialize wallets for each DiD
    let mut wallets = HashMap::new();
    for name in &["Alice", "Bob", "Charlie", "Dave"] {
        if let Some(did) = did_manager.get_did(&format!("did:icn:{}", name.to_lowercase())) {
            let mut wallet = Wallet::new();
            wallet.deposit(CurrencyType::BasicNeeds, 1000.0);
            wallet.deposit(CurrencyType::Education, 500.0);
            wallet.deposit(CurrencyType::Custom("CoopCoin".to_string()), 100.0);
            wallets.insert(did.id.clone(), wallet);
        }
    }

    // Add members to the consensus mechanism
    for name in &["Alice", "Bob", "Charlie", "Dave"] {
        if let Some(did) = did_manager.get_did(&format!("did:icn:{}", name.to_lowercase())) {
            blockchain.consensus.add_member(did.id.clone());
        }
    }

    println!("Members added to the consensus mechanism.");

    let mut dynamic_threshold = 0.66f64;

    // Simulate block creation, voting, and governance
    for i in 1..=30 {
        println!("\n--- Iteration {} ---", i);

        // Create and process regular transactions
        if i % 3 == 0 {
            println!("Creating block {}...", i / 3);
            let transactions = vec![
                Transaction::new("System".to_string(), format!("User{}", i % 3 + 1), 100.0, CurrencyType::BasicNeeds),
                Transaction::new(format!("User{}", i % 3 + 1), format!("User{}", (i + 1) % 3 + 1), 50.0, CurrencyType::Education),
            ];

            blockchain.create_block(transactions).expect("Failed to create block");

            let block_index = blockchain.chain.len() + blockchain.pending_blocks.len();
            
            println!("Voting on block {}...", block_index);
            let voters = ["Alice", "Bob", "Charlie", "Dave"];
            for voter in voters.iter() {
                if let Some(did) = did_manager.get_did(&format!("did:icn:{}", voter.to_lowercase())) {
                    let vote = rng.gen_bool(0.7); // 70% chance of voting in favor
                    blockchain.vote_on_block(&did.id, block_index, vote)
                        .unwrap_or_else(|e| println!("Failed to vote: {}", e));
                    
                    // Update DiD reputation based on participation
                    did_manager.update_reputation(&did.id, 0.1)
                        .unwrap_or_else(|e| println!("Failed to update reputation: {}", e));

                    // Add currency transaction for voting
                    if let Some(voter_wallet) = wallets.get_mut(&did.id) {
                        voter_wallet.deposit(CurrencyType::Volunteer, 1.0);
                        currency_system.get_currency_mut(&CurrencyType::Volunteer)
                            .unwrap()
                            .mint(1.0);
                    }
                }
            }

            blockchain.finalize_block(block_index);
            println!("Block {} finalized.", block_index);

            // Add currency transaction for block creation
            if let Some(block) = blockchain.chain.last() {
                if let Some(proposer_wallet) = wallets.get_mut(&block.proposer) {
                    proposer_wallet.deposit(CurrencyType::BasicNeeds, 10.0);
                    currency_system.get_currency_mut(&CurrencyType::BasicNeeds)
                        .unwrap()
                        .mint(10.0);
                }
            }
        }

        // Create and vote on proposals
        if i % 5 == 0 {
            let proposer = ["Alice", "Bob", "Charlie", "Dave"][i % 4];
            println!("Creating proposal by {}...", proposer);
            if let Some(did) = did_manager.get_did(&format!("did:icn:{}", proposer.to_lowercase())) {
                match blockchain.create_proposal(
                    format!("Proposal {}", i / 5),
                    format!("Description for proposal {}", i / 5),
                    did.id.clone(),
                    Duration::days(1),
                    ProposalType::EconomicAdjustment,
                    ProposalCategory::Economic,
                    0.6,
                ) {
                    Ok(proposal_id) => {
                        println!("Proposal created with ID: {}", proposal_id);
                        
                        // Simulate voting on the proposal
                        for voter in ["Alice", "Bob", "Charlie", "Dave"].iter() {
                            if let Some(voter_did) = did_manager.get_did(&format!("did:icn:{}", voter.to_lowercase())) {
                                let vote = rng.gen_bool(0.6); // 60% chance of voting in favor
                                match blockchain.vote_on_proposal(&proposal_id, &voter_did.id, vote) {
                                    Ok(_) => println!("{} voted {} on proposal {}", voter, if vote { "in favor" } else { "against" }, proposal_id),
                                    Err(e) => println!("{} failed to vote on proposal {}: {}", voter, proposal_id, e),
                                }
                            }
                        }
                    },
                    Err(e) => println!("Failed to create proposal: {}", e),
                }
            }
        }

        // Execute pending proposals
        if i % 10 == 0 {
            println!("Executing pending proposals...");
            let results = blockchain.execute_pending_proposals();
            for (index, result) in results.iter().enumerate() {
                match result {
                    Ok(_) => println!("Proposal {} executed successfully", index),
                    Err(e) => println!("Failed to execute proposal {}: {}", index, e),
                }
            }
        }

        blockchain.maintain_blockchain();
        
        // Update dynamic threshold (simplified for demonstration)
        dynamic_threshold = ((dynamic_threshold * 0.95 + 0.66 * 0.05) as f64).max(0.51).min(0.8);
        blockchain.consensus.set_vote_threshold(dynamic_threshold);

        // Perform adaptive issuance
        if i % 5 == 0 {
            currency_system.adaptive_issuance();
        }
    }

    // Print final blockchain state
    println!("\nFinal Blockchain State:");
    println!("Blockchain length: {}", blockchain.chain.len());
    println!("Last block: {:?}", blockchain.chain.last().unwrap());

    // Print final DiD reputations
    println!("\nFinal DiD Reputations:");
    for name in &["Alice", "Bob", "Charlie", "Dave"] {
        if let Some(did) = did_manager.get_did(&format!("did:icn:{}", name.to_lowercase())) {
            println!("{}'s DiD reputation: {:.2}", name, did.reputation);
        }
    }

    // Print final currency system state
    println!("\nFinal Currency System State:");
    currency_system.print_currency_supplies();

    // Print final wallet states
    println!("\nFinal Wallet States:");
    for (did_id, wallet) in &wallets {
        println!("Wallet for DiD {}:", did_id);
        wallet.print_balances();
    }
}