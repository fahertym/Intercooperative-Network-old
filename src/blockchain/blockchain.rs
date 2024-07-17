// src/blockchain/blockchain.rs

use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use rand::distributions::{Distribution, Uniform};
use rand::thread_rng;
use crate::blockchain::{Block, Transaction};
use crate::smart_contract::SmartContract;
use crate::consensus::Consensus;
use crate::currency::{AssetToken, Bond};
use log::{info, error, debug, warn};

#[derive(Clone, Serialize, Deserialize)]
pub struct Blockchain {
    pub chain: Vec<Block>,
    pub pending_transactions: Vec<Transaction>,
    #[serde(skip)]
    pub smart_contracts: HashMap<String, Box<dyn SmartContract>>,
    pub asset_tokens: HashMap<String, AssetToken>,
    pub bonds: HashMap<String, Bond>,
    #[serde(skip)]
    pub consensus: Consensus,
}

impl Blockchain {
    pub fn new() -> Self {
        let mut blockchain = Blockchain {
            chain: vec![],
            pending_transactions: vec![],
            smart_contracts: HashMap::new(),
            consensus: Consensus::new(),
            asset_tokens: HashMap::new(),
            bonds: HashMap::new(),
        };
        
        let genesis_block = Block::new(0, vec![], String::new());
        blockchain.chain.push(genesis_block);
        
        info!("New blockchain created with genesis block");
        blockchain
    }

    pub fn create_block(&mut self, proposer: String) -> Result<(), String> {
        let previous_block = self.chain.last().ok_or("No previous block found")?;
        let new_block = Block::new(self.chain.len() as u64, self.pending_transactions.clone(), previous_block.hash.clone());
        
        debug!("Validating new block");
        self.validate_block(&new_block)?;
        
        self.chain.push(new_block);
        info!("New block added to the chain. Chain length: {}", self.chain.len());
        
        self.pending_transactions.clear();
        debug!("Pending transactions cleared");
        
        Ok(())
    }

    pub fn validate_block(&self, block: &Block) -> Result<(), String> {
        if let Some(previous_block) = self.chain.last() {
            if block.previous_hash != previous_block.hash {
                error!("Invalid previous hash in new block");
                return Err("Invalid previous hash".to_string());
            }
        }

        for transaction in &block.transactions {
            if let Err(e) = self.validate_transaction(transaction) {
                error!("Transaction validation failed: {}", e);
                return Err(format!("Invalid transaction: {}", e));
            }
        }

        if block.hash != block.calculate_hash() {
            error!("Invalid block hash");
            return Err("Invalid block hash".to_string());
        }

        debug!("Block validated successfully");
        Ok(())
    }

    pub fn validate_transaction(&self, transaction: &Transaction) -> Result<(), String> {
        // Implement balance checking and signature verification logic here
        // This is a placeholder implementation
        debug!("Validating transaction: {:?}", transaction);
        Ok(())
    }

    pub fn add_transaction(&mut self, transaction: Transaction) {
        self.pending_transactions.push(transaction);
        debug!("Transaction added to pending transactions. Total pending: {}", self.pending_transactions.len());
    }

    pub fn get_latest_block(&self) -> Option<&Block> {
        self.chain.last()
    }

    pub fn is_chain_valid(&self) -> bool {
        for i in 1..self.chain.len() {
            let current_block = &self.chain[i];
            let previous_block = &self.chain[i - 1];

            if current_block.hash != current_block.calculate_hash() {
                error!("Invalid hash in block {}", i);
                return false;
            }

            if current_block.previous_hash != previous_block.hash {
                error!("Invalid previous hash in block {}", i);
                return false;
            }
        }
        debug!("Blockchain validated successfully");
        true
    }

    pub fn store_smart_contract(&mut self, contract: Box<dyn SmartContract>) -> Result<(), String> {
        let id = contract.id();
        self.smart_contracts.insert(id.clone(), contract);
        info!("Smart contract stored with ID: {}", id);
        Ok(())
    }

    pub fn get_smart_contract(&self, id: &str) -> Option<&Box<dyn SmartContract>> {
        self.smart_contracts.get(id)
    }

    pub fn update_smart_contract(&mut self, id: &str, updated_contract: Box<dyn SmartContract>) -> Result<(), String> {
        self.smart_contracts.insert(id.to_string(), updated_contract);
        info!("Smart contract updated: {}", id);
        Ok(())
    }

    pub fn remove_smart_contract(&mut self, id: &str) -> Result<(), String> {
        self.smart_contracts.remove(id);
        info!("Smart contract removed: {}", id);
        Ok(())
    }

    pub fn deploy_smart_contract(&mut self, contract: Box<dyn SmartContract>) -> Result<(), String> {
        let id = contract.id();
        if self.smart_contracts.contains_key(&id) {
            error!("Smart contract with ID {} already exists", id);
            return Err("Smart contract with this ID already exists".to_string());
        }
        self.smart_contracts.insert(id.clone(), contract);
        info!("Smart contract deployed: {}", id);
        Ok(())
    }

    pub fn execute_smart_contracts(&mut self, execution_environment: &mut crate::smart_contract::ExecutionEnvironment) -> Result<(), String> {
        let block = self.chain.last_mut().ok_or("No blocks found")?;
        let transactions = block.transactions.clone();
        for transaction in transactions {
            if let Some(contract) = self.smart_contracts.get(&transaction.smart_contract_id) {
                debug!("Executing smart contract: {}", contract.id());
                let result = contract.execute(execution_environment)?;
                block.add_smart_contract_result(contract.id(), result, transaction.gas_limit);
                info!("Smart contract executed: {}", contract.id());
            }
        }
        Ok(())
    }

    pub fn select_proposer(&self) -> Option<String> {
        let total_reputation: f64 = self.consensus.members.values().map(|member| member.reputation).sum();
        let mut rng = thread_rng();
        let selection_point: f64 = Uniform::new(0.0, total_reputation).sample(&mut rng);
        
        let mut cumulative_reputation = 0.0;
        for member in self.consensus.members.values() {
            cumulative_reputation += member.reputation;
            if cumulative_reputation >= selection_point {
                debug!("Proposer selected: {}", member.id);
                return Some(member.id.clone());
            }
        }

        warn!("No proposer selected");
        None
    }

    pub fn register_asset_token(&mut self, asset_token: &AssetToken) -> Result<(), String> {
        if self.asset_tokens.contains_key(&asset_token.asset_id) {
            error!("Asset token with ID {} already exists", asset_token.asset_id);
            return Err("Asset token with this ID already exists".to_string());
        }
        self.asset_tokens.insert(asset_token.asset_id.clone(), asset_token.clone());
        info!("Asset token registered: {}", asset_token.asset_id);
        Ok(())
    }

    pub fn transfer_asset_token(&mut self, asset_id: &str, new_owner: &str) -> Result<(), String> {
        let asset_token = self.asset_tokens.get_mut(asset_id).ok_or("Asset token not found")?;
        asset_token.owner = new_owner.to_string();
        info!("Asset token transferred: {} to {}", asset_id, new_owner);
        Ok(())
    }

    pub fn get_asset_token(&self, asset_id: &str) -> Option<&AssetToken> {
        self.asset_tokens.get(asset_id)
    }

    pub fn register_bond(&mut self, bond: &Bond) -> Result<(), String> {
        if self.bonds.contains_key(&bond.bond_id) {
            error!("Bond with ID {} already exists", bond.bond_id);
            return Err("Bond with this ID already exists".to_string());
        }
        self.bonds.insert(bond.bond_id.clone(), bond.clone());
        info!("Bond registered: {}", bond.bond_id);
        Ok(())
    }

    pub fn transfer_bond(&mut self, bond_id: &str, new_owner: &str) -> Result<(), String> {
        let bond = self.bonds.get_mut(bond_id).ok_or("Bond not found")?;
        bond.owner = new_owner.to_string();
        info!("Bond transferred: {} to {}", bond_id, new_owner);
        Ok(())
    }

    pub fn get_bond(&self, bond_id: &str) -> Option<&Bond> {
        self.bonds.get(bond_id)
    }
}