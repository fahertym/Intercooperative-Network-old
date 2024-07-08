use crate::consensus::{PoCConsensus, CurrencyType};
use sha2::{Sha256, Digest};
use serde::{Serialize, Deserialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Block {
    pub index: u64,
    pub timestamp: u64,
    pub transactions: Vec<Transaction>,
    pub previous_hash: String,
    pub hash: String,
    pub proposer: String,
    pub nonce: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Transaction {
    pub from: String,
    pub to: String,
    pub amount: f64,
    pub currency_type: CurrencyType,
}

pub struct Blockchain {
    pub chain: Vec<Block>,
    pub pending_blocks: Vec<Block>,
    pub consensus: PoCConsensus,
}

impl Block {
    pub fn new(index: u64, transactions: Vec<Transaction>, previous_hash: String, proposer: String) -> Self {
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        
        let mut block = Block {
            index,
            timestamp,
            transactions,
            previous_hash,
            hash: String::new(),
            proposer,
            nonce: 0,
        };
        
        block.hash = block.calculate_hash();
        block
    }

    pub fn calculate_hash(&self) -> String {
        let mut hasher = Sha256::new();
        let block_data = serde_json::to_string(&self).unwrap();
        hasher.update(block_data);
        format!("{:x}", hasher.finalize())
    }
}

impl Blockchain {
    pub fn new() -> Self {
        let mut blockchain = Blockchain {
            chain: Vec::new(),
            pending_blocks: Vec::new(),
            consensus: PoCConsensus::new(0.5, 0.66),
        };

        // Create and add genesis block
        let genesis_block = Block::new(
            0,
            Vec::new(),
            String::from("0"),
            String::from("Genesis")
        );
        blockchain.chain.push(genesis_block);

        blockchain
    }

    pub fn create_block(&mut self, transactions: Vec<(String, String, f64, CurrencyType)>) -> Result<(), String> {
        let proposer = self.consensus.select_proposer().ok_or("No eligible proposer")?;
        let previous_block = self.chain.last().ok_or("Chain is empty")?;
        let new_block = Block::new(
            previous_block.index + 1,
            transactions.into_iter().map(|(from, to, amount, currency_type)| 
                Transaction { from, to, amount, currency_type }
            ).collect(),
            previous_block.hash.clone(),
            proposer.clone(),
        );
        self.pending_blocks.push(new_block);
        self.consensus.update_reputation(&proposer, 0.1);
        Ok(())
    }

    pub fn vote_on_block(&mut self, voter: &str, block_index: usize, in_favor: bool) -> Result<(), String> {
        if block_index == 0 || block_index > self.pending_blocks.len() {
            return Err("Invalid block index".to_string());
        }
        if !self.consensus.is_eligible(voter) {
            return Err("Voter is not eligible".to_string());
        }
        self.consensus.submit_vote(block_index as u64, voter.to_string(), in_favor);
        Ok(())
    }

    pub fn finalize_block(&mut self, block_index: usize) {
        if block_index == 0 || block_index > self.pending_blocks.len() {
            return;
        }
        if self.consensus.is_block_valid(block_index as u64) {
            let block = self.pending_blocks.remove(block_index - 1);
            self.chain.push(block);
            self.consensus.finalize_block(block_index as u64);
        }
    }

    pub fn maintain_blockchain(&mut self) {
        self.consensus.decay_reputations();
        self.consensus.rehabilitate_members();
        self.check_for_slashing();
    }

    pub fn check_for_slashing(&mut self) {
        for block in &self.pending_blocks {
            for transaction in &block.transactions {
                if transaction.amount < 0.0 {
                    self.consensus.slash_reputation(&block.proposer, "critical_offense");
                    break;
                }
            }
        }
    }
}