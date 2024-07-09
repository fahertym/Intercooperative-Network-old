// src/main.rs

use icn_node::blockchain::{Blockchain, Transaction};
use icn_node::currency::CurrencyType;
use icn_node::democracy::{ProposalType, ProposalCategory};
use icn_node::IcnNode;

use chrono::Duration;
use ed25519_dalek::Keypair;
use rand::rngs::OsRng;

fn main() {
    println!("Intercooperative Network (ICN) Project");

    let mut blockchain = Blockchain::new();
    let mut csprng = OsRng{};
    let _keypair: Keypair = Keypair::generate(&mut csprng);

    // Add members and set initial reputations
    blockchain.consensus.add_member("Alice".to_string());
    blockchain.consensus.add_member("Bob".to_string());
    blockchain.consensus.add_member("Charlie".to_string());
    blockchain.consensus.add_member("Dave".to_string());

    // Ensure all members are initially eligible
    blockchain.consensus.update_reputation("Alice", 0.2);
    blockchain.consensus.update_reputation("Bob", 0.2);
    blockchain.consensus.update_reputation("Charlie", 0.2);
    blockchain.consensus.update_reputation("Dave", 0.2);

    // Simulate block creation and voting
    for i in 1..=5 {
        let transactions = vec![
            Transaction::new("System".to_string(), format!("User{}", i % 3 + 1), 100.0, CurrencyType::BasicNeeds),
            Transaction::new(format!("User{}", i % 3 + 1), format!("User{}", (i + 1) % 3 + 1), 50.0, CurrencyType::Education),
        ];

        // Create a block
        match blockchain.create_block(transactions) {
            Ok(_) => println!("Block {} created successfully", i),
            Err(e) => println!("Failed to create block {}: {}", i, e),
        }

        // Now vote on the block
        let voters = ["Alice", "Bob", "Charlie"];
        for (j, voter) in voters.iter().enumerate() {
            if let Err(e) = blockchain.vote_on_block(voter, i, j % 2 == 0) {
                println!("Failed to vote on block {}: {}", i, e);
            }
        }

        blockchain.finalize_block(i);

        if i % 2 == 0 {
            blockchain.maintain_blockchain();
        }
    }

    // Create a proposal
    let proposal_id = match blockchain.create_proposal(
        "New Economic Policy".to_string(),
        "Implement universal basic income".to_string(),
        "Alice".to_string(),
        Duration::days(7),
        ProposalType::EconomicAdjustment,
        ProposalCategory::Economic,
        0.6
    ) {
        Ok(id) => id,
        Err(e) => {
            println!("Failed to create proposal: {}", e);
            return;
        }
    };

    // Vote on the proposal
    if let Err(e) = blockchain.vote_on_proposal(&proposal_id, "Bob", true) {
        println!("Failed to vote on proposal: {}", e);
    }
    if let Err(e) = blockchain.vote_on_proposal(&proposal_id, "Charlie", false) {
        println!("Failed to vote on proposal: {}", e);
    }

    // Execute pending proposals
    let results = blockchain.execute_pending_proposals();
    for result in results {
        if let Err(e) = result {
            println!("Failed to execute proposal: {}", e);
        }
    }

    // Print final blockchain state
    println!("Blockchain length: {}", blockchain.chain.len());
    if let Some(last_block) = blockchain.chain.last() {
        println!("Last block: {:?}", last_block);
    }
    println!("Alice's reputation: {:?}", blockchain.consensus.get_reputation("Alice"));
    println!("Bob's reputation: {:?}", blockchain.consensus.get_reputation("Bob"));
    println!("Charlie's reputation: {:?}", blockchain.consensus.get_reputation("Charlie"));
    println!("Dave's reputation: {:?}", blockchain.consensus.get_reputation("Dave"));

    // Create an ICN Node
    let _icn_node = IcnNode::new();
    println!("ICN Node created successfully");
}