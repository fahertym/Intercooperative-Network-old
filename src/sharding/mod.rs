use std::error::Error;
use std::collections::HashMap;

use crate::logging::logger::LOGGER;
use crate::{log_info, log_warn, log_error, log_debug};
use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Shard {
    // Define the fields for the Shard struct
    id: usize,
    nodes: Vec<Node>,
    data: HashMap<String, String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Node {
    // Define the fields for the Node struct
    id: String,
    address: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Transaction {
    // Define the fields for the Transaction struct
    from: String,
    to: String,
    amount: f64,
    currency_type: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CurrencyType {
    // Define the fields for the CurrencyType struct
    name: String,
}

pub struct ShardingManager {
    shards: Vec<Shard>,
    nodes_per_shard: usize,
}

impl ShardingManager {
    pub fn new(shard_count: usize, nodes_per_shard: usize) -> Self {
        log_info!("Initializing ShardingManager with {} shards and {} nodes per shard", shard_count, nodes_per_shard);

        let shards = (0..shard_count).map(|id| Shard {
            id,
            nodes: Vec::new(),
            data: HashMap::new(),
        }).collect();

        ShardingManager {
            shards,
            nodes_per_shard,
        }
    }

    pub fn add_node_to_shard(&mut self, node: Node, shard_id: usize) -> Result<(), Box<dyn Error>> {
        let shard = self.shards.get_mut(shard_id).ok_or("Shard not found")?;

        if shard.nodes.len() >= self.nodes_per_shard {
            log_error!("Failed to assign node to shard {}: Shard is full", shard_id);
            return Err("Shard is full".into());
        }

        shard.nodes.push(node);
        log_info!("Assigned node {} to shard {}", node.id, shard_id);
        Ok(())
    }

    pub fn add_address_to_shard(&mut self, address: String, shard_id: usize) {
        let shard = self.shards.get_mut(shard_id).expect("Shard not found");
        shard.data.insert(address.clone(), String::new());
        log_info!("Added address {} to shard {}", address, shard_id);
    }

    pub fn initialize_balance(&mut self, address: String, currency_type: CurrencyType, amount: f64, shard_id: usize) {
        let shard = self.shards.get_mut(shard_id).expect("Shard not found");
        shard.data.insert(address.clone(), format!("{} {}", amount, currency_type.name));
        log_info!("Initialized balance of {} {} for {} in shard {:?}", amount, currency_type.name, address, shard_id);
    }

    pub fn transfer_between_shards(&mut self, from_shard: usize, to_shard: usize, transaction: &Transaction) -> Result<(), Box<dyn Error>> {
        log_info!("Initiating cross-shard transfer from shard {} to shard {}", from_shard, to_shard);

        let from_shard = self.shards.get_mut(from_shard).ok_or("From shard not found")?;
        let to_shard = self.shards.get_mut(to_shard).ok_or("To shard not found")?;

        let from_balance = from_shard.data.get_mut(&transaction.from).ok_or("Sender not found in source shard")?;
        let to_balance = to_shard.data.get_mut(&transaction.to).ok_or("Recipient not found in destination shard")?;

        let from_amount: f64 = from_balance.split_whitespace().next().unwrap().parse()?;
        if from_amount < transaction.amount {
            log_error!("Insufficient balance for {} in shard {:?}", transaction.from, from_shard);
            return Err("Insufficient balance".into());
        }

        let new_from_amount = from_amount - transaction.amount;
        *from_balance = format!("{} {}", new_from_amount, transaction.currency_type);

        let to_amount: f64 = to_balance.split_whitespace().next().unwrap().parse()?;
        let new_to_amount = to_amount + transaction.amount;
        *to_balance = format!("{} {}", new_to_amount, transaction.currency_type);

        log_info!("Cross-shard transfer completed successfully");
        Ok(())
    }

    pub fn get_balance(&self, address: &String, shard_id: usize) -> Option<&String> {
        self.shards.get(shard_id)?.data.get(address)
    }
}
