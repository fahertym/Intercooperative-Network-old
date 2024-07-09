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

// Re-export key structures and traits for easier use
pub use blockchain::{Block, Transaction, Blockchain};
pub use consensus::PoCConsensus;
pub use currency::{CurrencyType, CurrencySystem, Wallet};
pub use democracy::{DemocraticSystem, ProposalCategory, ProposalType};
pub use did::{DecentralizedIdentity, DidManager};
pub use network::{Node, Network};
pub use transaction_validator::TransactionValidator;

// ICN Node specific modules
pub use content_store::ContentStore;
pub use fib::ForwardingInformationBase;
pub use packet::{Packet, PacketType};
pub use pending_interest_table::PendingInterestTable;

pub struct IcnNode {
    #[allow(dead_code)]
    content_store: ContentStore,
    #[allow(dead_code)]
    pit: PendingInterestTable,
    #[allow(dead_code)]
    fib: ForwardingInformationBase,
}

impl IcnNode {
    pub fn new() -> Self {
        IcnNode {
            content_store: ContentStore::new(),
            pit: PendingInterestTable::new(),
            fib: ForwardingInformationBase::new(),
        }
    }

    // Add methods for ICN Node functionality here
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_icn_node_creation() {
        let _node = IcnNode::new();
        // Add assertions to check if the node is correctly initialized
    }
}
