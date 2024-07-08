mod blockchain;
mod consensus;
mod transaction_validator;
mod democracy;
mod tests;

use crate::blockchain::{Blockchain, Transaction};
use crate::consensus::CurrencyType;
use crate::democracy::{ProposalCategory, ProposalType};
use chrono::Duration;
use rand::Rng;

fn main() {
    let mut blockchain = Blockchain::new();
    let mut rng = rand::thread_rng();

    println!("Initializing blockchain...");

    // Add members
    blockchain.consensus.add_member("Alice".to_string());
    blockchain.consensus.add_member("Bob".to_string());
    blockchain.consensus.add_member("Charlie".to_string());
    blockchain.consensus.add_member("Dave".to_string());

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
                let vote = rng.gen_bool(0.7); // 70% chance of voting in favor
                blockchain.vote_on_block(voter, block_index, vote)
                    .unwrap_or_else(|e| println!("Failed to vote: {}", e));
            }

            blockchain.finalize_block(block_index);
            println!("Block {} finalized.", block_index);
        }

        // Create and vote on proposals
        if i % 5 == 0 {
            let proposer = ["Alice", "Bob", "Charlie", "Dave"][i % 4];
            println!("Creating proposal by {}...", proposer);
            match blockchain.create_proposal(
                format!("Proposal {}", i / 5),
                format!("Description for proposal {}", i / 5),
                proposer.to_string(),
                Duration::days(1),
                ProposalType::EconomicAdjustment,
                ProposalCategory::Economic,
                0.6,
            ) {
                Ok(proposal_id) => {
                    println!("Proposal created with ID: {}", proposal_id);
                    
                    // Simulate voting on the proposal
                    for voter in ["Alice", "Bob", "Charlie", "Dave"].iter() {
                        let vote = rng.gen_bool(0.6); // 60% chance of voting in favor
                        match blockchain.vote_on_proposal(&proposal_id, voter, vote) {
                            Ok(_) => println!("{} voted {} on proposal {}", voter, if vote { "in favor" } else { "against" }, proposal_id),
                            Err(e) => println!("{} failed to vote on proposal {}: {}", voter, proposal_id, e),
                        }
                    }
                },
                Err(e) => println!("Failed to create proposal: {}", e),
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
    }

    // Print final blockchain state
    println!("\nFinal Blockchain State:");
    println!("Blockchain length: {}", blockchain.chain.len());
    println!("Last block: {:?}", blockchain.chain.last().unwrap());
    println!("Alice's reputation: {:?}", blockchain.consensus.get_reputation("Alice"));
    println!("Bob's reputation: {:?}", blockchain.consensus.get_reputation("Bob"));
    println!("Charlie's reputation: {:?}", blockchain.consensus.get_reputation("Charlie"));
    println!("Dave's reputation: {:?}", blockchain.consensus.get_reputation("Dave"));
}