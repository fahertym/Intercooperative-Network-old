use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use hex::encode;
use std::collections::HashMap;

// =================================================
// Consensus: Proof of Contribution Mechanism
// =================================================

/// `Consensus` struct represents the Proof of Contribution consensus mechanism.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Consensus {
    pub members: Vec<String>,                 // List of consensus members
    pub reputation_scores: HashMap<String, u64>, // Reputation scores of each member
    pub total_contribution: u64,              // Total contribution across all members
    pub vote_threshold: f64,                  // Required percentage of reputation for a valid vote
}

impl Consensus {
    /// Creates a new `Consensus` instance with initial parameters.
    ///
    /// # Arguments
    ///
    /// * `initial_vote_threshold` - The initial vote threshold.
    /// * `minimum_vote_threshold` - The minimum vote threshold.
    ///
    /// # Returns
    ///
    /// * A new `Consensus` instance.
    pub fn new(initial_vote_threshold: f64, minimum_vote_threshold: f64) -> Self {
        Consensus {
            members: Vec::new(),
            reputation_scores: HashMap::new(),
            total_contribution: 0,
            vote_threshold: initial_vote_threshold.max(minimum_vote_threshold), // Ensure threshold isn't too low
        }
    }

    /// Adds a new member to the consensus.
    ///
    /// # Arguments
    ///
    /// * `member` - The member to add.
    pub fn add_member(&mut self, member: String) {
        self.members.push(member.clone());
        self.reputation_scores.insert(member, 0); // Start with 0 reputation
    }

    /// Updates a member's reputation based on their contribution.
    ///
    /// # Arguments
    ///
    /// * `member` - The member whose reputation to update.
    /// * `contribution` - The amount of contribution.
    pub fn update_reputation(&mut self, member: String, contribution: u64) {
        *self.reputation_scores.get_mut(&member).unwrap() += contribution;
        self.total_contribution += contribution;

        // Adjust the vote threshold based on the member's new reputation share
        self.vote_threshold = self.reputation_scores[&member] as f64 / self.total_contribution as f64;
    }
}

// =================================================
// Currency: Types of Currencies in the System
// =================================================

/// `CurrencyType` enum represents the types of currencies in the system.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CurrencyType {
    ICN,             // The native ICN token
    Custom(String),  // Custom tokens with their own names
}

// =================================================
// Transaction: Structure for a Single Transaction
// =================================================

/// `Transaction` struct represents a single transaction in the blockchain.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transaction {
    pub from: String,                // Sender's address
    pub to: String,                  // Recipient's address
    pub amount: f64,                 // Amount of currency being transferred
    pub currency_type: CurrencyType, // Type of currency
    pub gas_limit: u64,              // Maximum gas allowed for smart contract execution
    pub signature: Option<String>,   // Signature to verify authenticity
}

impl Transaction {
    /// Creates a new transaction.
    ///
    /// # Arguments
    ///
    /// * `from` - The sender's address.
    /// * `to` - The recipient's address.
    /// * `amount` - The amount of currency being transferred.
    /// * `currency_type` - The type of currency.
    /// * `gas_limit` - The maximum gas allowed for smart contract execution.
    ///
    /// # Returns
    ///
    /// * A new `Transaction` instance.
    pub fn new(from: String, to: String, amount: f64, currency_type: CurrencyType, gas_limit: u64) -> Self {
        Transaction {
            from,
            to,
            amount,
            currency_type,
            gas_limit,
            signature: None,
        }
    }
}

// =================================================
// Block: Structure for a Block in the Blockchain
// =================================================

/// `Block` struct represents a block in the blockchain.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Block {
    pub index: u64,                       // Index (height) of the block in the chain
    pub timestamp: DateTime<Utc>,         // Timestamp of block creation
    pub transactions: Vec<Transaction>,   // List of transactions included in the block
    pub previous_hash: String,            // Hash of the previous block
    pub hash: String,                     // Hash of this block (calculated based on its contents)
    pub nonce: u64,                       // A nonce used in the Proof-of-Work process (if applicable)
    pub gas_used: u64,                    // Total gas used by smart contract executions in this block
}

impl Block {
    /// Creates a new block.
    ///
    /// # Arguments
    ///
    /// * `index` - The index of the block in the blockchain.
    /// * `transactions` - The list of transactions to include in the block.
    /// * `previous_hash` - The hash of the previous block.
    ///
    /// # Returns
    ///
    /// * A new `Block` instance.
    pub fn new(index: u64, transactions: Vec<Transaction>, previous_hash: String) -> Self {
        let mut block = Block {
            index,
            timestamp: Utc::now(),
            transactions,
            previous_hash,
            hash: String::new(), // Will be calculated later
            nonce: 0,
            gas_used: 0,
        };
        block.hash = block.calculate_hash(); // Calculate and set the block's hash
        block
    }

    /// Calculates the hash of the block using SHA256.
    ///
    /// # Returns
    ///
    /// * The hash of the block as a string.
    fn calculate_hash(&self) -> String {
        let mut hasher = Sha256::new();
        let block_data = serde_json::to_string(self).unwrap();
        hasher.update(block_data.as_bytes());
        encode(hasher.finalize())
    }
}

// =================================================
// Blockchain: The Core Data Structure
// =================================================

/// `Blockchain` struct represents the core data structure of the blockchain.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Blockchain {
    pub chain: Vec<Block>,                          // The chain of blocks
    pub pending_transactions: Vec<Transaction>,     // Transactions not yet included in a block
    pub consensus: Consensus,                       // Consensus mechanism instance
}

impl Blockchain {
    /// Creates a new blockchain with a genesis block.
    ///
    /// # Returns
    ///
    /// * A new `Blockchain` instance.
    pub fn new() -> Self {
        let genesis_block = Block::new(0, Vec::new(), String::from("0"));
        Blockchain {
            chain: vec![genesis_block],
            pending_transactions: Vec::new(),
            consensus: Consensus::new(0.5, 0.1),
        }
    }

    /// Adds a transaction to the list of pending transactions.
    ///
    /// # Arguments
    ///
    /// * `transaction` - The transaction to add.
    pub fn add_transaction(&mut self, transaction: Transaction) {
        self.pending_transactions.push(transaction);
    }

    /// Creates a new block and adds it to the blockchain.
    ///
    /// # Arguments
    ///
    /// * `miner_address` - The address of the miner who mined the block.
    ///
    /// # Returns
    ///
    /// * A result indicating success or failure.
    pub fn create_block(&mut self, miner_address: String) -> Result<(), String> {
        let previous_hash = self.chain.last().unwrap().hash.clone();
        let mut block = Block::new(self.chain.len() as u64, self.pending_transactions.clone(), previous_hash);

        // Simulate mining by incrementing the nonce until a hash with leading zeros is found
        while !block.hash.starts_with("0000") {
            block.nonce += 1;
            block.hash = block.calculate_hash();
        }

        self.chain.push(block);
        self.pending_transactions = Vec::new();

        // Reward the miner with a new transaction
        let reward_transaction = Transaction::new(
            String::from("0"), // The blockchain itself is the sender
            miner_address,
            1.0, // Reward amount (this can be adjusted)
            CurrencyType::ICN,
            0,
        );
        self.pending_transactions.push(reward_transaction);

        Ok(())
    }

    /// Gets the latest block in the blockchain.
    ///
    /// # Returns
    ///
    /// * An option containing a reference to the latest block.
    pub fn get_latest_block(&self) -> Option<&Block> {
        self.chain.last()
    }

    /// Validates the blockchain.
    ///
    /// # Returns
    ///
    /// * A boolean indicating whether the blockchain is valid.
    pub fn is_valid(&self) -> bool {
        for i in 1..self.chain.len() {
            let current_block = &self.chain[i];
            let previous_block = &self.chain[i - 1];

            if current_block.hash != current_block.calculate_hash() {
                return false;
            }

            if current_block.previous_hash != previous_block.hash {
                return false;
            }
        }
        true
    }
}

// =================================================
// Tests for Blockchain Functionality
// =================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_blockchain_creation() {
        let blockchain = Blockchain::new();

        // The blockchain should start with one genesis block
        assert_eq!(blockchain.chain.len(), 1);

        // The genesis block should have index 0 and no transactions
        let genesis_block = &blockchain.chain[0];
        assert_eq!(genesis_block.index, 0);
        assert_eq!(genesis_block.transactions.len(), 0);
    }

    #[test]
    fn test_add_transaction() {
        let mut blockchain = Blockchain::new();

        let tx = Transaction::new(
            String::from("Alice"),
            String::from("Bob"),
            10.0,
            CurrencyType::ICN,
            0,
        );

        blockchain.add_transaction(tx.clone());
        assert_eq!(blockchain.pending_transactions.len(), 1);
        assert_eq!(blockchain.pending_transactions[0], tx);
    }

    #[test]
    fn test_create_block() {
        let mut blockchain = Blockchain::new();

        let tx = Transaction::new(
            String::from("Alice"),
            String::from("Bob"),
            10.0,
            CurrencyType::ICN,
            0,
        );

        blockchain.add_transaction(tx);
        let miner_address = String::from("Miner1");
        blockchain.create_block(miner_address.clone()).unwrap();

        assert_eq!(blockchain.chain.len(), 2);
        assert_eq!(blockchain.chain[1].transactions.len(), 1);
        assert_eq!(blockchain.pending_transactions.len(), 1);
        assert_eq!(blockchain.pending_transactions[0].to, miner_address);
    }

    #[test]
    fn test_is_valid() {
        let mut blockchain = Blockchain::new();

        let tx = Transaction::new(
            String::from("Alice"),
            String::from("Bob"),
            10.0,
            CurrencyType::ICN,
            0,
        );

        blockchain.add_transaction(tx);
        blockchain.create_block(String::from("Miner1")).unwrap();

        assert!(blockchain.is_valid());

        // Tamper with the blockchain
        blockchain.chain[1].transactions[0].amount = 100.0;

        assert!(!blockchain.is_valid());
    }
}
