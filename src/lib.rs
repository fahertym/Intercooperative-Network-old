use std::sync::{Arc, Mutex};
use std::error::Error;

// Import other necessary modules
// Each module encapsulates a specific aspect of the ICN node functionality
pub mod blockchain;   // Manages the blockchain, blocks, and transactions
pub mod consensus;     // Handles consensus mechanisms (e.g., Proof of Contribution)
pub mod content_store; // Stores content data received through the network
pub mod currency;     // Manages currency types and transactions
pub mod democracy;    // Implements the democratic decision-making process
pub mod did;          // Handles decentralized identities (DIDs)
pub mod fib;          // Maintains the Forwarding Information Base for routing
pub mod network;      // Handles network communication and message passing
pub mod packet;       // Defines the structure of packets (Interest, Data)
pub mod pending_interest_table; // Keeps track of pending interests for data
pub mod transaction_validator; // Validates transactions before adding them to the blockchain
pub mod smart_contract; // Handles smart contract execution
pub mod coop_vm;      // The Cooperative Virtual Machine for running smart contracts
pub mod cscl_compiler; // The CSCL (Cooperative Smart Contract Language) compiler

// Re-export key structures and traits from modules for easier use
pub use blockchain::{Block, Transaction, Blockchain};
pub use consensus::PoCConsensus;
pub use currency::{CurrencyType, CurrencySystem, Wallet};
pub use democracy::{DemocraticSystem, ProposalCategory, ProposalType};
pub use did::{DecentralizedIdentity, DidManager};
pub use network::{Node, Network};
pub use transaction_validator::TransactionValidator;
pub use coop_vm::{CoopVM, Opcode, Value};
pub use cscl_compiler::CSCLCompiler;

// Re-export structures specific to ICN node operations
pub use content_store::ContentStore;
pub use fib::ForwardingInformationBase;
pub use packet::{Packet, PacketType};
pub use pending_interest_table::PendingInterestTable;

// ==================================================
// IcnNode: The Core Structure of the ICN (InterCooperative Network) Node
// ==================================================

pub struct IcnNode {
    // Thread-safe shared access to the content store for storing and retrieving data
    content_store: Arc<Mutex<ContentStore>>, 

    // Thread-safe shared access to the pending interest table (PIT) for tracking interest packets
    pit: Arc<Mutex<PendingInterestTable>>, 

    // Thread-safe shared access to the forwarding information base (FIB) for routing decisions
    fib: Arc<Mutex<ForwardingInformationBase>>,  

    // Thread-safe shared access to the blockchain for storing transactions and state
    blockchain: Arc<Mutex<Blockchain>>, 

    // Thread-safe shared access to the CoopVM for executing smart contracts
    coop_vm: Arc<Mutex<CoopVM>>,  
}

impl IcnNode {
    // Constructor to create a new IcnNode
    pub fn new() -> Self {
        // Initialize blockchain and CoopVM with empty state
        let blockchain = Blockchain::new();
        let coop_vm = CoopVM::new(); 

        // Create the IcnNode with thread-safe shared access to its components
        IcnNode {
            content_store: Arc::new(Mutex::new(ContentStore::new())),
            pit: Arc::new(Mutex::new(PendingInterestTable::new())),
            fib: Arc::new(Mutex::new(ForwardingInformationBase::new())),
            blockchain: Arc::new(Mutex::new(blockchain)),
            coop_vm: Arc::new(Mutex::new(coop_vm)),
        }
    }

    // ==================================================
    // Core Packet Processing Logic
    // ==================================================

    // Process an incoming packet (Interest or Data)
    pub fn process_packet(&self, packet: Packet) -> Result<(), Box<dyn Error>> {
        match packet.packet_type {
            PacketType::Interest => self.process_interest(packet), // Process an Interest packet
            PacketType::Data => self.process_data(packet),         // Process a Data packet
        }
    }

    // Process an Interest packet
    fn process_interest(&self, packet: Packet) -> Result<(), Box<dyn Error>> {
        // Check if the requested content exists in the content store
        let content = self.content_store.lock().unwrap().get(&packet.name);

        if let Some(_data) = content {
            // If content exists, prepare and send a Data packet back to the requester
            // (implementation of sending data packet is needed here)
            println!("Sending data for interest: {}", packet.name);
            Ok(())
        } else {
            // If content doesn't exist, add the interest to the Pending Interest Table (PIT)
            self.pit.lock().unwrap().add_interest(packet.name.clone(), "default_interface");

            // Use the Forwarding Information Base (FIB) to determine the next hop(s) for the Interest
            // and forward the Interest packet accordingly (implementation needed)
            println!("Forwarding interest for: {}", packet.name);

            // Return an error indicating the content was not found
            Err(format!("Content '{}' not found", packet.name).into()) 
        }
    }

    // Process a Data packet
    fn process_data(&self, packet: Packet) -> Result<(), Box<dyn Error>> {
        // Store the received data in the content store
        self.content_store.lock().unwrap().insert(packet.name.clone(), packet.content.clone());

        // Check if there are any pending interests for this data in the PIT
        if let Some(_interfaces) = self.pit.lock().unwrap().get_incoming_interfaces(&packet.name) {
            // If pending interests exist, send the Data packet to the corresponding interfaces
            // (implementation of sending data to interfaces is needed here)
            println!("Satisfying pending interests for data: {}", packet.name);
        } 
        Ok(())
    }

    // ==================================================
    // Smart Contract Execution
    // ==================================================

    pub fn execute_smart_contract(&self, contract: String) -> Result<(), Box<dyn Error>> {
        let mut coop_vm = self.coop_vm.lock().unwrap();
        let opcodes = self.compile_contract(&contract)?;
        coop_vm.load_program(opcodes); // Assuming `load_program` sets the opcodes
        coop_vm.run()?;  // Propagate errors from CoopVM
        Ok(())
    }

    // Compile a CSCL (Cooperative Smart Contract Language) contract
    fn compile_contract(&self, contract: &str) -> Result<Vec<Opcode>, Box<dyn Error>> {
        let mut compiler = CSCLCompiler::new(contract);
        Ok(compiler.compile()) // Propagate errors from the compiler
    }
}

// ==================================================
// Unit Tests for IcnNode
// ==================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_icn_node_creation() {
        let node = IcnNode::new();

        // Check if internal data structures are initially empty
        assert!(node.content_store.lock().unwrap().is_empty()); 
        assert!(node.pit.lock().unwrap().is_empty());          
        assert!(node.fib.lock().unwrap().is_empty());           
    }

    #[test]
    fn test_packet_processing() {
        let node = IcnNode::new();

        // Create a test Interest packet
        let interest_packet = Packet {
            packet_type: PacketType::Interest,
            name: "test_data".to_string(),
            content: vec![],
        };

        // Initially, the content should not be found
        assert!(node.process_packet(interest_packet.clone()).is_err());

        // Create a test Data packet for the same content name
        let data_packet = Packet {
            packet_type: PacketType::Data,
            name: "test_data".to_string(),
            content: vec![1, 2, 3],
        };

        // Processing the data packet should store the content
        assert!(node.process_packet(data_packet).is_ok());

        // Now, create another Interest packet for the same content name
        let interest_packet = Packet {
            packet_type: PacketType::Interest,
            name: "test_data".to_string(),
            content: vec![],
        };

        // Processing this Interest packet should now succeed as the content is available
        assert!(node.process_packet(interest_packet).is_ok());
    }
}
