mod blockchain;
mod consensus;
mod transaction_validator;

use crate::blockchain::{Blockchain, Transaction};
use crate::consensus::CurrencyType;
use ed25519_dalek::Keypair;
use rand::rngs::OsRng;

fn main() {
    let mut blockchain = Blockchain::new();
    let mut csprng = OsRng{};
    let keypair: Keypair = Keypair::generate(&mut csprng);

    // Add members
    blockchain.consensus.add_member("Alice".to_string());
    blockchain.consensus.add_member("Bob".to_string());
    blockchain.consensus.add_member("Charlie".to_string());
    blockchain.consensus.add_member("Dave".to_string());

    // Simulate block creation and voting
    for i in 1..=20 {
        let transactions = vec![
            Transaction::new("System".to_string(), format!("User{}", i % 3 + 1), 100.0, CurrencyType::BasicNeeds),
            Transaction::new(format!("User{}", i % 3 + 1), format!("User{}", (i + 1) % 3 + 1), 50.0, CurrencyType::Education),
        ];

        // Create a block
        blockchain.create_block(transactions, &keypair.public).expect("Failed to create block");

        // Now vote on the block
        blockchain.vote_on_block("Alice", i as u64, true).expect("Failed to vote on block");
        blockchain.vote_on_block("Bob", i as u64, i % 2 == 0).expect("Failed to vote on block");
        blockchain.vote_on_block("Charlie", i as u64, i % 3 == 0).expect("Failed to vote on block");

        blockchain.finalize_block(i as u64);

        if i % 5 == 0 {
            blockchain.maintain_blockchain();
        }
    }  

    // Final maintenance
    blockchain.maintain_blockchain();

    // Print final blockchain state
    println!("Blockchain length: {}", blockchain.chain.len());
    println!("Last block: {:?}", blockchain.chain.last().unwrap());
    println!("Alice's reputation: {:?}", blockchain.consensus.get_reputation("Alice"));
    println!("Bob's reputation: {:?}", blockchain.consensus.get_reputation("Bob"));
    println!("Charlie's reputation: {:?}", blockchain.consensus.get_reputation("Charlie"));
    println!("Dave's reputation: {:?}", blockchain.consensus.get_reputation("Dave"));
}