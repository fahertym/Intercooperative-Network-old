use std::collections::HashMap;
use sha2::{Sha256, Digest};
use crate::blockchain::{Block, Transaction};
use crate::consensus::Consensus;
use crate::currency::CurrencyType;
use std::sync::{Arc, Mutex};

pub mod cross_shard_transaction_manager;
use cross_shard_transaction_manager::CrossShardTransactionManager;

pub trait ShardingManagerTrait: Send + Sync {
    fn get_shard_for_address(&self, address: &str) -> u64;
    fn lock_funds(&mut self, from: &str, currency_type: &CurrencyType, amount: f64, shard_id: u64) -> Result<(), String>;
    fn create_prepare_block(&mut self, transaction: &Transaction, shard_id: u64) -> Result<(), String>;
    fn commit_transaction(&mut self, transaction: &Transaction, shard_id: u64) -> Result<(), String>;
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

pub struct Shard {
    pub id: u64,
    pub blockchain: Vec<Block>,
    pub balances: HashMap<String, HashMap<CurrencyType, f64>>,
    pub locked_funds: HashMap<String, HashMap<CurrencyType, f64>>,
}

impl ShardingManager {
    pub fn new(shard_count: u64, nodes_per_shard: usize, consensus: Arc<Mutex<Consensus>>) -> Self {
        let mut shards = HashMap::new();
        for i in 0..shard_count {
            shards.insert(i, Arc::new(Mutex::new(Shard {
                id: i,
                blockchain: Vec::new(),
                balances: HashMap::new(),
                locked_funds: HashMap::new(),
            })));
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

    pub fn process_cross_shard_transaction(&self, transaction: Transaction) -> Result<(), String> {
        let cross_shard_tx_manager = self.cross_shard_tx_manager.as_ref().ok_or("Cross-shard transaction manager not initialized")?;
        let tx_manager_arc = Arc::clone(cross_shard_tx_manager);
        let mut manager = tx_manager_arc.lock().map_err(|_| "Failed to acquire lock on cross shard transaction manager")?;
        let tx_id = manager.initiate_cross_shard_transaction(transaction)?;
        drop(manager);
        {
            let mut manager = tx_manager_arc.lock().map_err(|_| "Failed to acquire lock on cross shard transaction manager")?;
            manager.process_cross_shard_transaction(&tx_id)?;
        }
        {
            let mut manager = tx_manager_arc.lock().map_err(|_| "Failed to acquire lock on cross shard transaction manager")?;
            manager.finalize_cross_shard_transaction(&tx_id)?;
        }
        Ok(())
    }

    fn hash_data(&self, data: &[u8]) -> u64 {
        let mut hasher = Sha256::new();
        hasher.update(data);
        let result = hasher.finalize();

        let mut bytes = [0u8; 8];
        bytes.copy_from_slice(&result[..8]);
        u64::from_be_bytes(bytes)
    }
}

impl ShardingManagerTrait for ShardingManager {
    fn get_shard_for_address(&self, address: &str) -> u64 {
        *self.address_to_shard.get(address).unwrap_or(&0)
    }

    fn lock_funds(&mut self, from: &str, currency_type: &CurrencyType, amount: f64, shard_id: u64) -> Result<(), String> {
        let shard = self.shards.get_mut(&shard_id).ok_or("Shard not found")?;
        let mut shard = shard.lock().map_err(|_| "Failed to acquire lock on shard")?;

        let locked_funds = shard.locked_funds.entry(from.to_string()).or_insert_with(HashMap::new);
        let current_amount = locked_funds.entry(currency_type.clone()).or_insert(0.0);
        *current_amount += amount;

        Ok(())
    }

    fn create_prepare_block(&mut self, transaction: &Transaction, shard_id: u64) -> Result<(), String> {
        let shard = self.shards.get_mut(&shard_id).ok_or("Shard not found")?;
        let mut shard = shard.lock().map_err(|_| "Failed to acquire lock on shard")?;

        let prepare_block = Block::new(
            shard.blockchain.len() as u64,
            vec![transaction.clone()],
            shard.blockchain.last().map(|b| b.hash.clone()).unwrap_or_default(),
        );

        shard.blockchain.push(prepare_block);
        Ok(())
    }

    fn commit_transaction(&mut self, transaction: &Transaction, shard_id: u64) -> Result<(), String> {
        let shard = self.shards.get_mut(&shard_id).ok_or("Shard not found")?;
        let mut shard = shard.lock().map_err(|_| "Failed to acquire lock on shard")?;

        if let Some(locked_funds) = shard.locked_funds.get_mut(&transaction.from) {
            if let Some(_amount) = locked_funds.remove(&transaction.currency_type) {
                if locked_funds.is_empty() {
                    shard.locked_funds.remove(&transaction.from);
                }
            }
        }

        shard.balances.entry(transaction.to.clone())
            .or_insert_with(HashMap::new)
            .entry(transaction.currency_type.clone())
            .and_modify(|e| *e += transaction.amount)
            .or_insert(transaction.amount);

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
    use crate::consensus::Consensus;
    use crate::currency::CurrencyType;

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
        let consensus = Arc::new(Mutex::new(Consensus::new()));
        let mut manager = ShardingManager::new(2, 10, Arc::clone(&consensus));

        manager.initialize_balance("Alice".to_string(), CurrencyType::BasicNeeds, 1000.0).unwrap();
        manager.add_address_to_shard("Alice".to_string(), 0);
        manager.add_address_to_shard("Bob".to_string(), 1);

        let transaction = Transaction::new(
            "Alice".to_string(),
            "Bob".to_string(),
            100.0,
            CurrencyType::BasicNeeds,
            1000,
        );

        assert!(manager.process_cross_shard_transaction(transaction).is_ok());

        assert_eq!(manager.get_balance("Alice", &CurrencyType::BasicNeeds), 900.0);
        assert_eq!(manager.get_balance("Bob", &CurrencyType::BasicNeeds), 100.0);
    }
}