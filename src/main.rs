use std::sync::{Arc, Mutex};
use icn_node::blockchain::{Blockchain, Transaction};
use icn_node::consensus::Consensus;
use icn_node::currency::CurrencyType;
use icn_node::sharding::ShardingManager;
use icn_node::sharding::ShardingManagerTrait;
use icn_node::sharding::cross_shard_transaction_manager::CrossShardTransactionManager;

fn main() {
    println!("Initializing ICN Node...");

    // Initialize consensus
    let consensus = Arc::new(Mutex::new(Consensus::new()));

    // Initialize sharding manager
    let sharding_manager: Arc<Mutex<dyn ShardingManagerTrait + Send + 'static>> = Arc::new(Mutex::new(ShardingManager::new(4, 10, Arc::clone(&consensus))));

    let blockchain = Arc::new(Mutex::new(Blockchain::new(
        Arc::clone(&consensus),
        Arc::clone(&sharding_manager),
    )));

    // Initialize CrossShardTransactionManager with the correct type
    let mut cross_shard_tx_manager = CrossShardTransactionManager::new(
        Arc::clone(&sharding_manager),
        Arc::clone(&consensus)
    );

    // Example: Create and process a cross-shard transaction
    let transaction = Transaction::new(
        "Alice".to_string(),
        "Bob".to_string(),
        100.0,
        CurrencyType::BasicNeeds,
        1000,
    );

    let result = process_cross_shard_transaction(transaction, &mut cross_shard_tx_manager);
    match result {
        Ok(_) => println!("Cross-shard transaction processed successfully"),
        Err(e) => println!("Error processing cross-shard transaction: {}", e),
    }

    // Example: Create a block
    let result = create_block(Arc::clone(&blockchain));
    match result {
        Ok(_) => println!("Block created successfully"),
        Err(e) => println!("Error creating block: {}", e),
    }

    println!("ICN Node initialized and running.");
}

// Process a cross-shard transaction
fn process_cross_shard_transaction(
    transaction: Transaction,
    manager: &mut CrossShardTransactionManager
) -> Result<(), String> {
    let tx_id = manager.initiate_cross_shard_transaction(transaction)?;
    manager.process_cross_shard_transaction(&tx_id)?;
    manager.finalize_cross_shard_transaction(&tx_id)
}

// Create a new block
fn create_block(blockchain: Arc<Mutex<Blockchain>>) -> Result<(), String> {
    let mut blockchain = blockchain.lock().map_err(|_| "Failed to acquire lock on blockchain")?;
    blockchain.create_block()
}