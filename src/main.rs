// src/main.rs

mod blockchain;
mod consensus;

use crate::blockchain::Blockchain;
use crate::consensus::CurrencyType;

fn main() {
    let mut blockchain = Blockchain::new();

    // Add members
    blockchain.consensus.add_member("Alice".to_string());
    blockchain.consensus.add_member("Bob".to_string());
    blockchain.consensus.add_member("Charlie".to_string());
    blockchain.consensus.add_member("Dave".to_string());

    // Simulate block creation and voting
    for i in 1..=5 {
        let transactions = vec![
            ("System".to_string(), format!("User{}", i % 3 + 1), 100.0, CurrencyType::BasicNeeds),
            (format!("User{}", i % 3 + 1), format!("User{}", (i + 1) % 3 + 1), 50.0, CurrencyType::Education),
        ];

        // Create a block
        blockchain.create_block(transactions).expect("Failed to create block");

        // Now vote on the block
        // Note: We vote on the latest block, which is at index 1 (0 is genesis)
        blockchain.vote_on_block("Alice", 1, true).expect("Failed to vote on block");
        blockchain.vote_on_block("Bob", 1, i % 2 == 0).expect("Failed to vote on block");
        blockchain.vote_on_block("Charlie", 1, i % 3 == 0).expect("Failed to vote on block");

        blockchain.finalize_block(1);

        if i % 5 == 0 {
            blockchain.maintain_blockchain();
        }
    }

    // Print final blockchain state
    println!("Blockchain length: {}", blockchain.chain.len());
    println!("Last block: {:?}", blockchain.chain.last().unwrap());
}