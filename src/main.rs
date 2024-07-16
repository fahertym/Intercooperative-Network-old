// File: src/main.rs

use icn_node::{
    IcnNode, CSCLCompiler, Blockchain, Transaction, PoCConsensus, DemocraticSystem,
    ProposalCategory, ProposalType, DecentralizedIdentity, Network, CoopVM, CurrencyType,
    Node, ShardingManager
};
use icn_node::network::network::NodeType;
use chrono::Utc;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

fn main() {
    // Initialize the ICN Node with all its components
    let _node = IcnNode::new();

    // Initialize the blockchain
    let mut blockchain = Blockchain::new();

    // Set up the Proof of Contribution consensus mechanism
    let mut consensus = PoCConsensus::new(0.5, 0.66);

    // Set up the democratic system for governance
    let mut democratic_system = DemocraticSystem::new();

    // Initialize the network
    let mut network = Network::new();

    // Initialize the sharding manager
    let sharding_manager = Arc::new(Mutex::new(ShardingManager::new(4, 10))); // 4 shards, 10 nodes per shard

    // Add initial members to the consensus mechanism
    consensus.consensus.add_member("Alice".to_string());
    consensus.consensus.add_member("Bob".to_string());
    consensus.consensus.add_member("Charlie".to_string());

    // Create decentralized identities for participants
    let (alice_did, _) = DecentralizedIdentity::new(HashMap::new());
    let (bob_did, _) = DecentralizedIdentity::new(HashMap::new());

    // Add initial transactions to the blockchain
    let tx1 = Transaction::new(
        alice_did.id.clone(),
        bob_did.id.clone(),
        100.0,
        CurrencyType::BasicNeeds,
        1000,
    );
    blockchain.add_transaction(tx1);

    // Create a block containing the initial transactions
    if let Err(e) = blockchain.create_block("Alice".to_string()) {
        println!("Error creating block: {}", e);
    }

    // Create a proposal for the democratic system
    let proposal_id = democratic_system.create_proposal(
        "Community Garden".to_string(),
        "Create a community garden in the local park".to_string(),
        "Alice".to_string(),
        chrono::Duration::weeks(1), // 1 week voting period
        ProposalType::Constitutional,
        ProposalCategory::Economic,
        0.51, // 51% quorum
        Some(Utc::now() + chrono::Duration::days(30)), // Execute in 30 days if passed
    );

    // Participants vote on the proposal
    if let Err(e) = democratic_system.vote("Bob".to_string(), proposal_id.clone(), true, 1.0) {
        println!("Error voting on proposal: {}", e);
    }
    if let Err(e) = democratic_system.vote("Charlie".to_string(), proposal_id.clone(), false, 1.0) {
        println!("Error voting on proposal: {}", e);
    }

    // Tally votes for the proposal
    if let Err(e) = democratic_system.tally_votes(&proposal_id) {
        println!("Error tallying votes: {}", e);
    }

    // Example usage of CSCL compiler
    let cscl_code = r#"
        x = 100 + 50;
        y = 200 - 25;
        z = x * y / 10;
        emit("Result", z);
    "#;
    let mut compiler = CSCLCompiler::new(cscl_code);
    match compiler.compile() {
        Ok(opcodes) => {
            println!("Compiled CSCL code into {} opcodes", opcodes.len());
            // Print all compiled opcodes
            for (i, opcode) in opcodes.iter().enumerate() {
                println!("Opcode {}: {:?}", i, opcode);
            }

            // Create a CoopVM instance and execute the compiled code
            let mut coop_vm = CoopVM::new(opcodes);
            match coop_vm.run() {
                Ok(_) => println!("CoopVM execution completed successfully"),
                Err(e) => println!("CoopVM execution failed: {}", e),
            }

            // Print the final state of the CoopVM
            println!("CoopVM final state:");
            println!("Stack: {:?}", coop_vm.get_stack());
            println!("Memory: {:?}", coop_vm.get_memory());
        },
        Err(e) => println!("Compilation failed: {}", e),
    }

    // Simulate some network activity
    let node1 = Node::new("Node1", NodeType::PersonalDevice, "127.0.0.1:8000");
    let node2 = Node::new("Node2", NodeType::PersonalDevice, "127.0.0.1:8001");
    network.add_node(node1.clone());
    network.add_node(node2.clone());

    // Assign nodes to shards
    {
        let mut sharding_manager = sharding_manager.lock().unwrap();
        if let Err(e) = sharding_manager.assign_node_to_shard(node1.clone(), 0) {
            println!("Error assigning node to shard: {}", e);
        }
        if let Err(e) = sharding_manager.assign_node_to_shard(node2.clone(), 1) {
            println!("Error assigning node to shard: {}", e);
        }
    }

    // Example of cross-shard transaction
    let cross_shard_tx = Transaction::new(
        "Alice".to_string(),
        "Bob".to_string(),
        50.0,
        CurrencyType::BasicNeeds,
        1000,
    );

    // Initialize balances for Alice and Bob
    {
        let mut sharding_manager = sharding_manager.lock().unwrap();
        sharding_manager.add_address_to_shard("Alice".to_string(), 0);
        sharding_manager.add_address_to_shard("Bob".to_string(), 1);
        sharding_manager.initialize_balance("Alice".to_string(), CurrencyType::BasicNeeds, 1000.0);
    }

    {
        let mut sharding_manager = sharding_manager.lock().unwrap();
        let from_shard = sharding_manager.get_shard_for_address(&cross_shard_tx.from);
        let to_shard = sharding_manager.get_shard_for_address(&cross_shard_tx.to);

        if from_shard != to_shard {
            match sharding_manager.transfer_between_shards(from_shard, to_shard, &cross_shard_tx) {
                Ok(_) => println!("Cross-shard transaction successful"),
                Err(e) => println!("Cross-shard transaction failed: {}", e),
            }
        }
    }

    // Print final balances
    {
        let sharding_manager = sharding_manager.lock().unwrap();
        let alice_balance = sharding_manager.get_balance("Alice".to_string(), CurrencyType::BasicNeeds);
        let bob_balance = sharding_manager.get_balance("Bob".to_string(), CurrencyType::BasicNeeds);
        println!("Final balances:");
        println!("Alice: {} BasicNeeds", alice_balance);
        println!("Bob: {} BasicNeeds", bob_balance);
    }

    // Broadcast the latest block
    if let Some(latest_block) = blockchain.get_latest_block() {
        network.broadcast_block(&latest_block);
    } else {
        println!("No blocks in the blockchain to broadcast");
    }

    // Print final blockchain state
    println!("Blockchain state:");
    println!("Number of blocks: {}", blockchain.chain.len());
    if let Some(latest_block) = blockchain.get_latest_block() {
        println!("Latest block hash: {}", latest_block.hash);
    } else {
        println!("No blocks in the blockchain");
    }

    // Print consensus state
    println!("Consensus state:");
    println!("Number of members: {}", consensus.consensus.members.len());
    println!("Current vote threshold: {}", consensus.vote_threshold);

    // Print democratic system state
    println!("Democratic system state:");
    println!("Number of active proposals: {}", democratic_system.list_active_proposals().len());

    // Print sharding state
    {
        let sharding_manager = sharding_manager.lock().unwrap();
        println!("Sharding state:");
        println!("Number of shards: {}", sharding_manager.get_shard_count());
        println!("Nodes per shard: {}", sharding_manager.get_nodes_per_shard());
    }

    println!("ICN Node simulation completed.");
}