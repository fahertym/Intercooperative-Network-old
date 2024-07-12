use std::sync::{Arc, Mutex};
use std::error::Error;

pub mod blockchain;
pub mod consensus;
pub mod currency;
pub mod governance;
pub mod identity;
pub mod network;
pub mod node;
pub mod smart_contract;
pub mod vm;

pub use blockchain::{Block, Transaction, Blockchain};
pub use consensus::PoCConsensus;
pub use currency::{CurrencyType, CurrencySystem, Wallet};
pub use governance::{DemocraticSystem, ProposalCategory, ProposalType};
pub use identity::{DecentralizedIdentity, DidManager};
pub use network::{Node, Network, Packet, PacketType};
pub use node::{ContentStore, ForwardingInformationBase, PendingInterestTable};
pub use smart_contract::{SmartContract, ExecutionEnvironment};
pub use vm::{CoopVM, Opcode, Value, CSCLCompiler};

/// The main struct representing an ICN Node.
/// It contains the content store, PIT, FIB, blockchain, and CoopVM.
pub struct IcnNode {
    pub content_store: Arc<Mutex<ContentStore>>,
    pub pit: Arc<Mutex<PendingInterestTable>>,
    pub fib: Arc<Mutex<ForwardingInformationBase>>,
    pub blockchain: Arc<Mutex<Blockchain>>,
    pub coop_vm: Arc<Mutex<CoopVM>>,
}

impl IcnNode {
    /// Creates a new instance of the ICN Node.
    pub fn new() -> Self {
        let blockchain = Blockchain::new();
        let coop_vm = CoopVM::new(Vec::new()); // Initialize with empty program

        IcnNode {
            content_store: Arc::new(Mutex::new(ContentStore::new())),
            pit: Arc::new(Mutex::new(PendingInterestTable::new())),
            fib: Arc::new(Mutex::new(ForwardingInformationBase::new())),
            blockchain: Arc::new(Mutex::new(blockchain)),
            coop_vm: Arc::new(Mutex::new(coop_vm)),
        }
    }

    /// Processes a packet, either an interest or a data packet.
    /// # Arguments
    /// * `packet` - The packet to be processed.
    /// # Returns
    /// Result indicating success or failure.
    pub fn process_packet(&self, packet: Packet) -> Result<(), Box<dyn Error>> {
        match packet.packet_type {
            PacketType::Interest => self.process_interest(packet),
            PacketType::Data => self.process_data(packet),
        }
    }

    /// Processes an interest packet by checking the content store and PIT.
    /// # Arguments
    /// * `packet` - The interest packet to be processed.
    /// # Returns
    /// Result indicating success or failure.
    fn process_interest(&self, packet: Packet) -> Result<(), Box<dyn Error>> {
        let content = self.content_store.lock().unwrap().get(&packet.name);

        if let Some(_data) = content {
            println!("Sending data for interest: {}", packet.name);
            Ok(())
        } else {
            self.pit.lock().unwrap().add_interest(packet.name.clone(), "default_interface");
            println!("Forwarding interest for: {}", packet.name);
            Err(format!("Content '{}' not found", packet.name).into())
        }
    }

    /// Processes a data packet by storing it in the content store and satisfying any pending interests.
    /// # Arguments
    /// * `packet` - The data packet to be processed.
    /// # Returns
    /// Result indicating success or failure.
    fn process_data(&self, packet: Packet) -> Result<(), Box<dyn Error>> {
        self.content_store.lock().unwrap().add(packet.name.clone(), packet.content.clone());

        if let Some(_interfaces) = self.pit.lock().unwrap().get_incoming_interfaces(&packet.name) {
            println!("Satisfying pending interests for data: {}", packet.name);
        }
        Ok(())
    }

    /// Executes a smart contract by compiling it and running it on the CoopVM.
    /// # Arguments
    /// * `contract` - The smart contract code as a string.
    /// # Returns
    /// Result indicating success or failure.
    pub fn execute_smart_contract(&self, contract: String) -> Result<(), Box<dyn Error>> {
        let mut coop_vm = self.coop_vm.lock().unwrap();
        let opcodes = self.compile_contract(&contract)?;
        coop_vm.load_program(opcodes); // Ensure this method is correctly called
        coop_vm.run()?;
        Ok(())
    }

    /// Compiles a smart contract from CSCL code to opcodes.
    /// # Arguments
    /// * `contract` - The smart contract code as a string.
    /// # Returns
    /// A vector of opcodes representing the compiled contract.
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

        assert!(node.process_packet(interest_packet.clone()).is_err());

        let data_packet = Packet {
            packet_type: PacketType::Data,
            name: "test_data".to_string(),
            content: vec![1, 2, 3],
        };

        assert!(node.process_packet(data_packet).is_ok());

        let interest_packet = Packet {
            packet_type: PacketType::Interest,
            name: "test_data".to_string(),
            content: vec![],
        };

        assert!(node.process_packet(interest_packet).is_ok());
    }
}
