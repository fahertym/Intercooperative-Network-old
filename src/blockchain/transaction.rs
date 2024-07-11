// Filename: src/blockchain/transaction.rs

// ===============================================
// Transaction Implementation
// ===============================================
// This file contains the structure and functions for a single transaction in the blockchain.
// A transaction represents a record of value transfer or smart contract interaction.

// ===============================================
// Imports
// ===============================================

use serde::{Deserialize, Serialize};
use ed25519_dalek::{Keypair, PublicKey, Signature, Signer, Verifier};

use crate::smart_contract::SmartContract;
use crate::currency::CurrencyType;

// ===============================================
// Transaction Struct
// ===============================================
// Represents a single transaction in the blockchain

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transaction {
    pub from: String,                // Sender's address (public key or DID)
    pub to: String,                  // Recipient's address
    pub amount: f64,                 // Amount of currency being transferred
    pub currency_type: CurrencyType, // Type of currency (e.g., BasicNeeds, Education)
    pub gas_limit: u64,              // Maximum gas allowed for smart contract execution
    pub smart_contract: Option<SmartContract>, // Optional smart contract to execute
    pub signature: Option<String>,   // Digital signature to verify authenticity
}

impl Transaction {
    // Create a new Transaction
    pub fn new(from: String, to: String, amount: f64, currency_type: CurrencyType, gas_limit: u64) -> Self {
        Transaction {
            from,
            to,
            amount,
            currency_type,
            gas_limit,
            smart_contract: None,
            signature: None,
        }
    }

    // Attach a smart contract to the transaction
    pub fn with_smart_contract(mut self, smart_contract: SmartContract) -> Self {
        self.smart_contract = Some(smart_contract);
        self
    }

    // Sign the transaction with the given keypair
    pub fn sign(&mut self, keypair: &Keypair) -> Result<(), String> {
        let message = self.to_bytes();
        let signature = keypair.sign(&message);
        self.signature = Some(hex::encode(signature.to_bytes()));
        Ok(())
    }

    // Verify the transaction's signature
    pub fn verify(&self, public_key: &PublicKey) -> Result<bool, String> {
        let message = self.to_bytes();
        let signature_bytes = hex::decode(self.signature.as_ref().ok_or("No signature present")?).map_err(|e| e.to_string())?;
        let signature = Signature::from_bytes(&signature_bytes).map_err(|e| e.to_string())?;
        Ok(public_key.verify(&message, &signature).is_ok())
    }

    // Convert the transaction to bytes for signing/verification
    fn to_bytes(&self) -> Vec<u8> {
        // In a real implementation, this would properly serialize all fields
        format!("{}{}{}{:?}{}", self.from, self.to, self.amount, self.currency_type, self.gas_limit).into_bytes()
    }
}
