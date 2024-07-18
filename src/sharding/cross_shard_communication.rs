use crate::blockchain::Transaction;
use crate::sharding::ShardingManager;
use crate::currency::CurrencyType;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tokio::sync::mpsc;
use uuid::Uuid;
use crate::error::{Error, Result};

#[derive(Clone, Debug)]
pub struct CrossShardTransaction {
    pub transaction: Transaction,
    pub from_shard: u64,
    pub to_shard: u64,
    pub status: CrossShardTransactionStatus,
}

#[derive(Clone, Debug, PartialEq)]
pub enum CrossShardTransactionStatus {
    Initiated,
    LockAcquired,
    Committed,
    Failed(String),
}

pub struct CrossShardCommunicator {
    sharding_manager: Arc<Mutex<ShardingManager>>,
    pending_transactions: HashMap<String, CrossShardTransaction>,
    tx_channels: HashMap<u64, mpsc::Sender<CrossShardTransaction>>,
}

impl CrossShardCommunicator {
    pub fn new(sharding_manager: Arc<Mutex<ShardingManager>>) -> Self {
        let mut tx_channels = HashMap::new();
        let shard_count = sharding_manager.lock().unwrap().get_shard_count();
        for i in 0..shard_count {
            let (tx, mut rx) = mpsc::channel(100);
            tx_channels.insert(i, tx);
            let sm = Arc::clone(&sharding_manager);
            tokio::spawn(async move {
                while let Some(transaction) = rx.recv().await {
                    if let Err(e) = Self::process_transaction(sm.clone(), transaction).await {
                        eprintln!("Error processing cross-shard transaction: {}", e);
                    }
                }
            });
        }

        CrossShardCommunicator {
            sharding_manager,
            pending_transactions: HashMap::new(),
            tx_channels,
        }
    }

    pub async fn initiate_cross_shard_transaction(&mut self, transaction: Transaction) -> Result<String> {
        let sharding_manager = self.sharding_manager.lock().unwrap();
        let from_shard = sharding_manager.get_shard_for_address(&transaction.from);
        let to_shard = sharding_manager.get_shard_for_address(&transaction.to);

        if from_shard == to_shard {
            return Err(Error::ShardingError("Not a cross-shard transaction".to_string()));
        }

        let cross_shard_tx = CrossShardTransaction {
            transaction: transaction.clone(),
            from_shard,
            to_shard,
            status: CrossShardTransactionStatus::Initiated,
        };

        let tx_id = Uuid::new_v4().to_string();
        self.pending_transactions.insert(tx_id.clone(), cross_shard_tx.clone());

        if let Some(tx) = self.tx_channels.get(&from_shard) {
            tx.send(cross_shard_tx).await.map_err(|e| Error::ShardingError(e.to_string()))?;
        } else {
            return Err(Error::ShardingError(format!("Channel for shard {} not found", from_shard)));
        }

        Ok(tx_id)
    }

    async fn process_transaction(sharding_manager: Arc<Mutex<ShardingManager>>, mut transaction: CrossShardTransaction) -> Result<()> {
        // Phase 1: Lock funds in the source shard
        {
            let mut sm = sharding_manager.lock().unwrap();
            sm.transfer_between_shards(transaction.from_shard, transaction.to_shard, &transaction.transaction)?;
        }
        transaction.status = CrossShardTransactionStatus::Committed;
        Ok(())
    }

    pub fn get_transaction_status(&self, tx_id: &str) -> Option<CrossShardTransactionStatus> {
        self.pending_transactions.get(tx_id).map(|tx| tx.status.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio;

    #[tokio::test]
    async fn test_cross_shard_transaction() {
        let sharding_manager = Arc::new(Mutex::new(ShardingManager::new(2, 10)));
        let mut communicator = CrossShardCommunicator::new(sharding_manager.clone());

        {
            let mut sm = sharding_manager.lock().unwrap();
            sm.add_address_to_shard("Alice".to_string(), 0);
            sm.add_address_to_shard("Bob".to_string(), 1);
            sm.initialize_balance("Alice".to_string(), CurrencyType::BasicNeeds, 1000.0).unwrap();
        }

        let transaction = Transaction::new(
            "Alice".to_string(),
            "Bob".to_string(),
            200.0,
            CurrencyType::BasicNeeds,
            1000,
        );

        let tx_id = communicator.initiate_cross_shard_transaction(transaction).await.unwrap();

        // Wait for the transaction to be processed
        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;

        let status = communicator.get_transaction_status(&tx_id).unwrap();
        assert_eq!(status, CrossShardTransactionStatus::Committed);

        let sm = sharding_manager.lock().unwrap();
        let alice_balance = sm.get_balance("Alice".to_string(), CurrencyType::BasicNeeds).unwrap();
        let bob_balance = sm.get_balance("Bob".to_string(), CurrencyType::BasicNeeds).unwrap();
        
        assert_eq!(alice_balance, 800.0);
        assert_eq!(bob_balance, 200.0);
    }

    #[tokio::test]
    async fn test_cross_shard_transaction_insufficient_balance() {
        let sharding_manager = Arc::new(Mutex::new(ShardingManager::new(2, 10)));
        let mut communicator = CrossShardCommunicator::new(sharding_manager.clone());

        {
            let mut sm = sharding_manager.lock().unwrap();
            sm.add_address_to_shard("Charlie".to_string(), 0);
            sm.add_address_to_shard("Dave".to_string(), 1);
            sm.initialize_balance("Charlie".to_string(), CurrencyType::BasicNeeds, 100.0).unwrap();
        }

        let transaction = Transaction::new(
            "Charlie".to_string(),
            "Dave".to_string(),
            200.0,
            CurrencyType::BasicNeeds,
            1000,
        );

        let tx_id = communicator.initiate_cross_shard_transaction(transaction).await.unwrap();

        // Wait for the transaction to be processed
        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;

        let status = communicator.get_transaction_status(&tx_id).unwrap();
        assert_eq!(status, CrossShardTransactionStatus::Failed("Insufficient balance".to_string()));

        let sm = sharding_manager.lock().unwrap();
        let charlie_balance = sm.get_balance("Charlie".to_string(), CurrencyType::BasicNeeds).unwrap();
        let dave_balance = sm.get_balance("Dave".to_string(), CurrencyType::BasicNeeds).unwrap();
        
        assert_eq!(charlie_balance, 100.0);
        assert_eq!(dave_balance, 0.0);
    }
}