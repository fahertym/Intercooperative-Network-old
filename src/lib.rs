use std::sync::{Arc, Mutex};
use std::error::Error;
use crate::sharding::{ShardingManager, ShardingManagerTrait};

pub mod blockchain;
pub mod consensus;
pub mod currency;
pub mod governance;
pub mod identity;
pub mod network;
pub mod node;
pub mod sharding;
pub mod smart_contract;
pub mod vm;

pub use blockchain::{Block, Blockchain, Transaction};
pub use consensus::Consensus;
pub use currency::{CurrencySystem, CurrencyType, Wallet};
pub use governance::{DemocraticSystem, ProposalCategory, ProposalType};
pub use identity::{DecentralizedIdentity, DidManager};
pub use network::{Network, Node, Packet, PacketType};
pub use node::{ContentStore, ForwardingInformationBase, PendingInterestTable};
pub use smart_contract::{ExecutionEnvironment, SmartContract};
pub use vm::{CSCLCompiler, CoopVM, Opcode, Value};

pub struct IcnNode {
    pub content_store: Arc<Mutex<ContentStore>>,
    pub pit: Arc<Mutex<PendingInterestTable>>,
    pub fib: Arc<Mutex<ForwardingInformationBase>>,
    pub blockchain: Arc<Mutex<Blockchain>>,
    pub coop_vm: Arc<Mutex<CoopVM>>,
    pub sharding_manager: Arc<Mutex<dyn ShardingManagerTrait + Send>>,
}

impl IcnNode {
    pub fn new() -> Self {
        let consensus = Arc::new(Mutex::new(Consensus::new()));
        let sharding_manager = Arc::new(Mutex::new(ShardingManager::new(4, 10, Arc::clone(&consensus))));
        let blockchain = Blockchain::new(Arc::clone(&consensus), Arc::clone(&sharding_manager) as Arc<Mutex<dyn ShardingManagerTrait + Send>>);
        let coop_vm = CoopVM::new(Vec::new());

        IcnNode {
            content_store: Arc::new(Mutex::new(ContentStore::new())),
            pit: Arc::new(Mutex::new(PendingInterestTable::new())),
            fib: Arc::new(Mutex::new(ForwardingInformationBase::new())),
            blockchain: Arc::new(Mutex::new(blockchain)),
            coop_vm: Arc::new(Mutex::new(coop_vm)),
            sharding_manager,
        }
    }

    fn process_data(&self, packet: Packet) -> Result<(), Box<dyn Error>> {
        self.content_store.lock().unwrap().add(packet.name.clone(), packet.content.clone());
        if let Some(_interfaces) = self.pit.lock().unwrap().get_incoming_interfaces(&packet.name) {
            println!("Satisfying pending interests for data: {}", packet.name);
        }
        Ok(())
    }

    fn process_interest(&self, packet: Packet) -> Result<(), Box<dyn Error>> {
        if packet.packet_type != PacketType::Interest {
            return Err("Invalid packet type".into());
        }

        if self.fib.lock().unwrap().longest_prefix_match(&packet.name).is_some() {
            println!("Routing interest for: {}", packet.name);
            Ok(())
        } else {
            let mut pit = self.pit.lock().unwrap();
            pit.add_interest(packet.name.clone(), "interface");
            drop(pit);
            Err("No route found for interest packet".into())
        }
    }

    pub fn main() {
        let node = IcnNode::new();
        let interest_packet = Packet {
            packet_type: PacketType::Interest,
            name: "test_data".to_string(),
            content: vec![],
        };
        node.process_interest(interest_packet.clone()).unwrap();

        let data_packet = Packet {
            packet_type: PacketType::Data,
            name: "test_data".to_string(),
            content: vec![1, 2, 3],
        };
        node.process_data(data_packet).unwrap();
    }

    pub fn execute_smart_contract(&self, contract: String) -> Result<(), Box<dyn Error>> {
        let mut coop_vm = self.coop_vm.lock().unwrap();
        let opcodes = self.compile_contract(&contract)?;
        coop_vm.load_program(opcodes);
        coop_vm.run()?;
        Ok(())
    }

    fn compile_contract(&self, contract: &str) -> Result<Vec<Opcode>, Box<dyn Error>> {
        let mut compiler = CSCLCompiler::new(contract);
        compiler.compile()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_icn_node_creation() {
        let node = IcnNode::new();
        assert!(node.content_store.lock().unwrap().is_empty());
        assert!(node.pit.lock().unwrap().is_empty());
        assert!(node.fib.lock().unwrap().is_empty());
    }

    #[test]
    fn test_packet_processing() {
        let node = IcnNode::new();
        let interest_packet = Packet {
            packet_type: PacketType::Interest,
            name: "test_data".to_string(),
            content: vec![],
        };
        assert!(node.process_interest(interest_packet.clone()).is_ok());

        let data_packet = Packet {
            packet_type: PacketType::Data,
            name: "test_data".to_string(),
            content: vec![1, 2, 3],
        };
        assert!(node.process_data(data_packet).is_ok());

        let interest_packet = Packet {
            packet_type: PacketType::Interest,
            name: "test_data".to_string(),
            content: vec![],
        };
        assert!(node.process_interest(interest_packet).is_ok());
    }
}
