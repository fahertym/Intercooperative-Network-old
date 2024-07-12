use icn_node::{
    IcnNode, CSCLCompiler, Blockchain, Transaction, PoCConsensus, DemocraticSystem,
    ProposalCategory, ProposalType, DecentralizedIdentity, Network, CoopVM, CurrencyType, Node,
};
use icn_node::network::network::NodeType;
use chrono::Utc;
use std::collections::HashMap;

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
    
    println!("ICN Node simulation completed.");
}