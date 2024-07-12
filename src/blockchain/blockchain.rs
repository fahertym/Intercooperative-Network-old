// Filename: src/blockchain/blockchain.rs

use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use rand::distributions::{Distribution, Uniform};
use rand::thread_rng;

use crate::blockchain::Block;
use crate::blockchain::Transaction;
use crate::smart_contract::{SmartContract, ExecutionEnvironment};
use crate::consensus::Consensus;

// ===============================================
// Blockchain Struct
// ===============================================
// Represents the entire blockchain

#[derive(Clone, Serialize, Deserialize)]
pub struct Blockchain {
    pub chain: Vec<Block>,
    pub pending_transactions: Vec<Transaction>,
    pub smart_contracts: HashMap<String, SmartContract>,
    pub execution_environment: ExecutionEnvironment,
    pub consensus: Consensus,
}

impl Blockchain {
    // Create a new Blockchain with a genesis block
    pub fn new() -> Self {
        let mut blockchain = Blockchain {
            chain: vec![],
            pending_transactions: vec![],
            smart_contracts: HashMap::new(),
            execution_environment: ExecutionEnvironment::new(),
            consensus: Consensus::new(),
        };
        
        // Create and add the genesis block
        let genesis_block = Block::new(0, vec![], String::new());
        blockchain.chain.push(genesis_block);
        
        blockchain
    }

    // Add a new block to the chain
    pub fn create_block(&mut self, _proposer: String) -> Result<(), String> {
        let previous_block = self.chain.last().ok_or("No previous block found")?;
        let new_block = Block::new(self.chain.len() as u64, self.pending_transactions.clone(), previous_block.hash.clone());
        
        // Validate the new block
        self.validate_block(&new_block)?;
        
        // Add the block to the chain
        self.chain.push(new_block);
        
        // Clear pending transactions
        self.pending_transactions.clear();
        
        Ok(())
    }

    // Validate a block
    pub fn validate_block(&self, block: &Block) -> Result<(), String> {
        // Check if the previous hash matches
        if let Some(previous_block) = self.chain.last() {
            if block.previous_hash != previous_block.hash {
                return Err("Invalid previous hash".to_string());
            }
        }

        // Verify each transaction in the block
        for transaction in &block.transactions {
            self.validate_transaction(transaction)?;
        }

        // Check if the block's hash is valid
        if block.hash != block.calculate_hash() {
            return Err("Invalid block hash".to_string());
        }

        Ok(())
    }

    // Validate a transaction
    pub fn validate_transaction(&self, _transaction: &Transaction) -> Result<(), String> {
        // Check if the sender has sufficient balance
        // TODO: Implement balance checking logic
        
        // Verify the transaction signature
        // TODO: Implement signature verification logic

        Ok(())
    }

    // Add a transaction to the pending transactions
    pub fn add_transaction(&mut self, transaction: Transaction) {
        self.pending_transactions.push(transaction);
    }

    // Get the latest block in the chain
    pub fn get_latest_block(&self) -> Option<&Block> {
        self.chain.last()
    }

    // Check if the entire blockchain is valid
    pub fn is_chain_valid(&self) -> bool {
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

    // Store a smart contract
    pub fn store_smart_contract(&mut self, contract: SmartContract) -> Result<(), String> {
        self.smart_contracts.insert(contract.id.clone(), contract);
        Ok(())
    }

    // Get a smart contract by its ID
    pub fn get_smart_contract(&self, id: &str) -> Option<&SmartContract> {
        self.smart_contracts.get(id)
    }

    // Update a smart contract
    pub fn update_smart_contract(&mut self, id: &str, updated_contract: SmartContract) -> Result<(), String> {
        self.smart_contracts.insert(id.to_string(), updated_contract);
        Ok(())
    }

    // Remove a smart contract
    pub fn remove_smart_contract(&mut self, id: &str) -> Result<(), String> {
        self.smart_contracts.remove(id);
        Ok(())
    }

    // Deploy a new smart contract
    pub fn deploy_smart_contract(&mut self, contract: SmartContract) -> Result<(), String> {
        if self.smart_contracts.contains_key(&contract.id) {
            return Err("Smart contract with this ID already exists".to_string());
        }
        self.smart_contracts.insert(contract.id.clone(), contract);
        Ok(())
    }

    // Execute all smart contracts in the latest block
    pub fn execute_smart_contracts(&mut self) -> Result<(), String> {
        let block = self.chain.last_mut().ok_or("No blocks found")?;
        let transactions = block.transactions.clone();
        for transaction in transactions {
            if let Some(ref contract) = transaction.smart_contract {
                let result = contract.execute(&mut self.execution_environment)?;
                block.add_smart_contract_result(contract.id.clone(), result, transaction.gas_limit);
            }
        }
        Ok(())
    }

    // Select a proposer for the next block based on reputation
    pub fn select_proposer(&self) -> Option<String> {
        let total_reputation: f64 = self.consensus.members.values().map(|member| member.reputation).sum();
        let mut rng = thread_rng();
        let selection_point: f64 = Uniform::new(0.0, total_reputation).sample(&mut rng);
        
        let mut cumulative_reputation = 0.0;
        for member in self.consensus.members.values() {
            cumulative_reputation += member.reputation;
            if cumulative_reputation >= selection_point {
                return Some(member.id.clone());
            }
        }

        None
    }
}
