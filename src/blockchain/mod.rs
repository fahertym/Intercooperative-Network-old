// Filename: src/blockchain/mod.rs

// =================================================
// Overview
// =================================================
// This module groups and re-exports the components of the blockchain module.
// It includes the block, transaction, and blockchain functionality.

// =================================================
// Imports and Module Declarations
// =================================================

mod block; // Import the block module
mod transaction; // Import the transaction module
mod blockchain; // Import the blockchain module

// Re-export the block, transaction, and blockchain structs for external use
pub use block::Block;
pub use transaction::Transaction;
pub use blockchain::Blockchain;
