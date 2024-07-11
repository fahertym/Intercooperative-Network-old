use icn_node::{
    IcnNode, CSCLCompiler, Blockchain, Transaction, PoCConsensus, DemocraticSystem,
    ProposalCategory, ProposalType, DecentralizedIdentity, Network, CoopVM, Opcode, Value,
    currency::CurrencyType, network::Node, network::NodeType
};
use chrono::Utc;

fn main() {
    // Initialize the ICN Node
    let mut node = IcnNode::new();

    // Set up the blockchain
    let mut blockchain = Blockchain::new();

    // Set up the consensus mechanism
    let mut consensus = PoCConsensus::new(0.5, 0.66);

    // Set up the democratic system
    let mut democratic_system = DemocraticSystem::new();

    // Set up the network
    let mut network = Network::new();

    // Add some initial members
    consensus.add_member("Alice".to_string());
    consensus.add_member("Bob".to_string());
    consensus.add_member("Charlie".to_string());

    // Create some decentralized identities
    let (alice_did, _) = DecentralizedIdentity::new(std::collections::HashMap::new());
    let (bob_did, _) = DecentralizedIdentity::new(std::collections::HashMap::new());

    // Add some initial transactions
    let tx1 = Transaction::new(
        alice_did.id.clone(),
        bob_did.id.clone(),
        100.0,
        CurrencyType::ICN,
        1000,
    );
    blockchain.add_transaction(tx1);

    // Create a block
    blockchain.create_block("Alice".to_string()).unwrap();

    // Create a proposal
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

    // Vote on the proposal
    democratic_system.vote("Bob".to_string(), proposal_id.clone(), true, 1.0).unwrap();
    democratic_system.vote("Charlie".to_string(), proposal_id.clone(), false, 1.0).unwrap();

    // Tally votes
    democratic_system.tally_votes(&proposal_id).unwrap();

    // Example usage of CSCL compiler
    let cscl_code = r#"
        function calculate_reward(contribution) {
            return contribution * 2;
        }

        initial_contribution = 100;
        reward = calculate_reward(initial_contribution);
        emit("RewardCalculated", reward);

        vote("proposal1", true);
        allocate_resource("computing_power", 50);
        update_reputation("user1", 5);
        
        proposal_id = create_proposal("New community project");
        status = get_proposal_status(proposal_id);
    "#;

    let mut compiler = CSCLCompiler::new(cscl_code);
    let opcodes = compiler.compile().expect("Compilation failed");

    println!("Compiled CSCL code into {} opcodes", opcodes.len());

    // Print all opcodes
    for (i, opcode) in opcodes.iter().enumerate() {
        println!("Opcode {}: {:?}", i, opcode);
    }

    // Create a CoopVM instance and execute the compiled code
    let mut coop_vm = CoopVM::new();
    coop_vm.load_program(opcodes);
    match coop_vm.run() {
        Ok(_) => println!("CoopVM execution completed successfully"),
        Err(e) => println!("CoopVM execution failed: {}", e),
    }

    // Print the final state of the CoopVM
    println!("CoopVM final state:");
    println!("Stack: {:?}", coop_vm.get_stack());
    println!("Memory: {:?}", coop_vm.get_memory());

    // Simulate some network activity
    let node1 = Node::new("Node1", NodeType::PersonalDevice, "127.0.0.1:8000");
    let node2 = Node::new("Node2", NodeType::PersonalDevice, "127.0.0.1:8001");
    network.add_node(node1.clone());
    network.add_node(node2.clone());

    // Broadcast the latest block
    let latest_block = blockchain.get_latest_block().unwrap();
    network.broadcast_block(&latest_block);

    // Print final blockchain state
    println!("Blockchain state:");
    println!("Number of blocks: {}", blockchain.chain.len());
    println!("Latest block hash: {}", blockchain.get_latest_block().unwrap().hash);

    // Print consensus state
    println!("Consensus state:");
    println!("Number of members: {}", consensus.reputation_scores.len());
    println!("Current vote threshold: {}", consensus.vote_threshold);

    // Print democratic system state
    println!("Democratic system state:");
    println!("Number of active proposals: {}", democratic_system.list_active_proposals().len());
    
    println!("ICN Node simulation completed.");
}
