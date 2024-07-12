use crate::blockchain::Transaction;
use serde::{Serialize, Deserialize};
use std::collections::HashMap;

// Struct representing a block in the blockchain
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Block {
    pub index: u64,                                // Index of the block in the chain
    pub timestamp: i64,                            // Timestamp when the block was created
    pub transactions: Vec<Transaction>,            // List of transactions in the block
    pub previous_hash: String,                     // Hash of the previous block in the chain
    pub hash: String,                              // Hash of the current block
    pub nonce: u64,                                // Nonce used for mining the block
    pub gas_used: u64,                             // Total gas used by smart contracts in the block
    pub smart_contract_results: HashMap<String, String>, // Results of smart contract executions
}

impl Block {
    // Create a new block
    pub fn new(index: u64, transactions: Vec<Transaction>, previous_hash: String) -> Self {
        let timestamp = chrono::Utc::now().timestamp();
        let mut block = Block {
            index,
            timestamp,
            transactions,
            previous_hash,
            hash: String::new(),
            nonce: 0,
            gas_used: 0,
            smart_contract_results: HashMap::new(),
        };
        block.hash = block.calculate_hash();
        block
    }

    // Calculate the hash of the block
    pub fn calculate_hash(&self) -> String {
        // Implement hash calculation logic
        "dummy_hash".to_string()
    }

    // Add the result of a smart contract execution to the block
    pub fn add_smart_contract_result(&mut self, contract_id: String, result: String, gas_used: u64) {
        self.smart_contract_results.insert(contract_id, result);
        self.gas_used += gas_used;
    }
}
