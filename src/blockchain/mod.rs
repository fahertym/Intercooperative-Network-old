use std::collections::HashMap;
use serde::{Serialize, Deserialize};
use crate::currency::CurrencyType;
use crate::consensus::PoCConsensus;

pub mod block;
pub mod transaction;

pub use block::Block;
pub use transaction::Transaction;

#[derive(Serialize, Deserialize)]
pub struct Blockchain {
    pub chain: Vec<Block>,
    pub pending_transactions: Vec<Transaction>,
    pub asset_tokens: HashMap<String, CurrencyType>,
    pub bonds: HashMap<String, CurrencyType>,
    #[serde(skip)]
    pub consensus: PoCConsensus,
}

impl Blockchain {
    pub fn new() -> Self {
        let mut blockchain = Blockchain {
            chain: vec![],
            pending_transactions: vec![],
            asset_tokens: HashMap::new(),
            bonds: HashMap::new(),
            consensus: PoCConsensus::new(0.5, 0.66),
        };
        
        let genesis_block = Block::new(0, vec![], String::new());
        blockchain.chain.push(genesis_block);
        
        blockchain
    }

    pub fn add_transaction(&mut self, transaction: Transaction) {
        self.pending_transactions.push(transaction);
    }

    pub fn create_block(&mut self, _author: String) -> Result<(), Box<dyn std::error::Error>> {
        let previous_block = self.chain.last().ok_or("No previous block found")?;
        let new_block = Block::new(
            self.chain.len() as u64,
            self.pending_transactions.clone(),
            previous_block.hash.clone(),
        );
        
        self.chain.push(new_block);
        self.pending_transactions.clear();
        Ok(())
    }

    pub fn get_latest_block(&self) -> Option<&Block> {
        self.chain.last()
    }
}