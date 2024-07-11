// ===============================================
// Tests Module
// ===============================================
// This module re-exports the contents of the tests submodules.
// The tests submodules contain various test cases to ensure
// the correctness and reliability of the blockchain implementation.

pub mod blockchain_and_consensus_tests;
pub mod blockchain_tests;
pub mod icn_node_tests;
pub mod integration_tests;
pub mod smart_contract_tests;

pub use blockchain_and_consensus_tests::*;
pub use blockchain_tests::*;
pub use icn_node_tests::*;
pub use integration_tests::*;
pub use smart_contract_tests::*;
