// File: src/sharding/mod.rs

use std::collections::HashMap;
use sha2::{Sha256, Digest};
use crate::blockchain::{Block, Transaction};
use crate::network::Node;
use crate::currency::CurrencyType;

/// Represents a shard in the network
pub struct Shard {
    pub id: u64,
    pub nodes: Vec<Node>,
    pub blockchain: Vec<Block>,
    pub balances: HashMap<String, HashMap<CurrencyType, f64>>,
}

/// Manages the sharding mechanism for the entire network
pub struct ShardingManager {
    shards: HashMap<u64, Shard>,
    shard_count: u64,
    nodes_per_shard: usize,
    address_to_shard: HashMap<String, u64>,
}

impl ShardingManager {
    /// Creates a new ShardingManager
    pub fn new(shard_count: u64, nodes_per_shard: usize) -> Self {
        let mut shards = HashMap::new();
        for i in 0..shard_count {
            shards.insert(i, Shard {
                id: i,
                nodes: Vec::new(),
                blockchain: Vec::new(),
                balances: HashMap::new(),
            });
        }

        ShardingManager {
            shards,
            shard_count,
            nodes_per_shard,
            address_to_shard: HashMap::new(),
        }
    }

    /// Assigns a node to a shard
    pub fn assign_node_to_shard(&mut self, node: Node, shard_id: u64) -> Result<(), String> {
        let shard = self.shards.get_mut(&shard_id).ok_or(format!("Shard {} not found", shard_id))?;
        if shard.nodes.len() >= self.nodes_per_shard {
            return Err(format!("Shard {} is full", shard_id));
        }
        shard.nodes.push(node);
        Ok(())
    }

    /// Gets the shard for a given transaction or block
    pub fn get_shard_for_data(&self, data: &[u8]) -> u64 {
        let hash = self.hash_data(data);
        hash % self.shard_count
    }

    /// Gets the shard for a given address
    pub fn get_shard_for_address(&self, address: &str) -> u64 {
        *self.address_to_shard.get(address).unwrap_or(&(self.hash_data(address.as_bytes()) % self.shard_count))
    }

    /// Handles cross-shard communication
    pub fn transfer_between_shards(&mut self, from_shard: u64, to_shard: u64, transaction: &Transaction) -> Result<(), String> {
        // First, check if both shards exist
        if !self.shards.contains_key(&from_shard) {
            return Err(format!("From shard {} not found", from_shard));
        }
        if !self.shards.contains_key(&to_shard) {
            return Err(format!("To shard {} not found", to_shard));
        }

        // Verify and execute the transaction in the source shard
        {
            let from_shard = self.shards.get_mut(&from_shard).unwrap();
            if let Some(sender_balance) = from_shard.balances.get_mut(&transaction.from) {
                if let Some(balance) = sender_balance.get_mut(&transaction.currency_type) {
                    if *balance >= transaction.amount {
                        *balance -= transaction.amount;
                    } else {
                        return Err("Insufficient balance".to_string());
                    }
                } else {
                    return Err("Sender doesn't have the required currency type".to_string());
                }
            } else {
                return Err("Sender not found in source shard".to_string());
            }
        }

        // Add the amount to the recipient in the destination shard
        {
            let to_shard = self.shards.get_mut(&to_shard).unwrap();
            to_shard.balances
                .entry(transaction.to.clone())
                .or_insert_with(HashMap::new)
                .entry(transaction.currency_type.clone())
                .and_modify(|balance| *balance += transaction.amount)
                .or_insert(transaction.amount);
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

    pub fn add_address_to_shard(&mut self, address: String, shard_id: u64) {
        self.address_to_shard.insert(address, shard_id);
    }

    pub fn initialize_balance(&mut self, address: String, currency_type: CurrencyType, amount: f64) {
        let shard_id = self.get_shard_for_address(&address);
        if let Some(shard) = self.shards.get_mut(&shard_id) {
            shard.balances
                .entry(address)
                .or_insert_with(HashMap::new)
                .insert(currency_type, amount);
        }
    }

    pub fn get_balance(&self, address: String, currency_type: CurrencyType) -> f64 {
        let shard_id = self.get_shard_for_address(&address);
        if let Some(shard) = self.shards.get(&shard_id) {
            shard.balances
                .get(&address)
                .and_then(|balances| balances.get(&currency_type))
                .cloned()
                .unwrap_or(0.0)
        } else {
            0.0
        }
    }

    pub fn get_shard_count(&self) -> u64 {
        self.shard_count
    }

    pub fn get_nodes_per_shard(&self) -> usize {
        self.nodes_per_shard
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::network::network::NodeType;

    #[test]
    fn test_create_sharding_manager() {
        let manager = ShardingManager::new(4, 10);
        assert_eq!(manager.shards.len(), 4);
        assert_eq!(manager.shard_count, 4);
        assert_eq!(manager.nodes_per_shard, 10);
    }

    #[test]
    fn test_assign_node_to_shard() {
        let mut manager = ShardingManager::new(4, 2);
        let node1 = Node::new("node1", NodeType::PersonalDevice, "127.0.0.1:8000");
        let node2 = Node::new("node2", NodeType::PersonalDevice, "127.0.0.1:8001");
        let node3 = Node::new("node3", NodeType::PersonalDevice, "127.0.0.1:8002");

        assert!(manager.assign_node_to_shard(node1, 0).is_ok());
        assert!(manager.assign_node_to_shard(node2, 0).is_ok());
        assert!(manager.assign_node_to_shard(node3, 0).is_err()); // Shard is full
    }

    #[test]
    fn test_get_shard_for_data() {
        let manager = ShardingManager::new(4, 10);
        let data1 = b"Transaction1";
        let data2 = b"Transaction2";

        assert!(manager.get_shard_for_data(data1) < 4);
        assert!(manager.get_shard_for_data(data2) < 4);
    }

    #[test]
    fn test_cross_shard_communication() {
        let mut manager = ShardingManager::new(2, 10);
        let node1 = Node::new("node1", NodeType::PersonalDevice, "127.0.0.1:8000");
        let node2 = Node::new("node2", NodeType::PersonalDevice, "127.0.0.1:8001");

        manager.assign_node_to_shard(node1, 0).unwrap();
        manager.assign_node_to_shard(node2, 1).unwrap();

        manager.initialize_balance("Alice".to_string(), CurrencyType::BasicNeeds, 100.0);

        let transaction = Transaction::new(
            "Alice".to_string(),
            "Bob".to_string(),
            50.0,
            CurrencyType::BasicNeeds,
            1000,
        );

        let from_shard = manager.get_shard_for_address(&transaction.from);
        let to_shard = manager.get_shard_for_address(&transaction.to);

        assert!(manager.transfer_between_shards(from_shard, to_shard, &transaction).is_ok());

        assert_eq!(manager.get_balance("Alice".to_string(), CurrencyType::BasicNeeds), 50.0);
        assert_eq!(manager.get_balance("Bob".to_string(), CurrencyType::BasicNeeds), 50.0);
    }
}