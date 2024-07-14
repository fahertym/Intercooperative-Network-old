use std::sync::{Arc, Mutex};
use std::io::{self, Write};
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

    // Run the CLI
    run_cli(blockchain, cross_shard_tx_manager);

    println!("ICN Node initialized and running.");
}

fn run_cli(blockchain: Arc<Mutex<Blockchain>>, mut cross_shard_tx_manager: CrossShardTransactionManager) {
    loop {
        print_menu();
        let choice = get_user_input("Enter your choice: ");

        match choice.trim() {
            "1" => add_transaction(blockchain.clone(), &mut cross_shard_tx_manager),
            "2" => create_block(blockchain.clone()),
            "3" => view_blockchain_state(blockchain.clone()),
            "4" => break,
            _ => println!("Invalid choice. Please try again."),
        }
    }
}

fn print_menu() {
    println!("\n--- ICN Node CLI ---");
    println!("1. Add a transaction");
    println!("2. Create a block");
    println!("3. View blockchain state");
    println!("4. Exit");
}

fn get_user_input(prompt: &str) -> String {
    print!("{}", prompt);
    io::stdout().flush().unwrap();
    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();
    input
}

fn add_transaction(blockchain: Arc<Mutex<Blockchain>>, cross_shard_tx_manager: &mut CrossShardTransactionManager) {
    let from = get_user_input("Enter sender address: ");
    let to = get_user_input("Enter recipient address: ");
    let amount: f64 = get_user_input("Enter amount: ").trim().parse().unwrap();
    let currency_type = CurrencyType::BasicNeeds; // For simplicity, we're using a fixed currency type here.
    let gas_limit: u64 = get_user_input("Enter gas limit: ").trim().parse().unwrap();

    let transaction = Transaction::new(from.trim().to_string(), to.trim().to_string(), amount, currency_type, gas_limit);
    let result = process_cross_shard_transaction(transaction, cross_shard_tx_manager);

    match result {
        Ok(_) => println!("Transaction added successfully"),
        Err(e) => println!("Error adding transaction: {}", e),
    }
}

fn create_block(blockchain: Arc<Mutex<Blockchain>>) {
    let result = {
        let mut blockchain = blockchain.lock().unwrap();
        blockchain.create_block()
    };
    match result {
        Ok(_) => println!("Block created successfully"),
        Err(e) => println!("Error creating block: {}", e),
    }
}

fn view_blockchain_state(blockchain: Arc<Mutex<Blockchain>>) {
    let blockchain = blockchain.lock().unwrap();
    println!("Blockchain state:");
    println!("Number of blocks: {}", blockchain.chain.len());
    println!("Latest block: {:?}", blockchain.chain.last());
}

fn process_cross_shard_transaction(
    transaction: Transaction,
    manager: &mut CrossShardTransactionManager
) -> Result<(), String> {
    let tx_id = manager.initiate_cross_shard_transaction(transaction)?;
    manager.process_cross_shard_transaction(&tx_id)?;
    manager.finalize_cross_shard_transaction(&tx_id)
}
