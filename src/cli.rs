// File: src/cli.rs

use crate::blockchain::Blockchain;
use crate::smart_contract::parse_contract;
use std::io::{self, Write};

// Function to run the command-line interface
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

// Function to print the menu
fn print_menu() {
    println!("\n--- Smart Contract CLI ---");
    println!("1. Deploy a new smart contract");
    println!("2. Execute smart contracts");
    println!("3. View blockchain state");
    println!("4. Exit");
}

// Function to get user input
fn get_user_input(prompt: &str) -> String {
    print!("{}", prompt);
    io::stdout().flush().unwrap();
    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();
    input
}

// Function to deploy a smart contract
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
        Ok(contract) => {
            match blockchain.deploy_smart_contract(contract) {
                Ok(_) => println!("Smart contract deployed successfully!"),
                Err(e) => println!("Failed to deploy smart contract: {}", e),
            }
        }
        Err(e) => println!("Failed to parse smart contract: {}", e),
    }
}

// Function to execute smart contracts
fn execute_contracts(blockchain: &mut Blockchain) {
    match blockchain.execute_smart_contracts() {
        Ok(_) => println!("Smart contracts executed successfully!"),
        Err(e) => println!("Failed to execute smart contracts: {}", e),
    }
}

// Function to view the blockchain state
fn view_blockchain_state(blockchain: &Blockchain) {
    println!("Blockchain state:");
    println!("Number of blocks: {}", blockchain.chain.len());
    println!("Latest block smart contracts: {}", blockchain.chain.last().unwrap().smart_contracts.len());
    // Add more state information as needed
}
