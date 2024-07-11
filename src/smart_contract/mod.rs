// src/smart_contract/mod.rs

mod smart_contract;

pub use smart_contract::{SmartContract, ExecutionEnvironment};
//use crate::vm::opcode::Opcode;
// Remove or comment out the following line:
// pub use crate::blockchain::TransactionValidator;