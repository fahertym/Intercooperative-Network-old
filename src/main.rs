mod blockchain;
mod consensus;
mod tests;

use crate::blockchain::Blockchain;

fn main() {
    let mut blockchain = Blockchain::new();

    // Add members
    blockchain.consensus.add_member("Alice".to_string());
    blockchain.consensus.add_member("Bob".to_string());
    blockchain.consensus.add_member("Charlie".to_string());
    blockchain.consensus.add_member("Dave".to_string());

    // Simulate block creation and voting
    for i in 1..=20 {
        blockchain.propose_block(format!("Transaction {}", i)).unwrap();
        blockchain.vote_on_block(i, "Alice".to_string(), true).unwrap();
        blockchain.vote_on_block(i, "Bob".to_string(), true).unwrap();
        blockchain.vote_on_block(i, "Charlie".to_string(), i % 2 == 0).unwrap(); // Charlie votes against every other block
        blockchain.vote_on_block(i, "Dave".to_string(), i % 3 == 0).unwrap(); // Dave votes for every third block
        blockchain.finalize_blocks();
        
        // Simulate maintenance every 5 blocks
        if i % 5 == 0 {
            blockchain.maintain_blockchain();
        }
    }

    // Simulate a malicious proposal and slashing
    blockchain.propose_block("malicious transaction".to_string()).unwrap();
    blockchain.check_for_slashing();

    // Simulate a slashing challenge
    blockchain.consensus.challenge_slashing("Charlie", 3); // 3 votes to challenge (success)
    blockchain.consensus.challenge_slashing("Dave", 1); // 1 vote to challenge (failure)

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