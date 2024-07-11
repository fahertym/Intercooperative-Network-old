// ===============================================
// Blockchain Core Implementation
// ===============================================
// This file contains the core structures and functions for our blockchain.
// It defines the Block and Blockchain structures, as well as methods for
// creating new blocks, validating the chain, and managing transactions.
//
// Key concepts:
// - Block: A container for transactions, linked together to form the blockchain
// - Transaction: A record of value transfer or smart contract interaction
// - Blockchain: The complete ledger of all blocks and transactions
// - Consensus: The mechanism for agreeing on the state of the blockchain

// ===============================================
// Imports
// ===============================================

use std::collections::HashMap;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use ed25519_dalek::{Keypair, PublicKey, Signature, Signer, Verifier};
use crate::smart_contract::{SmartContract, ExecutionEnvironment};
use crate::consensus::Consensus;
use crate::currency::CurrencyType;
use sha2::{Sha256, Digest};

// ===============================================
// Constants
// ===============================================

const MAX_GAS_PER_BLOCK: u64 = 1_000_000; // Maximum gas allowed per block

// ===============================================
// Transaction Struct
// ===============================================
// Represents a single transaction in the blockchain

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transaction {
    pub from: String,                // Sender's address (public key or DID)
    pub to: String,                  // Recipient's address
    pub amount: f64,                 // Amount of currency being transferred
    pub currency_type: CurrencyType, // Type of currency (e.g., BasicNeeds, Education)
    pub gas_limit: u64,              // Maximum gas allowed for smart contract execution
    pub smart_contract: Option<SmartContract>, // Optional smart contract to execute
    pub signature: Option<String>,   // Digital signature to verify authenticity
}

impl Transaction {
    // Create a new Transaction
    pub fn new(from: String, to: String, amount: f64, currency_type: CurrencyType, gas_limit: u64) -> Self {
        Transaction {
            from,
            to,
            amount,
            currency_type,
            gas_limit,
            smart_contract: None,
            signature: None,
        }
    }

    // Attach a smart contract to the transaction
    pub fn with_smart_contract(mut self, smart_contract: SmartContract) -> Self {
        self.smart_contract = Some(smart_contract);
        self
    }

    // Sign the transaction with the given keypair
    pub fn sign(&mut self, keypair: &Keypair) -> Result<(), String> {
        let message = self.to_bytes();
        let signature = keypair.sign(&message);
        self.signature = Some(hex::encode(signature.to_bytes()));
        Ok(())
    }

    // Verify the transaction's signature
    pub fn verify(&self, public_key: &PublicKey) -> Result<bool, String> {
        let message = self.to_bytes();
        let signature_bytes = hex::decode(self.signature.as_ref().ok_or("No signature present")?).map_err(|e| e.to_string())?;
        let signature = Signature::from_bytes(&signature_bytes).map_err(|e| e.to_string())?;
        Ok(public_key.verify(&message, &signature).is_ok())
    }

    // Convert the transaction to bytes for signing/verification
    fn to_bytes(&self) -> Vec<u8> {
        // In a real implementation, this would properly serialize all fields
        format!("{}{}{}{:?}{}", self.from, self.to, self.amount, self.currency_type, self.gas_limit).into_bytes()
    }
}

// ===============================================
// Block Struct
// ===============================================
// Represents a single block in the blockchain

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Block {
    pub index: u64,                     // Height of the block in the chain
    pub timestamp: DateTime<Utc>,       // Time the block was created
    pub transactions: Vec<Transaction>, // List of transactions in this block
    pub previous_hash: String,          // Hash of the previous block
    pub hash: String,                   // Hash of this block
    pub nonce: u64,                     // Nonce used for consensus mechanism
    pub gas_used: u64,                  // Total gas used by transactions in this block
    pub smart_contract_results: HashMap<String, Result<(), String>>, // Results of smart contract executions
}

impl Block {
    // Create a new Block
    pub fn new(index: u64, transactions: Vec<Transaction>, previous_hash: String) -> Self {
        let mut block = Block {
            index,
            timestamp: Utc::now(),
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
    fn calculate_hash(&self) -> String {
        let mut hasher = Sha256::new();
        hasher.update(format!("{}{:?}{}{}{}", self.index, self.timestamp, self.previous_hash, self.nonce, self.transactions.len()));
        format!("{:x}", hasher.finalize())
    }

    // Add the result of a smart contract execution to the block
    pub fn add_smart_contract_result(&mut self, contract_id: String, result: Result<(), String>, gas_used: u64) {
        self.smart_contract_results.insert(contract_id, result);
        self.gas_used += gas_used;
    }
}

// ===============================================
// Blockchain Struct
// ===============================================
// Represents the entire blockchain

#[derive(Debug, Clone, Serialize, Deserialize)]
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
    pub fn create_block(&mut self, proposer: String) -> Result<(), String> {
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
    pub fn validate_transaction(&self, transaction: &Transaction) -> Result<(), String> {
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
                let result = contract.execute(&mut self.execution_environment);
                block.add_smart_contract_result(contract.id.clone(), result, transaction.gas_limit);
            }
        }
        Ok(())
    }
}

// ===============================================
// Helper Functions
// ===============================================

// Function to calculate the hash of a given string
fn calculate_hash(data: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(data);
    format!("{:x}", hasher.finalize())
}

// ===============================================
// Tests
// ===============================================
// Unit tests to verify the functionality of our blockchain implementation

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_blockchain_creation() {
        let blockchain = Blockchain::new();
        assert_eq!(blockchain.chain.len(), 1);
        assert_eq!(blockchain.chain[0].index, 0);
    }

    #[test]
    fn test_add_block() {
        let mut blockchain = Blockchain::new();
        let transaction = Transaction::new(
            "Alice".to_string(),
            "Bob".to_string(),
            100.0,
            CurrencyType::BasicNeeds,
            1000,
        );
        blockchain.add_transaction(transaction);
        assert!(blockchain.create_block("Node1".to_string()).is_ok());
        assert_eq!(blockchain.chain.len(), 2);
    }

    #[test]
    fn test_blockchain_validity() {
        let mut blockchain = Blockchain::new();
        let transaction = Transaction::new(
            "Alice".to_string(),
            "Bob".to_string(),
            100.0,
            CurrencyType::BasicNeeds,
            1000,
        );
        blockchain.add_transaction(transaction);
        assert!(blockchain.create_block("Node1".to_string()).is_ok());
        assert!(blockchain.is_chain_valid());
    }
}

// ===============================================
// End of File
// ===============================================
// This concludes the implementation of our blockchain core. It provides
// the fundamental structures and functions needed for a basic blockchain
// system, including blocks, transactions, and smart contract integration.
// Filename: src/blockchain/mod.rs

// Declare modules for the blockchain directory
pub mod blockchain;
