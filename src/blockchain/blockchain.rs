use tokio::sync::RwLock;
use std::sync::Arc;
use futures::future;
use chrono::Utc;
use crate::error::{Error, Result};

pub struct Block {
    pub index: u64,
    pub timestamp: i64,
    pub transactions: Vec<Transaction>,
    pub previous_hash: String,
    pub hash: String,
    pub miner: String,
}

#[derive(Clone)]
pub struct Transaction {
    pub from: String,
    pub to: String,
    pub amount: f64,
}

impl Transaction {
    pub fn new(from: &str, to: &str, amount: f64) -> Self {
        Transaction {
            from: from.to_string(),
            to: to.to_string(),
            amount,
        }
    }
}

impl Block {
    pub fn new(index: u64, previous_hash: String, transactions: Vec<Transaction>, miner: String) -> Self {
        let timestamp = Utc::now().timestamp();
        let mut block = Block {
            index,
            timestamp,
            transactions,
            previous_hash,
            hash: String::new(),
            miner,
        };
        block.hash = block.calculate_hash();
        block
    }

    pub fn genesis() -> Self {
        Block::new(0, String::from("0"), Vec::new(), String::from("Genesis"))
    }

    fn calculate_hash(&self) -> String {
        // Implement proper hash calculation
        format!("block_hash_{}", self.index)
    }
}

pub struct Blockchain {
    chain: Arc<RwLock<Vec<Block>>>,
    pending_transactions: Arc<RwLock<Vec<Transaction>>>,
}

impl Blockchain {
    pub fn new() -> Self {
        Blockchain {
            chain: Arc::new(RwLock::new(vec![Block::genesis()])),
            pending_transactions: Arc::new(RwLock::new(Vec::new())),
        }
    }

    pub async fn add_transaction(&self, transaction: Transaction) -> Result<()> {
        // Add validation logic here if needed
        let mut pending_transactions = self.pending_transactions.write().await;
        pending_transactions.push(transaction);
        Ok(())
    }

    pub async fn create_block(&self, miner: String) -> Result<()> {
        let pending_transactions = {
            let mut pt = self.pending_transactions.write().await;
            std::mem::take(&mut *pt)
        };

        let last_block = {
            let chain = self.chain.read().await;
            chain.last().cloned().ok_or(Error::BlockchainError("Chain is empty".to_string()))?
        };

        let new_block = Block::new(
            last_block.index + 1,
            last_block.hash,
            pending_transactions,
            miner,
        );

        // Validate the new block
        if !self.is_valid_block(&new_block, &last_block).await {
            return Err(Error::BlockchainError("Invalid block".to_string()));
        }

        // Add the new block to the chain
        let mut chain = self.chain.write().await;
        chain.push(new_block);

        Ok(())
    }

    async fn is_valid_block(&self, block: &Block, previous_block: &Block) -> bool {
        // Implement block validation logic
        // This could include checking the block's hash, index, and transactions
        true // Placeholder
    }

    pub async fn get_balance(&self, address: &str) -> f64 {
        let chain = self.chain.read().await;
        let mut balance = 0.0;

        for block in chain.iter() {
            for transaction in &block.transactions {
                if transaction.from == address {
                    balance -= transaction.amount;
                }
                if transaction.to == address {
                    balance += transaction.amount;
                }
            }
        }

        balance
    }

    pub async fn process_transactions(&self) {
        let transactions = {
            let mut pt = self.pending_transactions.write().await;
            std::mem::take(&mut *pt)
        };

        let transaction_futures: Vec<_> = transactions
            .into_iter()
            .map(|tx| self.process_transaction(tx))
            .collect();

        future::join_all(transaction_futures).await;
    }

    async fn process_transaction(&self, transaction: Transaction) -> Result<()> {
        // Validate transaction
        if !self.is_valid_transaction(&transaction).await {
            return Err(Error::BlockchainError("Invalid transaction".to_string()));
        }

        // Process the transaction
        // This could include updating account balances, executing smart contracts, etc.

        Ok(())
    }

    async fn is_valid_transaction(&self, transaction: &Transaction) -> bool {
        // Implement transaction validation logic
        // This could include checking signatures, balances, etc.
        true // Placeholder
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_add_transaction() {
        let blockchain = Blockchain::new();
        let transaction = Transaction::new("Alice", "Bob", 100.0);

        assert!(blockchain.add_transaction(transaction).await.is_ok());

        let pending_transactions = blockchain.pending_transactions.read().await;
        assert_eq!(pending_transactions.len(), 1);
    }

    #[tokio::test]
    async fn test_create_block() {
        let blockchain = Blockchain::new();
        let transaction = Transaction::new("Alice", "Bob", 100.0);

        blockchain.add_transaction(transaction).await.unwrap();
        assert!(blockchain.create_block("Miner1".to_string()).await.is_ok());

        let chain = blockchain.chain.read().await;
        assert_eq!(chain.len(), 2); // Genesis block + new block
    }

    #[tokio::test]
    async fn test_get_balance() {
        let blockchain = Blockchain::new();
        let transaction1 = Transaction::new("Alice", "Bob", 100.0);
        let transaction2 = Transaction::new("Bob", "Alice", 50.0);

        blockchain.add_transaction(transaction1).await.unwrap();
        blockchain.add_transaction(transaction2).await.unwrap();
        blockchain.create_block("Miner1".to_string()).await.unwrap();

        assert_eq!(blockchain.get_balance("Alice").await, -50.0);
        assert_eq!(blockchain.get_balance("Bob").await, 50.0);
    }
}
