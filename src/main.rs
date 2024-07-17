use log::{info, warn};
use chrono::Utc;
use std::collections::HashMap;
use std::error::Error;

use icn_node::blockchain::Transaction;
use icn_node::consensus::PoCConsensus;
use icn_node::currency::CurrencyType;
use icn_node::governance::DemocraticSystem;
use icn_node::identity::DecentralizedIdentity;
use icn_node::network::Network;
use icn_node::network::node::{Node, NodeType};
use icn_node::vm::CSCLCompiler;
use icn_node::IcnNode;

fn main() -> Result<(), Box<dyn Error>> {
    env_logger::init();
    info!("Starting ICN Node");

    let node = IcnNode::new();
    let mut network = Network::new();
    let mut consensus = PoCConsensus::new(0.5, 0.66);
    let mut democratic_system = DemocraticSystem::new();

    setup_network_and_consensus(&mut network, &mut consensus)?;
    process_initial_transactions(&node)?;
    create_and_vote_on_proposal(&mut democratic_system)?;
    compile_and_run_cscl(&node)?;
    simulate_cross_shard_transaction(&node)?;
    print_final_state(&node, &consensus, &democratic_system);

    info!("ICN Node simulation completed.");
    Ok(())
}

fn setup_network_and_consensus(network: &mut Network, consensus: &mut PoCConsensus) -> Result<(), Box<dyn Error>> {
    let node1 = Node::new("Node1", NodeType::PersonalDevice, "127.0.0.1:8000");
    let node2 = Node::new("Node2", NodeType::PersonalDevice, "127.0.0.1:8001");
    network.add_node(node1);
    network.add_node(node2);

    consensus.add_member("Alice".to_string(), false);
    consensus.add_member("Bob".to_string(), false);
    consensus.add_member("Charlie".to_string(), false);
    consensus.add_member("CorpX".to_string(), true);

    Ok(())
}

fn process_initial_transactions(node: &IcnNode) -> Result<(), Box<dyn Error>> {
    let (alice_did, _) = DecentralizedIdentity::new(HashMap::new());
    let (bob_did, _) = DecentralizedIdentity::new(HashMap::new());

    let tx = Transaction::new(
        alice_did.id.clone(),
        bob_did.id.clone(),
        100.0,
        CurrencyType::BasicNeeds,
        1000
    );

    node.blockchain.write().unwrap().add_transaction(tx)?;
    node.blockchain.write().unwrap().create_block("Alice".to_string())?;

    if let Some(latest_block) = node.blockchain.read().unwrap().get_latest_block() {
        info!("New block created: {:?}", latest_block);
    } else {
        warn!("No blocks in the blockchain to broadcast");
    }

    Ok(())
}

fn create_and_vote_on_proposal(democratic_system: &mut DemocraticSystem) -> Result<(), Box<dyn Error>> {
    let proposal_id = democratic_system.create_proposal(
        "Community Garden".to_string(),
        "Create a community garden in the local park".to_string(),
        "Alice".to_string(),
        chrono::Duration::weeks(1),
        icn_node::governance::ProposalType::Constitutional,
        icn_node::governance::ProposalCategory::Economic,
        0.51,
        Some(Utc::now() + chrono::Duration::days(30)),
    )?;

    democratic_system.vote("Bob".to_string(), proposal_id.clone(), true, 1.0)?;
    democratic_system.vote("Charlie".to_string(), proposal_id.clone(), false, 1.0)?;
    democratic_system.vote("David".to_string(), proposal_id.clone(), true, 1.0)?;
    democratic_system.tally_votes(&proposal_id)?;

    let proposal = democratic_system.get_proposal(&proposal_id)
        .ok_or("Proposal not found after voting")?;
    info!("Proposal status after voting: {:?}", proposal.status);

    Ok(())
}

fn compile_and_run_cscl(node: &IcnNode) -> Result<(), Box<dyn Error>> {
    let cscl_code = r#"
    x = 100 + 50;
    y = 200 - 25;
    z = x * y / 10;
    emit("Result", z);
    "#;

    let mut compiler = CSCLCompiler::new(cscl_code);
    let opcodes = compiler.compile()?;
    
    let mut coop_vm = node.coop_vm.write().unwrap();
    coop_vm.load_program(opcodes);
    coop_vm.run()?;

    Ok(())
}

fn simulate_cross_shard_transaction(node: &IcnNode) -> Result<(), Box<dyn Error>> {
    let mut sharding_manager = node.sharding_manager.write().unwrap();
    sharding_manager.add_address_to_shard("Alice".to_string(), 0);
    sharding_manager.add_address_to_shard("Bob".to_string(), 1);
    sharding_manager.initialize_balance("Alice".to_string(), CurrencyType::BasicNeeds, 1000.0);
    drop(sharding_manager);

    let transaction = Transaction::new(
        "Alice".to_string(),
        "Bob".to_string(),
        500.0,
        CurrencyType::BasicNeeds,
        1000
    );

    node.process_cross_shard_transaction(&transaction)?;

    let sharding_manager = node.sharding_manager.read().unwrap();
    info!("Alice's balance after cross-shard transaction: {}", 
          sharding_manager.get_balance("Alice".to_string(), CurrencyType::BasicNeeds));
    info!("Bob's balance after cross-shard transaction: {}", 
          sharding_manager.get_balance("Bob".to_string(), CurrencyType::BasicNeeds));

    Ok(())
}

fn print_final_state(node: &IcnNode, consensus: &PoCConsensus, democratic_system: &DemocraticSystem) {
    info!("Blockchain state:");
    info!("Number of blocks: {}", node.blockchain.read().unwrap().chain.len());
    if let Some(latest_block) = node.blockchain.read().unwrap().get_latest_block() {
        info!("Latest block hash: {}", latest_block.hash);
    } else {
        warn!("No blocks in the blockchain");
    }

    info!("Consensus state:");
    info!("Number of members: {}", consensus.members.len());
    info!("Current vote threshold: {}", consensus.threshold);

    info!("Democratic system state:");
    info!("Number of active proposals: {}", democratic_system.list_active_proposals().len());

    info!("Sharding state:");
    info!("Number of shards: {}", node.sharding_manager.read().unwrap().get_shard_count());
}


#[cfg(test)]
mod tests {
    use super::*;
    use icn_node::network::{Packet, PacketType};
    use ed25519_dalek::Keypair;
    use rand::rngs::OsRng;

    #[test]
    fn test_icn_node_creation() {
        let node = IcnNode::new();
        assert!(node.content_store.read().unwrap().is_empty());
        assert!(node.pit.read().unwrap().is_empty());
        assert!(node.fib.read().unwrap().is_empty());
        info!("ICN Node creation test passed");
    }

    #[test]
    fn test_packet_processing() {
        let node = IcnNode::new();
        let interest_packet = Packet {
            packet_type: PacketType::Interest,
            name: "test_data".to_string(),
            content: vec![],
        };

        let data_packet = Packet {
            packet_type: PacketType::Data,
            name: "test_data".to_string(),
            content: vec![1, 2, 3, 4],
        };

        node.content_store.write().unwrap().add(data_packet.name.clone(), data_packet.content.clone());
        
        // Simulating packet processing
        if let PacketType::Interest = interest_packet.packet_type {
            let content = node.content_store.read().unwrap().get(&interest_packet.name);
            assert!(content.is_some());
        }

        info!("Packet processing test passed");
    }

    #[test]
    fn test_cross_shard_transaction() {
        let node = IcnNode::new();

        // Initialize balances
        {
            let mut sharding_manager = node.sharding_manager.write().unwrap();
            sharding_manager.add_address_to_shard("Alice".to_string(), 0);
            sharding_manager.add_address_to_shard("Bob".to_string(), 1);
            sharding_manager.initialize_balance("Alice".to_string(), CurrencyType::BasicNeeds, 1000.0);
        }

        let mut csprng = OsRng{};
        let keypair: Keypair = Keypair::generate(&mut csprng);

        let mut transaction = Transaction::new(
            "Alice".to_string(),
            "Bob".to_string(),
            500.0,
            CurrencyType::BasicNeeds,
            1000
        );
        transaction.sign(&keypair).unwrap();

        assert!(node.process_cross_shard_transaction(&transaction).is_ok());

        // Check balances after transaction
        let sharding_manager = node.sharding_manager.read().unwrap();
        assert_eq!(sharding_manager.get_balance("Alice".to_string(), CurrencyType::BasicNeeds), 500.0);
        assert_eq!(sharding_manager.get_balance("Bob".to_string(), CurrencyType::BasicNeeds), 500.0);

        info!("Cross-shard transaction test passed");
    }

    #[test]
    fn test_smart_contract_execution() {
        let node = IcnNode::new();
        assert!(compile_and_run_cscl(&node).is_ok());
        info!("Smart contract execution test passed");
    }

    #[test]
    fn test_democratic_system() {
        let mut democratic_system = DemocraticSystem::new();
        
        let result = create_and_vote_on_proposal(&mut democratic_system);
        assert!(result.is_ok());

        let proposals = democratic_system.list_active_proposals();
        assert_eq!(proposals.len(), 1);

        info!("Democratic system test passed");
    }

    #[test]
    fn test_consensus_mechanism() {
        let mut consensus = PoCConsensus::new(0.5, 0.66);
        
        consensus.add_member("Alice".to_string(), false);
        consensus.add_member("Bob".to_string(), false);
        consensus.add_member("Charlie".to_string(), false);

        assert_eq!(consensus.members.len(), 3);
        assert_eq!(consensus.threshold, 0.5);

        info!("Consensus mechanism test passed");
    }

    #[test]
    fn test_network_setup() {
        let mut network = Network::new();
        let mut consensus = PoCConsensus::new(0.5, 0.66);

        assert!(setup_network_and_consensus(&mut network, &mut consensus).is_ok());
        assert_eq!(network.node_count(), 2);
        assert_eq!(consensus.members.len(), 4);

        info!("Network setup test passed");
    }

    #[test]
    fn test_sharding() {
        let node = IcnNode::new();
        let result = simulate_cross_shard_transaction(&node);
        assert!(result.is_ok());

        let sharding_manager = node.sharding_manager.read().unwrap();
        assert_eq!(sharding_manager.get_balance("Alice".to_string(), CurrencyType::BasicNeeds), 500.0);
        assert_eq!(sharding_manager.get_balance("Bob".to_string(), CurrencyType::BasicNeeds), 500.0);

        info!("Sharding test passed");
    }
}