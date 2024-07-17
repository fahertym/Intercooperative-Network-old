use log::{debug, info, warn};
use chrono::Utc;
use std::collections::HashMap;
use std::error::Error;
use std::sync::{Arc, RwLock};

mod blockchain;
mod consensus;
mod currency;
mod governance;
mod identity;
mod network;
mod node;
mod smart_contract;
mod vm;
mod sharding;
mod logging;

use blockchain::{Transaction as BlockchainTransaction, Blockchain};
use consensus::PoCConsensus;
use currency::CurrencyType;
use governance::DemocraticSystem;
use identity::DecentralizedIdentity;
use network::{Network, Packet, PacketType};
use node::{ContentStore, ForwardingInformationBase, PendingInterestTable};
use vm::{CoopVM, Opcode, CSCLCompiler};
use sharding::ShardingManager;

pub struct IcnNode {
    content_store: Arc<RwLock<ContentStore>>,
    pit: Arc<RwLock<PendingInterestTable>>,
    fib: Arc<RwLock<ForwardingInformationBase>>,
    blockchain: Arc<RwLock<Blockchain>>,
    coop_vm: Arc<RwLock<CoopVM>>,
    sharding_manager: Arc<RwLock<ShardingManager>>,
}

impl IcnNode {
    pub fn new() -> Self {
        let blockchain = Arc::new(RwLock::new(Blockchain::new()));
        let coop_vm = Arc::new(RwLock::new(CoopVM::new(Vec::new())));
        let sharding_manager = Arc::new(RwLock::new(ShardingManager::new(4, 10)));

        info!("ICN Node initialized with default configuration");

        IcnNode {
            content_store: Arc::new(RwLock::new(ContentStore::new())),
            pit: Arc::new(RwLock::new(PendingInterestTable::new())),
            fib: Arc::new(RwLock::new(ForwardingInformationBase::new())),
            blockchain,
            coop_vm,
            sharding_manager,
        }
    }

    pub fn process_packet(&self, packet: Packet) -> Result<(), Box<dyn Error>> {
        match packet.packet_type {
            PacketType::Interest => self.process_interest(packet),
            PacketType::Data => self.process_data(packet),
        }
    }

    fn process_interest(&self, packet: Packet) -> Result<(), Box<dyn Error>> {
        let content = self.content_store.write().unwrap().get_and_pop(&packet.name);

        if let Some(_data) = content {
            info!("Sending data for interest: {}", packet.name);
            Ok(())
        } else {
            self.pit.write().unwrap().add_interest(packet.name.clone(), "default_interface");
            info!("Forwarding interest for: {}", packet.name);
            Err(format!("Content '{}' not found", packet.name).into())
        }
    }

    fn process_data(&self, packet: Packet) -> Result<(), Box<dyn Error>> {
        self.content_store.write().unwrap().add(packet.name.clone(), packet.content.clone());

        if let Some(_interfaces) = self.pit.read().unwrap().get_incoming_interfaces(&packet.name) {
            info!("Satisfying pending interests for data: {}", packet.name);
        }
        Ok(())
    }

    pub fn process_cross_shard_transaction(&self, transaction: &BlockchainTransaction) -> Result<(), Box<dyn Error>> {
        let mut sharding_manager = self.sharding_manager.write().unwrap();
        let from_shard = sharding_manager.get_shard_for_address(&transaction.from);
        let to_shard = sharding_manager.get_shard_for_address(&transaction.to);
    
        let sharding_transaction = BlockchainTransaction {
            from: transaction.from.clone(),
            to: transaction.to.clone(),
            amount: transaction.amount,
            currency_type: transaction.currency_type.clone(),
            gas_limit: transaction.gas_limit,
            smart_contract_id: transaction.smart_contract_id.clone(),
            signature: transaction.signature.clone(),
            public_key: transaction.public_key.clone(),
        };
    
        if from_shard != to_shard {
            sharding_manager.transfer_between_shards(from_shard, to_shard, &sharding_transaction)?;
            info!("Cross-shard transaction processed successfully");
        } else {
            info!("Transaction is within the same shard");
        }
    
        Ok(())
    }

    pub fn execute_smart_contract(&self, contract: String) -> Result<(), Box<dyn Error>> {
        let mut coop_vm = self.coop_vm.write().unwrap();
        let opcodes = self.compile_contract(&contract)?;
        coop_vm.load_program(opcodes);
        coop_vm.run()?;
        info!("Smart contract executed successfully");
        Ok(())
    }

    fn compile_contract(&self, contract: &str) -> Result<Vec<Opcode>, Box<dyn Error>> {
        let mut compiler = CSCLCompiler::new(contract);
        compiler.compile()
    }
}

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
    print_final_state(&node, &consensus, &democratic_system);

    info!("ICN Node simulation completed.");
    Ok(())
}

fn setup_network_and_consensus(network: &mut Network, consensus: &mut PoCConsensus) -> Result<(), Box<dyn Error>> {
    let _node1 = network::node::Node::new("Node1", network::node::NodeType::PersonalDevice, "127.0.0.1:8000");
    let _node2 = network::node::Node::new("Node2", network::node::NodeType::PersonalDevice, "127.0.0.1:8001");
    network.add_node(network::network::Node { id: "Node1".to_string(), node_type: network::node::NodeType::PersonalDevice, address: "127.0.0.1:8000".to_string() });
    network.add_node(network::network::Node { id: "Node2".to_string(), node_type: network::node::NodeType::PersonalDevice, address: "127.0.0.1:8001".to_string() });

    consensus.add_member("Alice".to_string(), false);
    consensus.add_member("Bob".to_string(), false);
    consensus.add_member("Charlie".to_string(), false);
    consensus.add_member("CorpX".to_string(), true);

    Ok(())
}

fn process_initial_transactions(node: &IcnNode) -> Result<(), Box<dyn Error>> {
    let (alice_did, _) = DecentralizedIdentity::new(HashMap::new());
    let (bob_did, _) = DecentralizedIdentity::new(HashMap::new());

    let tx = BlockchainTransaction::new(
        alice_did.id.clone(),
        bob_did.id.clone(),
        100.0,
        CurrencyType::BasicNeeds,
        1000
    );

    node.blockchain.write().unwrap().add_transaction(tx);
    node.blockchain.write().unwrap().create_block("Alice".to_string())?;

    if let Some(_latest_block) = node.blockchain.read().unwrap().get_latest_block() {
        // Assuming Network has a broadcast_block method
        // network.broadcast_block(&latest_block);
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
        governance::ProposalType::Constitutional,
        governance::ProposalCategory::Economic,
        0.51,
        Some(Utc::now() + chrono::Duration::days(30)),
    );

    democratic_system.vote("Bob".to_string(), proposal_id.clone(), true, 1.0)?;
    democratic_system.vote("Charlie".to_string(), proposal_id.clone(), false, 1.0)?;
    democratic_system.tally_votes(&proposal_id)?;

    Ok(())
}

fn compile_and_run_cscl(_node: &IcnNode) -> Result<(), Box<dyn Error>> {
    let cscl_code = r#"
    x = 100 + 50;
    y = 200 - 25;
    z = x * y / 10;
    emit("Result", z);
    "#;

    let mut compiler = CSCLCompiler::new(cscl_code);
    let opcodes = compiler.compile()?;

    info!("Compiled CSCL code into {} opcodes", opcodes.len());
    for (i, opcode) in opcodes.iter().enumerate() {
        debug!("Opcode {}: {:?}", i, opcode);
    }

    let mut coop_vm = CoopVM::new(opcodes);
    coop_vm.run()?;

    debug!("CoopVM final state:");
    debug!("Stack: {:?}", coop_vm.get_stack());
    debug!("Memory: {:?}", coop_vm.get_memory());

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
}

#[cfg(test)]
mod tests {
    use super::*;

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

        assert!(node.process_packet(interest_packet.clone()).is_err());

        let data_packet = Packet {
            packet_type: PacketType::Data,
            name: "test_data".to_string(),
            content: vec![1, 2, 3, 4],
        };

        assert!(node.process_packet(data_packet).is_ok());

        let interest_packet = Packet {
            packet_type: PacketType::Interest,
            name: "test_data".to_string(),
            content: vec![],
        };

        assert!(node.process_packet(interest_packet).is_ok());
        info!("Packet processing test passed");
    }

    #[test]
    fn test_cross_shard_transaction() {
        let node = IcnNode::new();

        // Initialize balances
        node.sharding_manager.write().unwrap().add_address_to_shard("Alice".to_string(), 0);
        node.sharding_manager.write().unwrap().add_address_to_shard("Bob".to_string(), 1);
        node.sharding_manager.write().unwrap().initialize_balance("Alice".to_string(), CurrencyType::BasicNeeds, 1000.0);

        let transaction = BlockchainTransaction::new(
            "Alice".to_string(),
            "Bob".to_string(),
            500.0,
            CurrencyType::BasicNeeds,
            1000
        );

        assert!(node.process_cross_shard_transaction(&transaction).is_ok());

        // Check balances after transaction
        assert_eq!(node.sharding_manager.read().unwrap().get_balance("Alice".to_string(), CurrencyType::BasicNeeds), 500.0);
        assert_eq!(node.sharding_manager.read().unwrap().get_balance("Bob".to_string(), CurrencyType::BasicNeeds), 500.0);
        info!("Cross-shard transaction test passed");
    }
}
