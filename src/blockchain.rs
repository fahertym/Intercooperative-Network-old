use sha2::{Sha256, Digest};
use chrono::Utc;
use serde::{Serialize, Deserialize};
use std::collections::HashMap;

use crate::consensus::PoCConsensus;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Block {
    pub index: u64,
    pub timestamp: i64,
    pub data: String,
    pub previous_hash: String,
    pub hash: String,
    pub proposer: String,
}

impl Block {
    pub fn new(index: u64, data: String, previous_hash: String, proposer: String) -> Self {
        let timestamp = Utc::now().timestamp();
        let mut block = Block {
            index,
            timestamp,
            data,
            previous_hash,
            hash: String::new(),
            proposer,
        };
        block.hash = block.calculate_hash();
        block
    }

    fn calculate_hash(&self) -> String {
        let mut hasher = Sha256::new();
        hasher.update(format!("{}{}{}{}{}", self.index, self.timestamp, self.data, self.previous_hash, self.proposer));
        format!("{:x}", hasher.finalize())
    }
}

pub struct Blockchain {
    pub chain: Vec<Block>,
    pub consensus: PoCConsensus,
    pending_blocks: HashMap<u64, Block>,
}

impl Blockchain {
    pub fn new() -> Self {
        Blockchain {
            chain: vec![Block::new(0, String::from("Genesis Block"), String::from("0"), String::from("Genesis Proposer"))],
            consensus: PoCConsensus::new(0.5, 0.66), // 66% vote threshold
            pending_blocks: HashMap::new(),
        }
    }

    pub fn propose_block(&mut self, data: String) -> Result<(), String> {
        let proposer = self.consensus.select_proposer().ok_or("No eligible proposer")?;
        let previous_block = self.chain.last().ok_or("Chain is empty")?;
        let new_block = Block::new(
            previous_block.index + 1,
            data,
            previous_block.hash.clone(),
            proposer.clone(),
        );
        self.pending_blocks.insert(new_block.index, new_block);
        self.consensus.update_reputation(&proposer, 0.1);  // Reward for successful block proposal
        Ok(())
    }

    pub fn vote_on_block(&mut self, block_index: u64, voter: String, in_favor: bool) -> Result<(), String> {
        if !self.pending_blocks.contains_key(&block_index) {
            return Err("Block not found in pending blocks".to_string());
        }
        if !self.consensus.is_eligible(&voter) {
            return Err("Voter is not eligible".to_string());
        }
        self.consensus.submit_vote(block_index, voter, in_favor);
        Ok(())
    }

    pub fn finalize_blocks(&mut self) {
        let valid_blocks: Vec<_> = self.pending_blocks.iter()
            .filter(|(&index, _)| self.consensus.is_block_valid(index))
            .map(|(&index, block)| (index, block.clone()))
            .collect();

        for (index, block) in valid_blocks {
            self.chain.push(block);
            self.consensus.finalize_block(index);
            self.pending_blocks.remove(&index);
        }
    }

    pub fn check_for_slashing(&mut self) {
        for (_, block) in &self.pending_blocks {
            if block.data.contains("malicious") {
                self.consensus.slash_reputation(&block.proposer, "critical_offense");
            }
        }
    }

    pub fn maintain_blockchain(&mut self) {
        self.consensus.decay_reputations();
        self.consensus.rehabilitate_members();
        self.check_for_slashing();
    }
}