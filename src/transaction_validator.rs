use crate::blockchain::{Transaction, Blockchain};
use ed25519_dalek::PublicKey;

pub struct TransactionValidator;

impl TransactionValidator {

impl TransactionValidator {
    pub fn validate_transaction(transaction: &Transaction, blockchain: &Blockchain, public_key: &PublicKey) -> bool {
        // Verify signature
        if !transaction.verify(public_key).unwrap_or(false) {
            return false;
        }

        // Check for double-spending
        if Self::is_double_spend(transaction, blockchain) {
            return false;
        }

        // Validate currency types and amounts
        if !Self::validate_currency_and_amount(transaction) {
            return false;
        }

        // Ensure sender has sufficient balance
        if !Self::check_sufficient_balance(transaction, blockchain) {
            return false;
        }

        true
    }

    fn is_double_spend(transaction: &Transaction, blockchain: &Blockchain) -> bool {
        for block in &blockchain.chain {
            for tx in &block.transactions {
                if tx.from == transaction.from && tx.timestamp == transaction.timestamp {
                    return true;
                }
            }
        }
        false
    }

    fn validate_currency_and_amount(transaction: &Transaction) -> bool {
        transaction.amount > 0.0
    }

    fn check_sufficient_balance(transaction: &Transaction, blockchain: &Blockchain) -> bool {
        let mut balance = 0.0;
        for block in &blockchain.chain {
            for tx in &block.transactions {
                if tx.from == transaction.from && tx.currency_type == transaction.currency_type {
                    balance -= tx.amount;
                }
                if tx.to == transaction.from && tx.currency_type == transaction.currency_type {
                    balance += tx.amount;
                }
            }
        }
        balance >= transaction.amount
    }
}