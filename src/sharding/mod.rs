use std::collections::HashMap;
use sha2::{Sha256, Digest};
use crate::blockchain::Block;
use crate::blockchain::Transaction;
use crate::network::Node;
use crate::currency::CurrencyType;
use std::sync::{Arc, Mutex};
use ed25519_dalek::{PublicKey, Signature, Verifier};
use log::{info, error, warn, debug};

pub mod cross_shard_communication;

/// Represents a shard in the network
pub struct Shard {
    pub id: u64,
    pub nodes: Vec<Node>,
    pub blockchain: Vec<Block>,
    pub balances: HashMap<String, HashMap<CurrencyType, f64>>,
}

/// Manages the sharding mechanism for the entire network
pub struct ShardingManager {
    shards: HashMap<u64, Arc<Mutex<Shard>>>,
    shard_count: u64,
    nodes_per_shard: usize,
    address_to_shard: HashMap<String, u64>,
    current_shard_id: u64,
}

impl ShardingManager {
    pub fn process_transaction(&mut self, shard_id: usize, transaction: &Transaction) -> Result<(), String> {
        self.withdraw(&transaction.from, &transaction.currency_type, transaction.amount, shard_id)?;
        self.deposit(&transaction.to, &transaction.currency_type, transaction.amount, shard_id)?;
        Ok(())
    }

    fn withdraw(&mut self, _from: &str, _currency_type: &CurrencyType, _amount: f64, _shard_id: usize) -> Result<(), String> {
        // Implement the logic for withdrawing funds from the specified account in the specified shard.
        // Return Ok(()) if successful, or an error message if there was a problem.
        // Example implementation:
        Ok(())
    }

    fn deposit(&mut self, _to: &str, _currency_type: &CurrencyType, _amount: f64, _shard_id: usize) -> Result<(), String> {
        // Implement the logic for depositing funds into the specified account in the specified shard.
        // Return Ok(()) if successful, or an error message if there was a problem.
        // Example implementation:
        Ok(())
    }

    pub fn add_balance(&mut self, address: &str, currency_type: CurrencyType, amount: f64) -> Result<(), String> {
        let shard_id = self.get_shard_for_address(address);
        if let Some(shard) = self.shards.get_mut(&shard_id) {
            let mut shard = shard.lock().unwrap();
            let balance = shard.balances
                .entry(address.to_string())
                .or_insert_with(HashMap::new)
                .entry(currency_type.clone())
                .or_insert(0.0);
            *balance += amount;
            info!("Added balance of {} {} for address {} in shard {}", amount, currency_type, address, shard_id);
            Ok(())
        } else {
            error!("Shard {} not found", shard_id);
            Err(format!("Shard {} not found", shard_id))
        }
    }

    /// Creates a new ShardingManager
    pub fn new(shard_count: u64, nodes_per_shard: usize) -> Self {
        let mut shards = HashMap::new();
        for i in 0..shard_count {
            shards.insert(i, Arc::new(Mutex::new(Shard {
                id: i,
                nodes: Vec::new(),
                blockchain: Vec::new(),
                balances: HashMap::new(),
            })));
        }
        
        info!("Created new ShardingManager with {} shards and {} nodes per shard", shard_count, nodes_per_shard);
        ShardingManager {
            shards,
            shard_count,
            nodes_per_shard,
            address_to_shard: HashMap::new(),
            current_shard_id: 0,
        }
    }

    /// Assigns a node to a shard
    pub fn assign_node_to_shard(&mut self, node: Node, shard_id: u64) -> Result<(), String> {
        let shard = self.shards.get(&shard_id).ok_or(format!("Shard {} not found", shard_id))?;
        let mut shard = shard.lock().unwrap();
        if shard.nodes.len() >= self.nodes_per_shard {
            error!("Failed to assign node to shard {}: Shard is full", shard_id);
            return Err(format!("Shard {} is full", shard_id));
        }
        shard.nodes.push(node.clone());
        info!("Assigned node {} to shard {}", node.id, shard_id);
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

    /// Gets the current shard ID
    pub fn get_current_shard_id(&self) -> u64 {
        self.current_shard_id
    }

    /// Adds an address to a specific shard
    pub fn add_address_to_shard(&mut self, address: String, shard_id: u64) {
        self.address_to_shard.insert(address.clone(), shard_id);
        info!("Added address {} to shard {}", address, shard_id);
    }

    /// Initialize balance for an address
    pub fn initialize_balance(&mut self, address: String, currency_type: CurrencyType, amount: f64) {
        let shard_id = self.get_shard_for_address(&address);
        if let Some(shard) = self.shards.get_mut(&shard_id) {
            let mut shard = shard.lock().unwrap();
            shard.balances
                .entry(address.clone())
                .or_insert_with(HashMap::new)
                .insert(currency_type.clone(), amount);
            info!("Initialized balance of {} {} for {} in shard {}", amount, currency_type, address, shard_id);
        }
    }

    /// Get balance for an address
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

    /// Handles cross-shard communication
    pub fn transfer_between_shards(&mut self, from_shard: u64, to_shard: u64, transaction: &Transaction) -> Result<(), String> {
        let from_shard = self.shards.get(&from_shard).ok_or(format!("From shard {} not found", from_shard))?;
        let to_shard = self.shards.get(&to_shard).ok_or(format!("To shard {} not found", to_shard))?;

        let mut from_shard = from_shard.lock().unwrap();
        let mut to_shard = to_shard.lock().unwrap();

        debug!("Verifying transaction in source shard: {}", from_shard.id);
        if !self.verify_transaction(&from_shard, transaction) {
            error!("Transaction verification failed in the source shard");
            return Err("Transaction verification failed in the source shard".to_string());
        }

        debug!("Updating balances in source shard: {}", from_shard.id);
        if let Err(e) = self.update_balances(&mut from_shard, transaction) {
            error!("Failed to update balances in source shard: {}", e);
            return Err(e);
        }

        debug!("Creating new block in destination shard: {}", to_shard.id);
        let new_block = Block {
            index: to_shard.blockchain.len() as u64,
            timestamp: chrono::Utc::now().timestamp(),
            transactions: vec![transaction.clone()],
            previous_hash: to_shard.blockchain.last().map(|b| b.hash.clone()).unwrap_or_default(),
            hash: self.calculate_block_hash(&to_shard.blockchain, transaction),
            nonce: 0, // In a real implementation, this would be calculated
            gas_used: 0, // This should be properly calculated
            smart_contract_results: HashMap::new(),
        };

        to_shard.blockchain.push(new_block);

        debug!("Adding balances in destination shard: {}", to_shard.id);
        if let Err(e) = self.add_balances(&mut to_shard, transaction) {
            error!("Failed to add balances: {}", e);
            return Err(e);
        }

        info!("Transaction moved from shard {} to shard {}", from_shard.id, to_shard.id);
        Ok(())
    }

    fn verify_transaction(&self, shard: &Shard, transaction: &Transaction) -> bool {
        debug!("Checking balance for sender: {}", transaction.from);
        if let Some(sender_balances) = shard.balances.get(&transaction.from) {
            if let Some(balance) = sender_balances.get(&transaction.currency_type) {
                if *balance < transaction.amount {
                    warn!("Insufficient balance for sender: {}", transaction.from);
                    return false; // Insufficient balance
                }
            } else {
                warn!("Sender does not have the required currency type");
                return false; // Sender doesn't have the required currency type
            }
        } else {
            warn!("Sender not found in this shard");
            return false; // Sender not found in this shard
        }

        debug!("Verifying transaction signature");
        if let (Some(public_key), Some(signature)) = (&transaction.public_key, &transaction.signature) {
            let public_key = PublicKey::from_bytes(public_key).unwrap();
            let signature = Signature::from_bytes(signature).unwrap();
            let message = transaction.to_bytes();
            if public_key.verify(&message, &signature).is_err() {
                warn!("Signature verification failed");
                return false; // Signature verification failed
            }
        } else {
            warn!("Missing public key or signature");
            return false; // Missing public key or signature
        }

        true // All checks passed
    }

    fn update_balances(&self, shard: &mut Shard, transaction: &Transaction) -> Result<(), String> {
        debug!("Updating balances for transaction from {} to {} of amount {}", transaction.from, transaction.to, transaction.amount);
        
        // Deduct from sender
        let sender_balances = shard.balances.entry(transaction.from.clone()).or_insert_with(HashMap::new);
        let sender_balance = sender_balances.entry(transaction.currency_type.clone()).or_insert(0.0);
        debug!("Sender initial balance: {}", sender_balance);
        if *sender_balance < transaction.amount {
            warn!("Insufficient balance for sender: {}", sender_balance);
            return Err("Insufficient balance".to_string());
        }
        *sender_balance -= transaction.amount;
        debug!("Sender balance after deduction: {}", sender_balance);

        Ok(())
    }

    fn add_balances(&self, shard: &mut Shard, transaction: &Transaction) -> Result<(), String> {
        debug!("Adding balances for transaction from {} to {} of amount {}", transaction.from, transaction.to, transaction.amount);

        // Add to recipient
        let recipient_balances = shard.balances.entry(transaction.to.clone()).or_insert_with(HashMap::new);
        let recipient_balance = recipient_balances.entry(transaction.currency_type.clone()).or_insert(0.0);
        debug!("Recipient initial balance: {}", recipient_balance);
        *recipient_balance += transaction.amount;
        debug!("Recipient balance after addition: {}", recipient_balance);

        Ok(())
    }

    fn calculate_block_hash(&self, blockchain: &[Block], transaction: &Transaction) -> String {
        let mut hasher = Sha256::new();
        if let Some(last_block) = blockchain.last() {
            hasher.update(&last_block.hash);
        }
        hasher.update(transaction.to_bytes());
        let result = hasher.finalize();
        hex::encode(result)
    }

    fn hash_data(&self, data: &[u8]) -> u64 {
        let mut hasher = Sha256::new();
        hasher.update(data);
        let result = hasher.finalize();
        
        let mut bytes = [0u8; 8];
        bytes.copy_from_slice(&result[..8]);
        u64::from_be_bytes(bytes)
    }

    // Set the current shard ID (for testing purposes)
    pub fn set_current_shard_id(&mut self, shard_id: u64) {
        self.current_shard_id = shard_id;
        info!("Set current shard ID to {}", shard_id);
    }
}

// ... (previous code remains the same)

#[cfg(test)]
mod tests {
    use super::*;
    use crate::network::Node;
    use crate::network::node::NodeType;
    use crate::currency::CurrencyType;
    use ed25519_dalek::{Keypair, Signer};

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
        
        for i in 0..10000 {
            let data = format!("Test data {}", i).into_bytes();
            let shard = manager.get_shard_for_data(&data);
            shard_counts[shard as usize] += 1;
        }
        
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

        let mut csprng = rand::rngs::OsRng{};
        let keypair: Keypair = Keypair::generate(&mut csprng);

        let mut transaction = Transaction::new(
            "Alice".to_string(),
            "Bob".to_string(),
            100.0,
            CurrencyType::BasicNeeds,
            1000,
        );

        let message = transaction.to_bytes();
        let signature = keypair.sign(&message);
        transaction.signature = Some(signature.to_bytes().to_vec());
        transaction.public_key = Some(keypair.public.to_bytes().to_vec());

        manager.add_address_to_shard("Alice".to_string(), 0);
        manager.add_address_to_shard("Bob".to_string(), 1);

        // Initialize Alice's balance
        manager.initialize_balance("Alice".to_string(), CurrencyType::BasicNeeds, 1000.0);

        assert!(manager.transfer_between_shards(0, 1, &transaction).is_ok());

        // Verify balances after transfer
        let alice_balance = manager.get_balance("Alice".to_string(), CurrencyType::BasicNeeds);
        assert_eq!(alice_balance, 900.0);

        let bob_balance = manager.get_balance("Bob".to_string(), CurrencyType::BasicNeeds);
        assert_eq!(bob_balance, 100.0);
    }
}