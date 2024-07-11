// ===============================================
// ICN Node Implementation
// ===============================================
// This file defines the ICN Node structure and its functionalities. It includes methods
// for processing packets, managing the forwarding information base (FIB), pending interest table (PIT),
// content store, and interacting with the blockchain and virtual machine (CoopVM).
//
// Key concepts:
// - Forwarding Information Base (FIB): A table that stores routing information for named data.
// - Pending Interest Table (PIT): A table that keeps track of interests that have been forwarded but not yet satisfied.
// - Content Store: A cache for storing data packets temporarily.
// - CoopVM: A virtual machine for executing compiled CSCL smart contract code.

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
pub use network::{Node, Network};
pub use node::{ContentStore, ForwardingInformationBase, Packet, PacketType, PendingInterestTable};
pub use smart_contract::TransactionValidator;
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
        let coop_vm = CoopVM::new();

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
        self.content_store.lock().unwrap().insert(packet.name.clone(), packet.content.clone());

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
        coop_vm.load_program(opcodes);
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
        Ok(compiler.compile())
    }
}
