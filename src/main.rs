mod blockchain;
mod smart_contract;
mod consensus;
mod currency;
mod democracy;
mod did;
mod network;
mod transaction_validator;
mod cli;

use crate::blockchain::Blockchain;
use crate::smart_contract::{SmartContract, ContractType, parse_contract};
use crate::consensus::PoCConsensus;
use crate::currency::CurrencySystem;
use crate::democracy::DemocraticSystem;
use crate::did::DidManager;
use crate::network::Network;
use crate::transaction_validator::TransactionValidator;
use crate::cli::run_cli;

use std::collections::HashMap;

fn main() {
    println!("Intercooperative Network (ICN) Project");

    // Initialize blockchain and related systems
    let consensus = PoCConsensus::new(0.5, 0.66);
    let currency_system = CurrencySystem::new();
    let democratic_system = DemocraticSystem::new();
    let did_manager = DidManager::new();
    let network = Network::new();
    let transaction_validator = TransactionValidator;

    let mut blockchain = Blockchain::new();

    // Add initial members
    blockchain.consensus.add_member("Alice".to_string());
    blockchain.consensus.add_member("Bob".to_string());
    blockchain.consensus.add_member("Charlie".to_string());
    blockchain.consensus.add_member("Dave".to_string());

    // Initialize member balances
    blockchain.execution_environment.balances.insert(
        "Alice".to_string(),
        [("ICN_TOKEN".to_string(), 1000.0)].iter().cloned().collect(),
    );
    blockchain.execution_environment.balances.insert(
        "Bob".to_string(),
        [("ICN_TOKEN".to_string(), 1000.0)].iter().cloned().collect(),
    );
    blockchain.execution_environment.balances.insert(
        "Charlie".to_string(),
        [("ICN_TOKEN".to_string(), 1000.0)].iter().cloned().collect(),
    );
    blockchain.execution_environment.balances.insert(
        "Dave".to_string(),
        [("ICN_TOKEN".to_string(), 1000.0)].iter().cloned().collect(),
    );

    // Example: Deploy an asset transfer smart contract
    let contract_input = "Asset Transfer
Creator: Alice
From: Alice
To: Bob
Asset: ICN_TOKEN
Amount: 100.0";

    match parse_contract(contract_input) {
        Ok(contract) => {
            if let Err(e) = blockchain.deploy_smart_contract(contract) {
                println!("Failed to deploy asset transfer contract: {}", e);
            } else {
                println!("Asset transfer contract deployed successfully!");
            }
        }
        Err(e) => println!("Failed to parse asset transfer contract: {}", e),
    }

    // Example: Deploy a proposal smart contract
    let proposal_input = "Proposal
Creator: Charlie
Title: New Community Project
Description: Implement a recycling program
Voting Period: 604800
Option 1: Approve
Option 2: Reject
Quorum: 0.5";

    match parse_contract(proposal_input) {
        Ok(contract) => {
            if let Err(e) = blockchain.deploy_smart_contract(contract) {
                println!("Failed to deploy proposal contract: {}", e);
            } else {
                println!("Proposal contract deployed successfully!");
            }
        }
        Err(e) => println!("Failed to parse proposal contract: {}", e),
    }

    // Execute smart contracts
    if let Err(e) = blockchain.execute_smart_contracts() {
        println!("Failed to execute smart contracts: {}", e);
    } else {
        println!("Smart contracts executed successfully!");
    }

    // Print initial blockchain state
    println!("\nInitial Blockchain State:");
    println!("Number of blocks: {}", blockchain.chain.len());
    println!("Latest block smart contracts: {}", blockchain.chain.last().unwrap().smart_contracts.len());
    println!("Member balances:");
    for member in ["Alice", "Bob", "Charlie", "Dave"].iter() {
        let balance = blockchain.execution_environment.balances.get(*member)
            .and_then(|b| b.get("ICN_TOKEN"))
            .unwrap_or(&0.0);
        println!("{}: {} ICN_TOKEN", member, balance);
    }

    // Run the CLI
    run_cli(&mut blockchain);

    println!("Exiting ICN Project. Goodbye!");
}
