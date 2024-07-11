// ===============================================
// Command Line Interface (CLI) for ICN Node
// ===============================================
// This file defines the command line interface for interacting with the ICN Node.
// It provides options for deploying and executing smart contracts, and viewing blockchain state.
//
// Key concepts:
// - User Interaction: Allows users to interact with the ICN Node through a CLI.
// - Smart Contract Management: Provides functionalities to deploy and execute smart contracts.
// - Blockchain State Viewing: Allows users to view the current state of the blockchain.

use crate::blockchain::Blockchain;
use crate::smart_contract::parse_contract;
use std::io::{self, Write};

/// Runs the command-line interface for the ICN Node.
/// # Arguments
/// * `blockchain` - A mutable reference to the Blockchain instance.
pub fn run_cli(blockchain: &mut Blockchain) {
    loop {
        print_menu();
        let choice = get_user_input("Enter your choice: ");

        match choice.trim() {
            "1" => deploy_contract(blockchain),
            "2" => execute_contracts(blockchain),
            "3" => view_blockchain_state(blockchain),
            "4" => break,
            _ => println!("Invalid choice. Please try again."),
        }
    }
}

/// Prints the menu options for the CLI.
fn print_menu() {
    println!("\n--- Smart Contract CLI ---");
    println!("1. Deploy a new smart contract");
    println!("2. Execute smart contracts");
    println!("3. View blockchain state");
    println!("4. Exit");
}

/// Retrieves user input from the command line.
/// # Arguments
/// * `prompt` - A prompt message to display to the user.
/// # Returns
/// The user's input as a string.
fn get_user_input(prompt: &str) -> String {
    print!("{}", prompt);
    io::stdout().flush().unwrap();
    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();
    input
}

/// Deploys a smart contract to the blockchain.
/// # Arguments
/// * `blockchain` - A mutable reference to the Blockchain instance.
fn deploy_contract(blockchain: &mut Blockchain) {
    println!("Enter the smart contract details (type 'END' on a new line when finished):");
    let mut contract_input = String::new();
    loop {
        let line = get_user_input("");
        if line.trim() == "END" {
            break;
        }
        contract_input.push_str(&line);
    }

    match parse_contract(&contract_input) {
        Ok(mut contract) => {
            contract.activate(); // Activate the contract before deployment
            match blockchain.deploy_smart_contract(contract) {
                Ok(_) => println!("Smart contract deployed successfully!"),
                Err(e) => println!("Failed to deploy smart contract: {}", e),
            }
        }
        Err(e) => println!("Failed to parse smart contract: {}", e),
    }
}

/// Executes smart contracts on the blockchain.
/// # Arguments
/// * `blockchain` - A mutable reference to the Blockchain instance.
fn execute_contracts(blockchain: &mut Blockchain) {
    match blockchain.execute_smart_contracts() {
        Ok(_) => println!("Smart contracts executed successfully!"),
        Err(e) => println!("Failed to execute smart contracts: {}", e),
    }
}

/// Views the current state of the blockchain.
/// # Arguments
/// * `blockchain` - A reference to the Blockchain instance.
fn view_blockchain_state(blockchain: &Blockchain) {
    println!("Blockchain state:");
    println!("Number of blocks: {}", blockchain.chain.len());
    println!("Latest block smart contract results: {}", blockchain.chain.last().unwrap().smart_contract_results.len());
    // Add more state information as needed
}
