// src/sharding/mod.rs

use std::collections::HashMap;
use sha2::{Sha256, Digest};
use crate::blockchain::{Block, Transaction};
use crate::network::Node;
use std::sync::{Arc, Mutex};

/// Represents a shard in the network
pub struct Shard {
    pub id: u64,
    pub nodes: Vec<Node>,
    pub blockchain: Vec<Block>,
}

/// Manages the sharding mechanism for the entire network
pub struct ShardingManager {
    shards: HashMap<u64, Arc<Mutex<Shard>>>,
    shard_count: u64,
    nodes_per_shard: usize,
}

impl ShardingManager {
    /// Creates a new ShardingManager
    pub fn new(shard_count: u64, nodes_per_shard: usize) -> Self {
        let mut shards = HashMap::new();
        for i in 0..shard_count {
            shards.insert(i, Arc::new(Mutex::new(Shard {
                id: i,
                nodes: Vec::new(),
                blockchain: Vec::new(),
            })));
        }
        
        ShardingManager {
            shards,
            shard_count,
            nodes_per_shard,
        }
    }

    /// Assigns a node to a shard
    pub fn assign_node_to_shard(&mut self, node: Node, shard_id: u64) -> Result<(), String> {
        let shard = self.shards.get(&shard_id).ok_or(format!("Shard {} not found", shard_id))?;
        let mut shard = shard.lock().unwrap();
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

    /// Handles cross-shard communication
    pub fn cross_shard_communication(&self, from_shard: u64, to_shard: u64, transaction: &Transaction) -> Result<(), String> {
        let from_shard = self.shards.get(&from_shard).ok_or(format!("From shard {} not found", from_shard))?;
        let to_shard = self.shards.get(&to_shard).ok_or(format!("To shard {} not found", to_shard))?;

        let from_shard = from_shard.lock().unwrap();
        let mut to_shard = to_shard.lock().unwrap();

        // Verify the transaction in the from_shard
        if !self.verify_transaction(&from_shard, transaction) {
            return Err("Transaction verification failed in the source shard".to_string());
        }

        // Create a new block with the transaction
        let new_block = Block {
            index: to_shard.blockchain.len() as u64,
            timestamp: chrono::Utc::now().timestamp(),
            transactions: vec![transaction.clone()],
            previous_hash: to_shard.blockchain.last().map(|b| b.hash.clone()).unwrap_or_default(),
            hash: "".to_string(), // This should be properly calculated
            nonce: 0,
            gas_used: 0, // This should be properly calculated
            smart_contract_results: HashMap::new(),
        };

        // Add the new block to the destination shard
        to_shard.blockchain.push(new_block);

        println!("Transaction moved from shard {} to shard {}", from_shard.id, to_shard.id);
        Ok(())
    }

    // Helper function to verify a transaction within a shard
    fn verify_transaction(&self, _shard: &Shard, _transaction: &Transaction) -> bool {
        // This is a placeholder. In a real implementation, we would:
        // 1. Check if the transaction exists in the shard's blockchain
        // 2. Verify the transaction's signature
        // 3. Check if the sender has sufficient balance
        // 4. Ensure the transaction hasn't been double-spent
        true
    }

    // Updated hash_data method using SHA-256
    fn hash_data(&self, data: &[u8]) -> u64 {
        let mut hasher = Sha256::new();
        hasher.update(data);
        let result = hasher.finalize();
        
        // Convert the first 8 bytes of the hash to a u64
        let mut bytes = [0u8; 8];
        bytes.copy_from_slice(&result[..8]);
        u64::from_be_bytes(bytes)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::network::Node;
    use crate::network::network::NodeType;
    use crate::currency::CurrencyType;

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
    fn test_hash_data_distribution() {
        let manager = ShardingManager::new(4, 10);
        let mut shard_counts = [0; 4];
        
        // Generate a large number of test data items
        for i in 0..10000 {
            let data = format!("Test data {}", i).into_bytes();
            let shard = manager.get_shard_for_data(&data);
            shard_counts[shard as usize] += 1;
        }
        
        // Check that the distribution is roughly even
        for count in shard_counts.iter() {
            assert!(*count > 2000 && *count < 3000);
        }
    }

    #[test]
    fn test_cross_shard_communication() {
        let mut manager = ShardingManager::new(2, 10);
        let node1 = Node::new("node1", NodeType::PersonalDevice, "127.0.0.1:8000");
        let node2 = Node::new("node2", NodeType::PersonalDevice, "127.0.0.1:8001");
        
        manager.assign_node_to_shard(node1, 0).unwrap();
        manager.assign_node_to_shard(node2, 1).unwrap();

        let transaction = Transaction::new(
            "Alice".to_string(),
            "Bob".to_string(),
            100.0,
            CurrencyType::BasicNeeds,
            1000,
        );

        assert!(manager.cross_shard_communication(0, 1, &transaction).is_ok());
    }
}