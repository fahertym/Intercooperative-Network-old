use crate::consensus::PoCConsensus;
use crate::currency::CurrencyType;
use crate::democracy::{DemocraticSystem, ProposalCategory, ProposalType};
use chrono::{DateTime, Utc, Duration};
use serde::{Serialize, Deserialize};
use sha2::{Sha256, Digest};
use std::fmt;
use crate::smart_contract::{SmartContract, ExecutionEnvironment};

impl fmt::Display for CurrencyType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Transaction {
    pub from: String,
    pub to: String,
    pub amount: f64,
    pub currency_type: CurrencyType,
    pub timestamp: DateTime<Utc>,
}

impl Transaction {
    pub fn new(from: String, to: String, amount: f64, currency_type: CurrencyType) -> Self {
        Transaction {
            from,
            to,
            amount,
            currency_type,
            timestamp: Utc::now(),
        }
    }

    pub fn is_valid_amount(&self) -> bool {
        self.amount >= 0.0
    }

    fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.extend(self.from.as_bytes());
        bytes.extend(self.to.as_bytes());
        bytes.extend(self.amount.to_le_bytes());
        bytes.extend(self.currency_type.to_string().as_bytes());
        bytes.extend(self.timestamp.timestamp().to_le_bytes());
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
    pub smart_contracts: Vec<SmartContract>,
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
            smart_contracts: vec![],
        };
        block.hash = block.calculate_hash();
        block
    }

    pub fn add_smart_contract(&mut self, contract: SmartContract) {
        self.smart_contracts.push(contract);
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
        format!("{:x}", hasher.finalize())
    }
}

pub struct Blockchain {
    pub chain: Vec<Block>,
    pub pending_blocks: Vec<Block>,
    pub consensus: PoCConsensus,
    pub democratic_system: DemocraticSystem,
    pub execution_environment: ExecutionEnvironment,
}

impl Blockchain {
    pub fn new() -> Self {
        let mut blockchain = Blockchain {
            chain: Vec::new(),
            pending_blocks: Vec::new(),
            consensus: PoCConsensus::new(0.5, 0.66),
            democratic_system: DemocraticSystem::new(),
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
        let current_block = self.get_latest_block()?;
        current_block.add_smart_contract(contract);
        Ok(())
    }

    pub fn execute_smart_contracts(&mut self) -> Result<(), String> {
        let mut execution_environment = std::mem::take(&mut self.execution_environment);
    
        if let Some(current_block) = self.chain.last_mut() {
            for contract in &current_block.smart_contracts {
                let mut contract_clone = contract.clone();
                contract_clone.execute(&mut execution_environment)?;
            }
        }
    
        self.execution_environment = execution_environment;
        Ok(())
    }
    
    fn get_latest_block(&mut self) -> Result<&mut Block, String> {
        self.chain.last_mut().ok_or_else(|| "Blockchain is empty".to_string())
    }

    pub fn create_proposal(&mut self, title: String, description: String, proposer: String,
                           voting_duration: Duration, proposal_type: ProposalType,
                           category: ProposalCategory, required_quorum: f64) -> Result<String, String> {
        if !self.consensus.is_eligible(&proposer) {
            return Err("Proposer is not eligible to create a proposal".to_string());
        }

        let execution_timestamp = Utc::now() + voting_duration + Duration::days(1);
        let proposal_id = self.democratic_system.create_proposal(
            title, description, proposer, voting_duration, proposal_type,
            category, required_quorum, Some(execution_timestamp)
        );

        Ok(proposal_id)
    }

    pub fn vote_on_proposal(&mut self, proposal_id: &str, voter: &str, in_favor: bool) -> Result<(), String> {
        let voter_weight = self.consensus.get_reputation(voter).ok_or("Voter not found")?;

        if !self.consensus.is_eligible(voter) {
            return Err("Voter is not eligible to vote".to_string());
        }

        self.democratic_system.vote(voter.to_string(), proposal_id.to_string(), in_favor, voter_weight)?;

        // Update reputation for participation in governance
        self.consensus.update_reputation(voter, 0.1);

        Ok(())
    }

    pub fn execute_pending_proposals(&mut self) -> Vec<Result<(), String>> {
        let current_time = Utc::now();
        let active_proposals: Vec<_> = self.democratic_system.list_active_proposals()
            .iter()
            .filter(|proposal| current_time >= proposal.voting_ends_at)
            .map(|proposal| proposal.id.clone())
            .collect();
    
        let mut results = Vec::new();
        let mut to_execute = Vec::new();
    
        // First, tally votes
        for id in active_proposals {
            match self.democratic_system.tally_votes(&id) {
                Ok(_) => {
                    if let Some(execution_time) = self.democratic_system.get_proposal(&id).unwrap().execution_timestamp {
                        if current_time >= execution_time {
                            to_execute.push(id);
                        }
                    }
                },
                Err(e) => results.push(Err(e)),
            }
        }
    
        // Separate mutable borrow scope
        let mut democratic_system = std::mem::replace(&mut self.democratic_system, DemocraticSystem::new());
    
        // Then, execute proposals
        for id in to_execute {
            let result = democratic_system.execute_proposal(&id, self);
            results.push(result);
        }
    
        self.democratic_system = democratic_system;
    
        results
    }

    pub fn create_block(&mut self, transactions: Vec<Transaction>) -> Result<(), String> {
        let valid_transactions: Vec<Transaction> = transactions
            .into_iter()
            .filter(|tx| tx.is_valid_amount())
            .collect();
    
        if valid_transactions.is_empty() {
            return Err("No valid transactions to include in the block".to_string());
        }
    
        let previous_block = self.chain.last().unwrap();
        let proposer = self.consensus.select_proposer().ok_or("No eligible proposer found")?;
    
        let new_block = Block::new(
            previous_block.index + 1,
            valid_transactions,
            previous_block.hash.clone(),
            proposer.clone(),
        );
        self.pending_blocks.push(new_block);
        self.consensus.update_reputation(&proposer, 0.1);
        Ok(())
    }

    pub fn vote_on_block(&mut self, voter: &str, block_index: usize, in_favor: bool) -> Result<(), String> {
        if block_index == 0 || block_index > self.chain.len() + self.pending_blocks.len() {
            return Err("Invalid block index".to_string());
        }
        if !self.consensus.is_eligible(voter) {
            return Err("Voter is not eligible".to_string());
        }
        self.consensus.submit_vote(block_index as u64, voter.to_string(), in_favor);
        Ok(())
    }

    pub fn finalize_block(&mut self, block_index: usize) {
        if block_index == 0 || block_index > self.chain.len() + self.pending_blocks.len() {
            return;
        }
        if self.consensus.is_block_valid(block_index as u64) {
            if let Some(block) = self.pending_blocks.get(0) {
                self.chain.push(block.clone());
                self.pending_blocks.remove(0);
                self.consensus.finalize_block(block_index as u64);
                println!("Block {} finalized and added to the chain", block_index);
            }
        } else {
            println!("Block {} is not valid and cannot be finalized", block_index);
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
