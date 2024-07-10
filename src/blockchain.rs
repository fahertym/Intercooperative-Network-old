use crate::consensus::PoCConsensus;
use crate::currency::CurrencyType;
use crate::smart_contract::{SmartContract, ExecutionEnvironment};
use chrono::{DateTime, Utc};
use ed25519_dalek::{Keypair, PublicKey, Signature, Signer, Verifier};
use serde::{Serialize, Deserialize};
use sha2::{Sha256, Digest};
use std::collections::HashMap;

const MAX_GAS_PER_BLOCK: u64 = 1_000_000;


#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Transaction {
    pub from: String,
    pub to: String,
    pub amount: f64,
    pub currency_type: CurrencyType,
    pub timestamp: DateTime<Utc>,
    pub signature: Option<Vec<u8>>,
    pub smart_contract: Option<SmartContract>,
    pub gas_limit: u64,
}

impl Transaction {
    pub fn new(from: String, to: String, amount: f64, currency_type: CurrencyType, gas_limit: u64) -> Self {
        Transaction {
            from,
            to,
            amount,
            currency_type,
            timestamp: Utc::now(),
            signature: None,
            smart_contract: None,
            gas_limit,
        }
    }


    pub fn with_smart_contract(mut self, smart_contract: SmartContract) -> Self {
        self.smart_contract = Some(smart_contract);
        self
    }

    pub fn sign(&mut self, keypair: &Keypair) -> Result<(), ed25519_dalek::SignatureError> {
        let message = self.to_bytes();
        let signature = keypair.sign(&message);
        self.signature = Some(signature.to_bytes().to_vec());
        Ok(())
    }

    pub fn verify(&self, public_key: &PublicKey) -> Result<bool, ed25519_dalek::SignatureError> {
        if let Some(sig_bytes) = &self.signature {
            let signature = Signature::from_bytes(sig_bytes)?;
            let message = self.to_bytes();
            public_key.verify(&message, &signature).map(|_| true)
        } else {
            Ok(false)
        }
    }

    fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.extend(self.from.as_bytes());
        bytes.extend(self.to.as_bytes());
        bytes.extend(self.amount.to_le_bytes());
        bytes.extend(self.currency_type.to_string().as_bytes());
        bytes.extend(self.timestamp.timestamp().to_le_bytes());
        bytes.extend(self.gas_limit.to_le_bytes());
        if let Some(contract) = &self.smart_contract {
            bytes.extend(serde_json::to_vec(contract).unwrap_or_default());
        }
        bytes
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Block {
    pub index: u64,
    pub timestamp: DateTime<Utc>,
    pub transactions: Vec<Transaction>,
    pub previous_hash: String,
    pub hash: String,
    pub proposer: String,
    pub nonce: u64,
    pub smart_contract_results: Vec<SmartContractResult>,
    pub gas_used: u64,
}




#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SmartContractResult {
    pub contract_id: String,
    pub result: Option<Result<(), String>>,
    pub gas_used: u64,
}

impl Block {
    pub fn new(index: u64, transactions: Vec<Transaction>, previous_hash: String, proposer: String) -> Self {
        let mut block = Block {
            index,
            timestamp: Utc::now(),
            transactions,
            previous_hash,
            hash: String::new(),
            proposer,
            nonce: 0,
            smart_contract_results: Vec::new(),
            gas_used: 0,
            smart_contracts: Vec::new(),
        };
        block.hash = block.calculate_hash();
        block
    }

    pub fn calculate_hash(&self) -> String {
        let mut hasher = Sha256::new();
        hasher.update(self.index.to_string().as_bytes());
        hasher.update(self.timestamp.timestamp().to_string().as_bytes());
        for transaction in &self.transactions {
            hasher.update(transaction.to_bytes());
        }
        hasher.update(self.previous_hash.as_bytes());
        hasher.update(self.proposer.as_bytes());
        hasher.update(self.nonce.to_string().as_bytes());
        for result in &self.smart_contract_results {
            hasher.update(&result.contract_id);
            hasher.update(result.result.as_ref().map(|_| "Ok").unwrap_or("Err").as_bytes());
            hasher.update(result.gas_used.to_le_bytes());
        }
        hasher.update(self.gas_used.to_le_bytes());
        format!("{:x}", hasher.finalize())
    }

    pub fn add_smart_contract_result(&mut self, contract_id: String, result: Result<(), String>, gas_used: u64) {
        self.smart_contract_results.push(SmartContractResult {
            contract_id,
            result: Some(result),
            gas_used,
        });
        self.gas_used += gas_used;
        self.hash = self.calculate_hash();
    }
}

impl Blockchain {
    pub fn new() -> Self {
        let mut blockchain = Blockchain {
            chain: Vec::new(),
            pending_transactions: Vec::new(),
            consensus: PoCConsensus::new(0.5, 0.66),
            smart_contracts: HashMap::new(),
            execution_environment: ExecutionEnvironment::new(),
        };
        
        // Create genesis block
        let genesis_block = Block::new(
            0,
            Vec::new(),
            String::from("0"),
            String::from("Genesis"),
        );
        blockchain.chain.push(genesis_block);
        
        blockchain
    }

    pub fn deploy_smart_contract(&mut self, contract: SmartContract) -> Result<(), String> {
        self.smart_contracts.insert(contract.id.clone(), contract);
        Ok(())
    }

    pub fn create_block(&mut self, proposer: String) -> Result<(), String> {
        let previous_block = self.chain.last().ok_or("Chain is empty")?;
        let mut new_block = Block::new(
            previous_block.index + 1,
            self.pending_transactions.clone(),
            previous_block.hash.clone(),
            proposer.clone(),
        );

        // Execute smart contracts and add results
        let mut total_gas_used = 0;
        let mut contract_results = Vec::new();
    
        for transaction in &new_block.transactions {
            if let Some(contract) = &transaction.smart_contract {
                if let Some(stored_contract) = self.smart_contracts.get_mut(&contract.id) {
                    let available_gas = MAX_GAS_PER_BLOCK.saturating_sub(total_gas_used);
                    let gas_limit = transaction.gas_limit.min(available_gas);
                    let result = stored_contract.execute(&mut self.execution_environment);
                    let gas_used = gas_limit / 2; // Simulating gas usage
                    contract_results.push((contract.id.clone(), result, gas_used));
                    total_gas_used += gas_used;
                    if total_gas_used >= MAX_GAS_PER_BLOCK {
                        break;
                    }
                } else {
                    return Err(format!("Smart contract with id {} not found", contract.id));
                }
            }
        }

        // Add contract results to the block
        for (contract_id, result, gas_used) in contract_results {
            new_block.add_smart_contract_result(contract_id, result, gas_used);
        }
    
        if self.validate_block(&new_block).is_err() {
            return Err("Invalid block".to_string());
        }
    
        self.chain.push(new_block);
        self.pending_transactions.clear();
        self.consensus.update_reputation(&proposer, 0.1);
        Ok(())
    }

    pub fn execute_smart_contracts(&mut self) -> Result<(), String> {
        let contracts_to_execute: Vec<SmartContract> = self.smart_contracts.values().cloned().collect();
        for contract in contracts_to_execute {
            contract.execute(&mut self.execution_environment)?;
        }
        Ok(())
    }

    pub fn validate_block(&self, block: &Block) -> Result<(), String> {
        // Check if the block index is correct
        if block.index != self.chain.len() as u64 {
            return Err("Invalid block index".to_string());
        }

        // Check if the previous hash is correct
        if let Some(previous_block) = self.chain.last() {
            if block.previous_hash != previous_block.hash {
                return Err("Invalid previous hash".to_string());
            }
        } else if block.index != 0 {
            return Err("Previous block not found".to_string());
        }

        // Validate all transactions in the block
        for transaction in &block.transactions {
            if let Err(e) = self.validate_transaction(transaction) {
                return Err(format!("Invalid transaction: {}", e));
            }
        }

        // Validate smart contract results
        if block.gas_used > MAX_GAS_PER_BLOCK {
            return Err("Block exceeds maximum gas limit".to_string());
        }

        for result in &block.smart_contract_results {
            if !self.smart_contracts.contains_key(&result.contract_id) {
                return Err(format!("Smart contract {} not found", result.contract_id));
            }
        }

        // Check if the block hash is correct
        if block.hash != block.calculate_hash() {
            return Err("Invalid block hash".to_string());
        }

        Ok(())
    }

    pub fn validate_transaction(&self, transaction: &Transaction) -> Result<(), String> {
        // Check if the sender has enough balance
        // (This is a simplified check and should be more comprehensive in a real implementation)
        if transaction.amount < 0.0 {
            return Err("Invalid transaction amount".to_string());
        }

        // Verify the transaction signature
        // (In a real implementation, you would need to retrieve the sender's public key)
        if let Some(_signature) = &transaction.signature {
            // Placeholder for signature verification
            // public_key.verify(&transaction.to_bytes(), signature).map_err(|e| e.to_string())?;
        } else {
            return Err("Transaction is not signed".to_string());
        }

        // If there's a smart contract, validate it
        if let Some(contract) = &transaction.smart_contract {
            if !self.smart_contracts.contains_key(&contract.id) {
                return Err(format!("Smart contract {} not found", contract.id));
            }
        }

        Ok(())
    }

    pub fn add_transaction(&mut self, transaction: Transaction) {
        self.pending_transactions.push(transaction);
    }

    pub fn get_latest_block(&self) -> Option<&Block> {
        self.chain.last()
    }

    pub fn is_chain_valid(&self) -> bool {
        for (i, block) in self.chain.iter().enumerate() {
            if let Err(e) = self.validate_block(block) {
                eprintln!("Block {} is invalid: {}", i, e);
                return false;
            }
        }
        true
    }

    pub fn store_smart_contract(&mut self, contract: SmartContract) -> Result<(), String> {
        if self.smart_contracts.contains_key(&contract.id) {
            return Err(format!("Smart contract with id {} already exists", contract.id));
        }
        self.smart_contracts.insert(contract.id.clone(), contract);
        Ok(())
    }

    pub fn get_smart_contract(&self, id: &str) -> Option<&SmartContract> {
        self.smart_contracts.get(id)
    }

    pub fn update_smart_contract(&mut self, id: &str, updated_contract: SmartContract) -> Result<(), String> {
        if !self.smart_contracts.contains_key(id) {
            return Err(format!("Smart contract with id {} does not exist", id));
        }
        self.smart_contracts.insert(id.to_string(), updated_contract);
        Ok(())
    }

    pub fn remove_smart_contract(&mut self, id: &str) -> Result<(), String> {
        if self.smart_contracts.remove(id).is_none() {
            return Err(format!("Smart contract with id {} does not exist", id));
        }
        Ok(())
    }
}


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
    fn test_block_creation() {
        let mut blockchain = Blockchain::new();
        let transaction = Transaction::new(
            "Alice".to_string(),
            "Bob".to_string(),
            100.0,
            CurrencyType::BasicNeeds,
            1000,
        );
        blockchain.add_transaction(transaction);
        blockchain.create_block("Proposer".to_string()).unwrap();
        assert_eq!(blockchain.chain.len(), 2);
        assert_eq!(blockchain.chain[1].index, 1);
        assert_eq!(blockchain.chain[1].transactions.len(), 1);
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
        blockchain.create_block("Proposer".to_string()).unwrap();
        assert!(blockchain.is_chain_valid());

        // Tamper with a block
        blockchain.chain[1].transactions[0].amount = 200.0;
        assert!(!blockchain.is_chain_valid());
    }

    #[test]
    fn test_smart_contract_integration() {
        let mut blockchain = Blockchain::new();
        let smart_contract = SmartContract::new(
            crate::smart_contract::ContractType::AssetTransfer,
            "Creator".to_string(),
            "{}".to_string(),
        );
        blockchain.store_smart_contract(smart_contract.clone()).unwrap();
        
        let transaction = Transaction::new(
            "Alice".to_string(),
            "Bob".to_string(),
            100.0,
            CurrencyType::BasicNeeds,
            1000,
        ).with_smart_contract(smart_contract);
        blockchain.add_transaction(transaction);
        blockchain.create_block("Proposer".to_string()).unwrap();
        
        let latest_block = blockchain.get_latest_block().unwrap();
        assert_eq!(latest_block.smart_contract_results.len(), 1);
    }
}