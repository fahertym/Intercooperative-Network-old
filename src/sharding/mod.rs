use std::collections::HashMap;
use sha2::{Sha256, Digest};
use crate::blockchain::{Block, Transaction};
use crate::network::Node;
use crate::currency::CurrencyType;
use crate::consensus::Consensus;
use std::sync::{Arc, Mutex};

pub mod cross_shard_transaction_manager;
pub use cross_shard_transaction_manager::CrossShardTransactionManager;

pub struct Shard {
    pub id: u64,
    pub nodes: Vec<Node>,
    pub blockchain: Vec<Block>,
    pub balances: HashMap<String, HashMap<CurrencyType, f64>>,
    pub locked_funds: HashMap<String, HashMap<CurrencyType, f64>>,
}

pub struct ShardingManager {
    shards: HashMap<u64, Arc<Mutex<Shard>>>,
    shard_count: u64,
    nodes_per_shard: usize,
    address_to_shard: HashMap<String, u64>,
    current_shard_id: u64,
    cross_shard_tx_manager: Option<CrossShardTransactionManager>,
}

impl ShardingManager {
    pub fn new(shard_count: u64, nodes_per_shard: usize, consensus: Arc<Mutex<Consensus>>) -> Arc<Mutex<Self>> {
        let mut shards = HashMap::new();
        for i in 0..shard_count {
            shards.insert(i, Arc::new(Mutex::new(Shard {
                id: i,
                nodes: Vec::new(),
                blockchain: Vec::new(),
                balances: HashMap::new(),
                locked_funds: HashMap::new(),
            })));
        }
        
        let sharding_manager = Arc::new(Mutex::new(ShardingManager {
            shards,
            shard_count,
            nodes_per_shard,
            address_to_shard: HashMap::new(),
            current_shard_id: 0,
            cross_shard_tx_manager: None,
        }));

        let cross_shard_tx_manager = CrossShardTransactionManager::new(Arc::clone(&sharding_manager), consensus);
        sharding_manager.lock().unwrap().cross_shard_tx_manager = Some(cross_shard_tx_manager);

        sharding_manager
    }

    pub fn assign_node_to_shard(&mut self, node: Node, shard_id: u64) -> Result<(), String> {
        let shard = self.shards.get(&shard_id).ok_or(format!("Shard {} not found", shard_id))?;
        let mut shard = shard.lock().map_err(|_| "Failed to acquire lock on shard".to_string())?;
        if shard.nodes.len() >= self.nodes_per_shard {
            return Err(format!("Shard {} is full", shard_id));
        }
        shard.nodes.push(node);
        Ok(())
    }

    pub fn get_shard_for_address(&self, address: &str) -> u64 {
        *self.address_to_shard.get(address).unwrap_or(&(self.hash_data(address.as_bytes()) % self.shard_count))
    }

    pub fn add_address_to_shard(&mut self, address: String, shard_id: u64) {
        self.address_to_shard.insert(address, shard_id);
    }

    pub fn initialize_balance(&mut self, address: String, currency_type: CurrencyType, amount: f64) {
        let shard_id = self.get_shard_for_address(&address);
        if let Some(shard) = self.shards.get_mut(&shard_id) {
            let mut shard = shard.lock().unwrap();
            shard.balances
                .entry(address)
                .or_insert_with(HashMap::new)
                .insert(currency_type, amount);
        }
    }

    pub fn get_balance(&self, address: String, currency_type: CurrencyType) -> f64 {
        let shard_id = self.get_shard_for_address(&address);
        if let Some(shard) = self.shards.get(&shard_id) {
            let shard = shard.lock().unwrap();
            shard.balances
                .get(&address)
                .and_then(|balances| balances.get(&currency_type))
                .cloned()
                .unwrap_or(0.0)
        } else {
            0.0
        }
    }

    pub fn lock_funds(&mut self, from: &str, currency_type: &CurrencyType, amount: f64, shard_id: u64) -> Result<(), String> {
        let shard = self.shards.get_mut(&shard_id).ok_or("Shard not found")?;
        let mut shard = shard.lock().map_err(|_| "Failed to acquire lock on shard")?;
        
        let balance = shard.balances.get_mut(from)
            .and_then(|balances| balances.get_mut(currency_type))
            .ok_or("Insufficient balance")?;

        if *balance < amount {
            return Err("Insufficient balance".to_string());
        }

        *balance -= amount;
        shard.locked_funds.entry(from.to_string())
            .or_insert_with(HashMap::new)
            .entry(currency_type.clone())
            .and_modify(|e| *e += amount)
            .or_insert(amount);

        Ok(())
    }

    pub fn create_prepare_block(&mut self, transaction: &Transaction, shard_id: u64) -> Result<(), String> {
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

    pub fn commit_transaction(&mut self, transaction: &Transaction, shard_id: u64) -> Result<(), String> {
        let shard = self.shards.get_mut(&shard_id).ok_or("Shard not found")?;
        let mut shard = shard.lock().map_err(|_| "Failed to acquire lock on shard")?;
        
        // If this is the source shard, remove the locked funds
        if let Some(locked_funds) = shard.locked_funds.get_mut(&transaction.from) {
            if let Some(_amount) = locked_funds.remove(&transaction.currency_type) {
                if locked_funds.is_empty() {
                    shard.locked_funds.remove(&transaction.from);
                }
            }
        }
    
        let to_shard_id = self.get_shard_for_address(&transaction.to);
        
        // If this is the destination shard, add the funds to the recipient
        if shard_id == to_shard_id {
            shard.balances.entry(transaction.to.clone())
                .or_insert_with(HashMap::new)
                .entry(transaction.currency_type.clone())
                .and_modify(|e| *e += transaction.amount)
                .or_insert(transaction.amount);
        }
    
        Ok(())
    }

    pub fn process_cross_shard_transaction(&mut self, transaction: Transaction) -> Result<(), String> {
        if let Some(ref mut cross_shard_tx_manager) = self.cross_shard_tx_manager {
            let tx_id = cross_shard_tx_manager.initiate_cross_shard_transaction(transaction)?;
            cross_shard_tx_manager.process_cross_shard_transaction(&tx_id)?;
            cross_shard_tx_manager.finalize_cross_shard_transaction(&tx_id)?;
            Ok(())
        } else {
            Err("Cross-shard transaction manager not initialized".to_string())
        }
    }

    fn hash_data(&self, data: &[u8]) -> u64 {
        let mut hasher = Sha256::new();
        hasher.update(data);
        let result = hasher.finalize();
        
        let mut bytes = [0u8; 8];
        bytes.copy_from_slice(&result[..8]);
        u64::from_be_bytes(bytes)
    }

    pub fn set_current_shard_id(&mut self, shard_id: u64) {
        self.current_shard_id = shard_id;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::network::Node;
    use crate::network::network::NodeType;

    #[test]
    fn test_create_sharding_manager() {
        let consensus = Arc::new(Mutex::new(Consensus::new()));
        let manager = ShardingManager::new(4, 10, consensus);
        let manager = manager.lock().unwrap();
        assert_eq!(manager.shards.len(), 4);
        assert_eq!(manager.shard_count, 4);
        assert_eq!(manager.nodes_per_shard, 10);
    }

    #[test]
    fn test_assign_node_to_shard() {
        let consensus = Arc::new(Mutex::new(Consensus::new()));
        let manager = ShardingManager::new(4, 2, consensus);
        let mut manager = manager.lock().unwrap();
        let node1 = Node::new("node1", NodeType::PersonalDevice, "127.0.0.1:8000");
        let node2 = Node::new("node2", NodeType::PersonalDevice, "127.0.0.1:8001");
        let node3 = Node::new("node3", NodeType::PersonalDevice, "127.0.0.1:8002");
        
        assert!(manager.assign_node_to_shard(node1, 0).is_ok());
        assert!(manager.assign_node_to_shard(node2, 0).is_ok());
        assert!(manager.assign_node_to_shard(node3, 0).is_err()); // Shard is full
    }

    #[test]
    fn test_cross_shard_transaction() {
        let consensus = Arc::new(Mutex::new(Consensus::new()));
        let manager = ShardingManager::new(2, 10, consensus);
        let mut manager = manager.lock().unwrap();

        // Add some initial balance
        manager.initialize_balance("Alice".to_string(), CurrencyType::BasicNeeds, 1000.0);
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

        // Verify the balances after the transaction
        assert_eq!(manager.get_balance("Alice".to_string(), CurrencyType::BasicNeeds), 900.0);
        assert_eq!(manager.get_balance("Bob".to_string(), CurrencyType::BasicNeeds), 100.0);
    }
}