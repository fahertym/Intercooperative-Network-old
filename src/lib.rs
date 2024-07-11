// src/lib.rs

pub mod blockchain;
pub mod consensus;
pub mod content_store;
pub mod currency;
pub mod democracy;
pub mod did;
pub mod fib;
pub mod network;
pub mod packet;
pub mod pending_interest_table;
pub mod transaction_validator;
pub mod smart_contract;
pub mod coop_vm;
pub mod cscl_compiler;

// Re-export key structures and traits for easier use
pub use blockchain::{Block, Transaction, Blockchain};
pub use consensus::PoCConsensus;
pub use currency::{CurrencyType, CurrencySystem, Wallet};
pub use democracy::{DemocraticSystem, ProposalCategory, ProposalType};
pub use did::{DecentralizedIdentity, DidManager};
pub use network::{Node, Network};
pub use transaction_validator::TransactionValidator;
pub use coop_vm::{CoopVM, Opcode, Value};
pub use cscl_compiler::CSCLCompiler;

// ICN Node specific modules
pub use content_store::ContentStore;
pub use fib::ForwardingInformationBase;
pub use packet::{Packet, PacketType};
pub use pending_interest_table::PendingInterestTable;

use std::sync::{Arc, Mutex};

pub struct IcnNode {
    content_store: Arc<Mutex<ContentStore>>,
    pit: Arc<Mutex<PendingInterestTable>>,
    fib: Arc<Mutex<ForwardingInformationBase>>,
    blockchain: Arc<Mutex<Blockchain>>,
    coop_vm: Arc<Mutex<CoopVM>>,
}

impl IcnNode {
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

    pub fn process_packet(&self, packet: Packet) -> Result<(), String> {
        match packet.packet_type {
            PacketType::Interest => self.process_interest(packet),
            PacketType::Data => self.process_data(packet),
        }
    }

    fn process_interest(&self, _packet: Packet) -> Result<(), String> {
        // Implementation for processing interest packets
        // This would involve checking the ContentStore, updating the PIT, and using the FIB
        Ok(())
    }

    fn process_data(&self, _packet: Packet) -> Result<(), String> {
        // Implementation for processing data packets
        // This would involve updating the ContentStore and satisfying pending interests in the PIT
        Ok(())
    }

    pub fn execute_smart_contract(&self, contract: String) -> Result<(), String> {
        let mut coop_vm = self.coop_vm.lock().unwrap();
        let opcodes = self.compile_contract(contract)?;
        coop_vm.run() // Changed from coop_vm.run(opcodes)
    }

    fn compile_contract(&self, contract: String) -> Result<Vec<Opcode>, String> {
        let mut compiler = CSCLCompiler::new(&contract);
        Ok(compiler.compile())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_icn_node_creation() {
        let node = IcnNode::new();
        // Add assertions to check if the node is correctly initialized
        assert!(node.content_store.lock().unwrap().get("test").is_none());
        assert!(node.pit.lock().unwrap().get_incoming_interfaces("test").is_none());
        assert!(node.fib.lock().unwrap().get_next_hops("test").is_none());
    }

    #[test]
    fn test_packet_processing() {
        let node = IcnNode::new();
        let interest_packet = Packet {
            packet_type: PacketType::Interest,
            name: "test".to_string(),
            content: vec![],
        };
        assert!(node.process_packet(interest_packet).is_ok());

        let data_packet = Packet {
            packet_type: PacketType::Data,
            name: "test".to_string(),
            content: vec![1, 2, 3],
        };
        assert!(node.process_packet(data_packet).is_ok());
    }
}