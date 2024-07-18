use crate::blockchain::Transaction;
use crate::sharding::ShardingManager;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

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
    InProgress,
    Completed,
    Failed,
}

pub struct CrossShardCommunicator {
    sharding_manager: Arc<Mutex<ShardingManager>>,
    pending_transactions: HashMap<String, CrossShardTransaction>,
}

impl CrossShardCommunicator {
    pub fn new(sharding_manager: Arc<Mutex<ShardingManager>>) -> Self {
        CrossShardCommunicator {
            sharding_manager,
            pending_transactions: HashMap::new(),
        }
    }

    pub fn initiate_cross_shard_transaction(&mut self, transaction: Transaction) -> Result<String, String> {
        let sharding_manager = self.sharding_manager.lock().unwrap();
        let from_shard = sharding_manager.get_shard_for_address(&transaction.from);
        let to_shard = sharding_manager.get_shard_for_address(&transaction.to);

        if from_shard == to_shard {
            return Err("Not a cross-shard transaction".to_string());
        }

        let cross_shard_tx = CrossShardTransaction {
            transaction: transaction.clone(),
            from_shard,
            to_shard,
            status: CrossShardTransactionStatus::Initiated,
        };

        let tx_id = format!("CST-{}", uuid::Uuid::new_v4());
        self.pending_transactions.insert(tx_id.clone(), cross_shard_tx);

        Ok(tx_id)
    }

    pub fn process_cross_shard_transaction(&mut self, tx_id: &str) -> Result<(), String> {
        let cross_shard_tx = self.pending_transactions.get_mut(tx_id)
            .ok_or("Cross-shard transaction not found")?;

        cross_shard_tx.status = CrossShardTransactionStatus::InProgress;

        let mut sharding_manager = self.sharding_manager.lock().unwrap();
        sharding_manager.transfer_between_shards(
            cross_shard_tx.from_shard,
            cross_shard_tx.to_shard,
            &cross_shard_tx.transaction,
        ).map_err(|e| e.to_string())?;

        cross_shard_tx.status = CrossShardTransactionStatus::Completed;
        Ok(())
    }

    pub fn receive_cross_shard_transaction(&mut self, from_shard: u64, transaction: Transaction) -> Result<(), String> {
        let mut sharding_manager = self.sharding_manager.lock().unwrap();
        let to_shard = sharding_manager.get_shard_for_address(&transaction.to);

        if to_shard != sharding_manager.get_current_shard_id() {
            return Err("Incorrect destination shard".to_string());
        }

        sharding_manager.add_balance(
            &transaction.to,
            transaction.currency_type.clone(),
            transaction.amount
        ).map_err(|e| e.to_string())?;

        println!("Received cross-shard transaction from shard {} to shard {}", from_shard, to_shard);
        println!("Transaction details: {:?}", transaction);

        Ok(())
    }

    pub fn get_cross_shard_transaction_status(&self, tx_id: &str) -> Option<CrossShardTransactionStatus> {
        self.pending_transactions.get(tx_id).map(|tx| tx.status.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::currency::CurrencyType;
    use ed25519_dalek::Keypair;
    use rand::rngs::OsRng;

    #[test]
    fn test_cross_shard_transaction() {
        let sharding_manager = Arc::new(Mutex::new(ShardingManager::new(2, 10)));
        let mut communicator = CrossShardCommunicator::new(sharding_manager.clone());

        let mut csprng = OsRng{};
        let keypair: Keypair = Keypair::generate(&mut csprng);

        let mut transaction = Transaction::new(
            "Alice".to_string(),
            "Bob".to_string(),
            100.0,
            CurrencyType::BasicNeeds,
            1000,
        );
        transaction.sign(&keypair).unwrap();

        {
            let mut sharding_manager = sharding_manager.lock().unwrap();
            sharding_manager.add_address_to_shard("Alice".to_string(), 0);
            sharding_manager.add_address_to_shard("Bob".to_string(), 1);
            sharding_manager.initialize_balance("Alice".to_string(), CurrencyType::BasicNeeds, 1000.0);
        }

        let tx_id = communicator.initiate_cross_shard_transaction(transaction.clone()).unwrap();
        assert_eq!(communicator.get_cross_shard_transaction_status(&tx_id), Some(CrossShardTransactionStatus::Initiated));

        communicator.process_cross_shard_transaction(&tx_id).unwrap();
        assert_eq!(communicator.get_cross_shard_transaction_status(&tx_id), Some(CrossShardTransactionStatus::Completed));

        {
            let mut sharding_manager = sharding_manager.lock().unwrap();
            sharding_manager.set_current_shard_id(1);
        }

        let result = communicator.receive_cross_shard_transaction(0, transaction);
        assert!(result.is_ok(), "Failed to receive cross-shard transaction: {:?}", result.err());

        {
            let sharding_manager = sharding_manager.lock().unwrap();
            let bob_balance = sharding_manager.get_balance("Bob".to_string(), CurrencyType::BasicNeeds);
            assert_eq!(bob_balance, 200.0);
        }
    }
}
