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
    for i in 1..=20 {
        let transactions = vec![
            ("System".to_string(), format!("User{}", i % 3 + 1), 100.0, CurrencyType::BasicNeeds),
            (format!("User{}", i % 3 + 1), format!("User{}", (i + 1) % 3 + 1), 50.0, CurrencyType::Education),
        ];

        // Create a block
        blockchain.create_block(transactions).expect("Failed to create block");

        // Now vote on the block
        blockchain.vote_on_block("Alice", i, true).expect("Failed to vote on block");
        blockchain.vote_on_block("Bob", i, i % 2 == 0).expect("Failed to vote on block");
        blockchain.vote_on_block("Charlie", i, i % 3 == 0).expect("Failed to vote on block");

        blockchain.finalize_block(i);

        if i % 5 == 0 {
            blockchain.maintain_blockchain();
        }
    }  

    // Simulate a malicious proposal and slashing
    blockchain.propose_block("malicious transaction".to_string()).unwrap();
    blockchain.check_for_slashing();

    // Simulate a slashing challenge
    let charlie_challenge = blockchain.consensus.challenge_slashing("Charlie", 3);
    println!("Charlie's challenge result: {}", charlie_challenge);
    let dave_challenge = blockchain.consensus.challenge_slashing("Dave", 1);
    println!("Dave's challenge result: {}", dave_challenge);

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