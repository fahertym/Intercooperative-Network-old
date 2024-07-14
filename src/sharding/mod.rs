use std::collections::HashMap;
use crate::blockchain::{Block, Transaction}; // Ensure Blockchain is not imported here as it's unused
use crate::currency::CurrencyType;
use crate::consensus::Consensus;
use std::sync::{Arc, Mutex};

pub mod cross_shard_transaction_manager;
use cross_shard_transaction_manager::CrossShardTransactionManager;

pub trait ShardingManagerTrait: Send + Sync {
    fn get_shard_for_address(&self, address: &str) -> u64;
    fn lock_funds(&mut self, _from: &str, _currency_type: &CurrencyType, _amount: f64, _shard_id: u64) -> Result<(), String> {
        // Add your implementation here
        Ok(())
    }
    fn create_prepare_block(&mut self, _transaction: &Transaction, _shard_id: u64) -> Result<(), String>;
    fn commit_transaction(&mut self, _transaction: &Transaction, _shard_id: u64) -> Result<(), String>;
    fn get_balance(&self, address: &str, currency_type: &CurrencyType) -> f64;
}

#[derive(Clone)]
pub struct ShardingManager {
    pub shards: HashMap<u64, Arc<Mutex<Shard>>>,
    pub shard_count: u64,
    pub nodes_per_shard: usize,
    pub address_to_shard: HashMap<String, u64>,
    pub cross_shard_tx_manager: Option<Arc<Mutex<CrossShardTransactionManager>>>,
}

impl ShardingManager {
    pub fn process_cross_shard_transaction(&self, _transaction: Transaction) -> Result<(), String> {
        // Add your implementation here
        Ok(())
    }
}

pub struct Shard {
    pub id: u64,
    pub blockchain: Vec<Block>,
    pub balances: HashMap<String, HashMap<CurrencyType, f64>>,
    pub locked_funds: HashMap<String, HashMap<CurrencyType, f64>>,
}

impl Shard {
    pub fn new(id: u64) -> Self {
        Shard {
            id,
            blockchain: Vec::new(),
            balances: HashMap::new(),
            locked_funds: HashMap::new(),
        }
    }
}

impl ShardingManager {
    pub fn new(shard_count: u64, nodes_per_shard: usize, consensus: Arc<Mutex<Consensus>>) -> Self {
        let mut shards = HashMap::new();
        for i in 0..shard_count {
            shards.insert(i, Arc::new(Mutex::new(Shard::new(i))));
        }

        let mut sharding_manager = ShardingManager {
            shards,
            shard_count,
            nodes_per_shard,
            address_to_shard: HashMap::new(),
            cross_shard_tx_manager: None,
        };

        let cross_shard_tx_manager = Arc::new(Mutex::new(CrossShardTransactionManager::new(
            Arc::new(Mutex::new(sharding_manager.clone())),
            consensus,
        )));
        sharding_manager.cross_shard_tx_manager = Some(cross_shard_tx_manager);

        sharding_manager
    }

    pub fn add_address_to_shard(&mut self, address: String, shard_id: u64) {
        self.address_to_shard.insert(address, shard_id);
    }

    pub fn initialize_balance(&mut self, address: String, currency_type: CurrencyType, amount: f64) -> Result<(), String> {
        let shard_id = self.get_shard_for_address(&address);
        if let Some(shard) = self.shards.get_mut(&shard_id) {
            let mut shard = shard.lock().map_err(|_| "Failed to acquire lock on shard")?;
            shard.balances
                .entry(address)
                .or_insert_with(HashMap::new)
                .insert(currency_type, amount);
        }
        Ok(())
    }
}

impl ShardingManagerTrait for ShardingManager {
    fn get_shard_for_address(&self, address: &str) -> u64 {
        *self.address_to_shard.get(address).unwrap_or(&0)
    }

    fn lock_funds(&mut self, _from: &str, _currency_type: &CurrencyType, _amount: f64, _shard_id: u64) -> Result<(), String> {
        let shard = self.shards.get_mut(&_shard_id).ok_or("Shard not found")?;
        let mut shard = shard.lock().map_err(|_| "Failed to acquire lock on shard")?;

        let locked_funds = shard.locked_funds.entry(_from.to_string()).or_insert_with(HashMap::new);
        let current_amount = locked_funds.entry(_currency_type.clone()).or_insert(0.0);
        *current_amount += _amount;

        Ok(())
    }

    fn create_prepare_block(&mut self, _transaction: &Transaction, _shard_id: u64) -> Result<(), String> {
        let shard = self.shards.get_mut(&_shard_id).ok_or("Shard not found")?;
        let mut shard = shard.lock().map_err(|_| "Failed to acquire lock on shard")?;

        let prepare_block = Block::new(
            shard.blockchain.len() as u64,
            vec![_transaction.clone()],
            shard.blockchain.last().map(|b| b.hash.clone()).unwrap_or_default(),
        );

        shard.blockchain.push(prepare_block);
        Ok(())
    }

    fn commit_transaction(&mut self, _transaction: &Transaction, _shard_id: u64) -> Result<(), String> {
        let shard = self.shards.get_mut(&_shard_id).ok_or("Shard not found")?;
        let mut shard = shard.lock().map_err(|_| "Failed to acquire lock on shard")?;

        if let Some(locked_funds) = shard.locked_funds.get_mut(&_transaction.from) {
            if let Some(_amount) = locked_funds.remove(&_transaction.currency_type) {
                if locked_funds.is_empty() {
                    shard.locked_funds.remove(&_transaction.from);
                }
            }
        }

        shard.balances.entry(_transaction.to.clone())
            .or_insert_with(HashMap::new)
            .entry(_transaction.currency_type.clone())
            .and_modify(|e| *e += _transaction.amount)
            .or_insert(_transaction.amount);

        Ok(())
    }

    fn get_balance(&self, address: &str, currency_type: &CurrencyType) -> f64 {
        let shard_id = self.get_shard_for_address(address);
        if let Some(shard) = self.shards.get(&shard_id) {
            let shard = shard.lock().unwrap();
            shard.balances
                .get(address)
                .and_then(|balances| balances.get(currency_type))
                .cloned()
                .unwrap_or(0.0)
        } else {
            0.0
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::blockchain::Blockchain; // Ensure this import is here
    use crate::consensus::Consensus;

    #[test]
    fn test_create_sharding_manager() {
        let consensus = Arc::new(Mutex::new(Consensus::new()));
        let manager = ShardingManager::new(4, 10, consensus);
        assert_eq!(manager.shards.len(), 4);
        assert_eq!(manager.shard_count, 4);
        assert_eq!(manager.nodes_per_shard, 10);
    }

    #[test]
    fn test_cross_shard_transaction() {
        struct MockShardingManager;

        impl ShardingManagerTrait for MockShardingManager {
            fn get_shard_for_address(&self, _address: &str) -> u64 {
                0 // Assuming all addresses map to shard 0 for simplicity
            }

            fn lock_funds(&mut self, _from: &str, _currency_type: &CurrencyType, _amount: f64, _shard_id: u64) -> Result<(), String> {
                Ok(())
            }

            fn create_prepare_block(&mut self, _transaction: &Transaction, _shard_id: u64) -> Result<(), String> {
                Ok(())
            }

            fn commit_transaction(&mut self, _transaction: &Transaction, _shard_id: u64) -> Result<(), String> {
                Ok(())
            }

            fn get_balance(&self, _address: &str, _currency_type: &CurrencyType) -> f64 {
                100.0 // Mock balance
            }
        }

        let consensus = Arc::new(Mutex::new(Consensus::new()));
        let sharding_manager = Arc::new(Mutex::new(MockShardingManager));
        let mut blockchain = Blockchain::new(consensus, sharding_manager.clone());

        let transaction = Transaction::new(
            "Alice".to_string(),
            "Bob".to_string(),
            1000.0,
            CurrencyType::BasicNeeds,
            0,
        );

        assert!(blockchain.add_transaction(transaction.clone()).is_ok());
        assert_eq!(blockchain.pending_transactions.len(), 1, "The transaction should be in the pending transactions");
        assert!(blockchain.process_cross_shard_transaction(transaction).is_ok());

        let sharding_manager = sharding_manager.lock().unwrap();
        assert_eq!(sharding_manager.get_balance("Alice", &CurrencyType::BasicNeeds), 100.0);
        assert_eq!(sharding_manager.get_balance("Bob", &CurrencyType::BasicNeeds), 100.0);
    }
}
