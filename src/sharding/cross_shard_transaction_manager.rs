use crate::blockchain::Transaction;
use crate::consensus::Consensus;
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex};
use uuid::Uuid;
use super::ShardingManagerTrait;

#[derive(Debug, Clone, PartialEq)]
pub enum TransactionStatus {
    Pending,
    InProgress,
    Completed,
    Failed,
}

#[derive(Debug, Clone)]
pub struct CrossShardTransaction {
    pub id: String,
    pub transaction: Transaction,
    pub from_shard: u64,
    pub to_shard: u64,
    pub status: TransactionStatus,
}

pub struct CrossShardTransactionManager {
    sharding_manager: Arc<Mutex<dyn ShardingManagerTrait + Send + 'static>>,
    #[allow(dead_code)]
    consensus: Arc<Mutex<Consensus>>,
    pending_transactions: HashMap<String, CrossShardTransaction>,
    processed_transactions: HashSet<String>,
}

impl CrossShardTransactionManager {
    pub fn new(
        sharding_manager: Arc<Mutex<dyn ShardingManagerTrait + Send + 'static>>,
        consensus: Arc<Mutex<Consensus>>
    ) -> Self {
        CrossShardTransactionManager {
            sharding_manager,
            consensus,
            pending_transactions: HashMap::new(),
            processed_transactions: HashSet::new(),
        }
    }

    pub fn initiate_cross_shard_transaction(&mut self, transaction: Transaction) -> Result<String, String> {
        let sharding_manager = self.sharding_manager.lock().map_err(|_| "Failed to acquire lock on sharding manager")?;
        let from_shard = sharding_manager.get_shard_for_address(&transaction.from);
        let to_shard = sharding_manager.get_shard_for_address(&transaction.to);

        if from_shard == to_shard {
            return Err("Not a cross-shard transaction".to_string());
        }

        let transaction_id = Uuid::new_v4().to_string();
        let cross_shard_tx = CrossShardTransaction {
            id: transaction_id.clone(),
            transaction,
            from_shard,
            to_shard,
            status: TransactionStatus::Pending,
        };

        self.pending_transactions.insert(transaction_id.clone(), cross_shard_tx);
        Ok(transaction_id)
    }

    pub fn process_cross_shard_transaction(&mut self, transaction_id: &str) -> Result<(), String> {
        let transaction = self.pending_transactions.get(transaction_id)
            .ok_or("Transaction not found")?
            .clone();

        if transaction.status != TransactionStatus::Pending {
            return Err("Transaction is not in a pending state".to_string());
        }

        if !self.verify_transaction(&transaction.transaction) {
            self.pending_transactions.get_mut(transaction_id).unwrap().status = TransactionStatus::Failed;
            return Err("Transaction verification failed".to_string());
        }

        self.lock_funds(&transaction.transaction, transaction.from_shard)?;
        self.create_prepare_block(&transaction.transaction, transaction.to_shard)?;

        let pending_tx = self.pending_transactions.get_mut(transaction_id).unwrap();
        pending_tx.status = TransactionStatus::Completed;
        self.processed_transactions.insert(transaction_id.to_string());
        Ok(())
    }

    fn verify_transaction(&self, _transaction: &Transaction) -> bool {
        // Implement transaction verification logic
        true // Placeholder implementation
    }

    fn lock_funds(&self, transaction: &Transaction, shard_id: u64) -> Result<(), String> {
        let mut sharding_manager = self.sharding_manager.lock().map_err(|_| "Failed to acquire lock on sharding manager")?;
        sharding_manager.lock_funds(&transaction.from, &transaction.currency_type, transaction.amount, shard_id)
    }

    fn create_prepare_block(&self, transaction: &Transaction, shard_id: u64) -> Result<(), String> {
        let mut sharding_manager = self.sharding_manager.lock().map_err(|_| "Failed to acquire lock on sharding manager")?;
        sharding_manager.create_prepare_block(transaction, shard_id)
    }

    pub fn finalize_cross_shard_transaction(&mut self, transaction_id: &str) -> Result<(), String> {
        let transaction = self.pending_transactions.get(transaction_id)
            .ok_or("Transaction not found")?;

        if transaction.status != TransactionStatus::Completed {
            return Err("Transaction is not in a completed state".to_string());
        }

        self.commit_changes(&transaction.transaction, transaction.from_shard)?;
        self.commit_changes(&transaction.transaction, transaction.to_shard)?;

        self.processed_transactions.insert(transaction_id.to_string());
        self.pending_transactions.remove(transaction_id);

        Ok(())
    }

    fn commit_changes(&self, transaction: &Transaction, shard_id: u64) -> Result<(), String> {
        let mut sharding_manager = self.sharding_manager.lock().map_err(|_| "Failed to acquire lock on sharding manager")?;
        sharding_manager.commit_transaction(transaction, shard_id)
    }

    pub fn get_transaction_status(&self, transaction_id: &str) -> Result<TransactionStatus, String> {
        if let Some(transaction) = self.pending_transactions.get(transaction_id) {
            Ok(transaction.status.clone())
        } else if self.processed_transactions.contains(transaction_id) {
            Ok(TransactionStatus::Completed)
        } else {
            Err("Transaction not found".to_string())
        }
    }
}

// Tests for CrossShardTransactionManager
#[cfg(test)]
mod tests {
    use super::*;
    use crate::currency::CurrencyType;

    // Mock implementation of ShardingManagerTrait for testing
    struct MockShardingManager {
        shard_map: HashMap<String, u64>,
    }

    impl ShardingManagerTrait for MockShardingManager {
        fn get_shard_for_address(&self, address: &str) -> u64 {
            *self.shard_map.get(address).unwrap_or(&0)
        }

        fn lock_funds(&mut self, _from: &str, _currency_type: &CurrencyType, _amount: f64, _shard_id: u64) -> Result<(), String> { Ok(()) }

        fn create_prepare_block(&mut self, _transaction: &Transaction, _shard_id: u64) -> Result<(), String> { Ok(()) }

        fn commit_transaction(&mut self, _transaction: &Transaction, _shard_id: u64) -> Result<(), String> { Ok(()) }

        fn get_balance(&self, _address: &str, _currency_type: &CurrencyType) -> f64 { 1000.0 }
    }

    // Test the cross-shard transaction flow
    #[test]
    fn test_cross_shard_transaction_flow() {
        let mock_sharding_manager = MockShardingManager {
            shard_map: [("Alice".to_string(), 0), ("Bob".to_string(), 1)].iter().cloned().collect(),
        };
        let sharding_manager = Arc::new(Mutex::new(mock_sharding_manager));
        let consensus = Arc::new(Mutex::new(Consensus::new()));
        let mut manager = CrossShardTransactionManager::new(sharding_manager, consensus);

        let transaction = Transaction::new(
            "Alice".to_string(),
            "Bob".to_string(),
            100.0,
            CurrencyType::BasicNeeds,
            1000,
        );

        // Initiate transaction
        let tx_id = manager.initiate_cross_shard_transaction(transaction).unwrap();
        assert_eq!(manager.get_transaction_status(&tx_id).unwrap(), TransactionStatus::Pending);

        // Process transaction
        manager.process_cross_shard_transaction(&tx_id).unwrap();
        assert_eq!(manager.get_transaction_status(&tx_id).unwrap(), TransactionStatus::Completed);

        // Finalize transaction
        manager.finalize_cross_shard_transaction(&tx_id).unwrap();
        assert_eq!(manager.get_transaction_status(&tx_id).unwrap(), TransactionStatus::Completed);

        // Verify transaction is no longer in pending_transactions
        assert!(manager.pending_transactions.is_empty());
        assert!(manager.processed_transactions.contains(&tx_id));
    }
}
