use serde::{Deserialize, Serialize};
use ed25519_dalek::{Keypair, PublicKey, Signature, Signer, Verifier};
use crate::smart_contract::SmartContract;
use crate::currency::CurrencyType;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Transaction {
    pub from: String,
    pub to: String,
    pub amount: f64,
    pub currency_type: CurrencyType,
    pub gas_limit: u64,
    pub smart_contract: Option<SmartContract>,
    pub signature: Option<Vec<u8>>,
    pub public_key: Option<Vec<u8>>,
}

impl Transaction {
    pub fn new(from: String, to: String, amount: f64, currency_type: CurrencyType, gas_limit: u64) -> Self {
        Transaction {
            from,
            to,
            amount,
            currency_type,
            gas_limit,
            smart_contract: None,
            signature: None,
            public_key: None,
        }
    }

    pub fn sign(&mut self, keypair: &Keypair) -> Result<(), String> {
        let message = self.to_bytes();
        let signature = keypair.sign(&message);
        self.signature = Some(signature.to_bytes().to_vec());
        self.public_key = Some(keypair.public.to_bytes().to_vec());
        Ok(())
    }

    pub fn verify(&self) -> Result<bool, String> {
        let public_key_bytes = self.public_key.as_ref().ok_or("No public key present")?;
        let signature_bytes = self.signature.as_ref().ok_or("No signature present")?;
        
        let public_key = PublicKey::from_bytes(public_key_bytes).map_err(|e| e.to_string())?;
        let signature = Signature::from_bytes(signature_bytes).map_err(|e| e.to_string())?;
        
        let message = self.to_bytes();
        Ok(public_key.verify(&message, &signature).is_ok())
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        format!(
            "{}{}{}:{:?}:{}",
            self.from,
            self.to,
            self.amount,
            self.currency_type,
            self.gas_limit
        ).into_bytes()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::rngs::OsRng;

    #[test]
    fn test_transaction_sign_and_verify() {
        let mut csprng = OsRng;
        let keypair: Keypair = Keypair::generate(&mut csprng);

        let mut transaction = Transaction::new(
            "Alice".to_string(),
            "Bob".to_string(),
            100.0,
            CurrencyType::BasicNeeds,
            1000,
        );

        // Sign the transaction
        transaction.sign(&keypair).unwrap();

        // Verify the transaction
        assert!(transaction.verify().unwrap());

        // Tamper with the transaction
        transaction.amount = 200.0;

        // Verification should fail
        assert!(!transaction.verify().unwrap());
    }
}