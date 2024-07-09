// src/did.rs

use chrono::{DateTime, Utc};
use ed25519_dalek::{Keypair, PublicKey, Signature, Signer, Verifier};
use rand::rngs::OsRng;
use serde::{Serialize, Deserialize};
use sha2::{Sha256, Digest};
use std::collections::HashMap;

/// Represents a Decentralized Identity (DiD)
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DecentralizedIdentity {
    pub id: String,
    pub public_key: PublicKey,
    pub created_at: DateTime<Utc>,
    pub reputation: f64,
    pub attributes: HashMap<String, String>,
}

impl DecentralizedIdentity {
    /// Creates a new Decentralized Identity
    pub fn new(attributes: HashMap<String, String>) -> (Self, Keypair) {
        let mut csprng = OsRng{};
        let keypair: Keypair = Keypair::generate(&mut csprng);
        let public_key = keypair.public;
        
        let id = format!("did:icn:{}", hex::encode(public_key.as_bytes()));
        
        (Self {
            id,
            public_key,
            created_at: Utc::now(),
            reputation: 1.0, // Initial reputation
            attributes,
        }, keypair)
    }

    /// Verifies a signature using the DiD's public key
    pub fn verify_signature(&self, message: &[u8], signature: &Signature) -> bool {
        self.public_key.verify(message, signature).is_ok()
    }
}

/// Manages Decentralized Identities
pub struct DidManager {
    identities: HashMap<String, DecentralizedIdentity>,
}

impl DidManager {
    /// Creates a new DidManager
    pub fn new() -> Self {
        Self {
            identities: HashMap::new(),
        }
    }

    /// Registers a new Decentralized Identity
    pub fn register_did(&mut self, did: DecentralizedIdentity) -> Result<(), String> {
        if self.identities.contains_key(&did.id) {
            return Err("DiD already exists".to_string());
        }
        self.identities.insert(did.id.clone(), did);
        Ok(())
    }

    /// Retrieves a Decentralized Identity by its ID
    pub fn get_did(&self, id: &str) -> Option<&DecentralizedIdentity> {
        self.identities.get(id)
    }

    /// Updates the reputation of a Decentralized Identity
    pub fn update_reputation(&mut self, id: &str, delta: f64) -> Result<(), String> {
        let did = self.identities.get_mut(id).ok_or("DiD not found")?;
        did.reputation += delta;
        did.reputation = did.reputation.max(0.0).min(100.0); // Clamp reputation between 0 and 100
        Ok(())
    }

    /// Verifies the identity of a DiD owner
    pub fn verify_identity(&self, id: &str, message: &[u8], signature: &Signature) -> bool {
        if let Some(did) = self.identities.get(id) {
            did.verify_signature(message, signature)
        } else {
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_did_creation_and_verification() {
        let mut attributes = HashMap::new();
        attributes.insert("name".to_string(), "Alice".to_string());
        let (did, keypair) = DecentralizedIdentity::new(attributes);

        assert!(did.id.starts_with("did:icn:"));
        assert_eq!(did.attributes.get("name"), Some(&"Alice".to_string()));

        let message = b"Hello, World!";
        let signature = keypair.sign(message);

        assert!(did.verify_signature(message, &signature));
    }

    #[test]
    fn test_did_manager() {
        let mut manager = DidManager::new();

        let mut attributes = HashMap::new();
        attributes.insert("name".to_string(), "Bob".to_string());
        let (did, _) = DecentralizedIdentity::new(attributes);

        assert!(manager.register_did(did.clone()).is_ok());
        assert!(manager.register_did(did.clone()).is_err());

        let retrieved_did = manager.get_did(&did.id).unwrap();
        assert_eq!(retrieved_did.attributes.get("name"), Some(&"Bob".to_string()));

        assert!(manager.update_reputation(&did.id, 5.0).is_ok());
        let updated_did = manager.get_did(&did.id).unwrap();
        assert_eq!(updated_did.reputation, 6.0);
    }
}