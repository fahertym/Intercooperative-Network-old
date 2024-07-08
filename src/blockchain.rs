use crate::consensus::{PoCConsensus, CurrencyType};
use crate::transaction_validator::TransactionValidator;
use chrono::{DateTime, Utc};
use ed25519_dalek::{Keypair, PublicKey, Signature, Signer, Verifier};
use serde::{Serialize, Deserialize};
use sha2::{Sha256, Digest};


#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Transaction {
    pub from: String,
    pub to: String,
    pub amount: f64,
    pub currency_type: CurrencyType,
    pub timestamp: DateTime<Utc>,
    pub signature: Option<Vec<u8>>,
}

impl Transaction {
    pub fn new(from: String, to: String, amount: f64, currency_type: CurrencyType) -> Self {
        Transaction {
            from,
            to,
            amount,
            currency_type,
            timestamp: Utc::now(),
            signature: None,
        }
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
        format!("{:x}", hasher.finalize())
    }
}

pub struct Blockchain {
    pub chain: Vec<Block>,
    pub pending_blocks: Vec<Block>,
    pub consensus: PoCConsensus,
}

impl Blockchain {
    pub fn new() -> Self {
        let mut blockchain = Blockchain {
            chain: Vec::new(),
            pending_blocks: Vec::new(),
            consensus: PoCConsensus::new(0.5, 0.66),
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

    pub fn create_block(&mut self, transactions: Vec<Transaction>, public_key: &PublicKey) -> Result<(), String> {
        let proposer = self.consensus.select_proposer().ok_or("No eligible proposer")?;
        let previous_block = self.chain.last().ok_or("Chain is empty")?;
        
        // Validate transactions
        let valid_transactions: Vec<Transaction> = transactions
            .into_iter()
            .filter(|tx| TransactionValidator::validate_transaction(tx, self, public_key))
            .collect();

        if valid_transactions.is_empty() {
            return Err("No valid transactions to include in the block".to_string());
        }

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


    
    pub fn vote_on_block(&mut self, voter: &str, block_index: u64, in_favor: bool) -> Result<(), String> {
        if block_index == 0 || block_index as usize > self.pending_blocks.len() {
            return Err("Invalid block index".to_string());
        }
        if !self.consensus.is_eligible(voter) {
            return Err("Voter is not eligible".to_string());
        }
        self.consensus.submit_vote(block_index, voter.to_string(), in_favor);
        Ok(())
    }

    pub fn finalize_block(&mut self, block_index: u64) {
        if block_index == 0 || block_index as usize > self.pending_blocks.len() {
            return;
        }
        if self.consensus.is_block_valid(block_index) {
            let block = self.pending_blocks.remove(block_index as usize - 1);
            self.chain.push(block);
            self.consensus.finalize_block(block_index);
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