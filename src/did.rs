// File: src/did.rs

// ==================================================
// Imports
// ==================================================
use chrono::{DateTime, Utc};
use ed25519_dalek::{Keypair, PublicKey, Signature, Verifier};
use rand::rngs::OsRng;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ==================================================
// Enums and Structs for Decentralized Identity
// ==================================================

// Struct to represent a decentralized identity
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DecentralizedIdentity {
    pub id: String,                      // Unique identifier for the DID
    #[serde(with = "public_key_serde")]
    pub public_key: PublicKey,           // Public key for the DID
    pub created_at: DateTime<Utc>,       // Timestamp of creation
    pub reputation: f64,                 // Reputation score (0 to 100)
    pub attributes: HashMap<String, String>, // Additional attributes
}

// Module for (de)serialization of the public key
mod public_key_serde {
    use ed25519_dalek::PublicKey;
    use serde::{Deserialize, Deserializer, Serialize, Serializer};

    pub fn serialize<S>(public_key: &PublicKey, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let bytes = public_key.to_bytes();
        bytes.serialize(serializer)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<PublicKey, D::Error>
    where
        D: Deserializer<'de>,
    {
        let bytes = Vec::<u8>::deserialize(deserializer)?;
        PublicKey::from_bytes(&bytes).map_err(serde::de::Error::custom)
    }
}

// Implementation of DecentralizedIdentity
impl DecentralizedIdentity {
    // Function to create a new decentralized identity
    pub fn new(attributes: HashMap<String, String>) -> (Self, Keypair) {
        let mut csprng = OsRng {};
        let keypair: Keypair = Keypair::generate(&mut csprng); // Generate a key pair
        let public_key = keypair.public;

        // Generate a unique ID based on the public key
        let id = format!("did:icn:{}", hex::encode(public_key.to_bytes()));

        // Return the new identity and the key pair
        (
            Self {
                id,
                public_key,
                created_at: Utc::now(),
                reputation: 1.0, // Initial reputation
                attributes,
            },
            keypair,
        )
    }

    // Function to verify a signature
    pub fn verify_signature(&self, message: &[u8], signature: &Signature) -> bool {
        self.public_key.verify(message, signature).is_ok()
    }
}

// ==================================================
// Struct to Manage Decentralized Identities
// ==================================================

// Struct to manage decentralized identities
pub struct DidManager {
    identities: HashMap<String, DecentralizedIdentity>, // Map of DIDs to their identities
}

// Implementation of DidManager
impl DidManager {
    // Constructor to create a new DidManager
    pub fn new() -> Self {
        Self {
            identities: HashMap::new(),
        }
    }

    // Function to register a decentralized identity
    pub fn register_did(&mut self, did: DecentralizedIdentity) -> Result<(), String> {
        if self.identities.contains_key(&did.id) {
            return Err("DiD already exists".to_string());
        }
        self.identities.insert(did.id.clone(), did);
        Ok(())
    }

    // Function to get a decentralized identity by ID
    pub fn get_did(&self, id: &str) -> Option<&DecentralizedIdentity> {
        self.identities.get(id)
    }

    // Function to update the reputation of a decentralized identity
    pub fn update_reputation(&mut self, id: &str, delta: f64) -> Result<(), String> {
        let did = self.identities.get_mut(id).ok_or("DiD not found")?;
        did.reputation += delta;
        did.reputation = did.reputation.max(0.0).min(100.0); // Clamp reputation between 0 and 100
        Ok(())
    }

    // Function to verify the identity of a decentralized identity
    pub fn verify_identity(&self, id: &str, message: &[u8], signature: &Signature) -> bool {
        if let Some(did) = self.identities.get(id) {
            did.verify_signature(message, signature)
        } else {
            false
        }
    }

    // Function to update the attributes of a decentralized identity
    pub fn update_attributes(&mut self, id: &str, attributes: HashMap<String, String>) -> Result<(), String> {
        let did = self.identities.get_mut(id).ok_or("DiD not found")?;
        did.attributes.extend(attributes);
        Ok(())
    }

    // Function to revoke a decentralized identity
    pub fn revoke_did(&mut self, id: &str) -> Result<(), String> {
        if self.identities.remove(id).is_some() {
            Ok(())
        } else {
            Err("DiD not found".to_string())
        }
    }

    // Function to list all decentralized identities
    pub fn list_dids(&self) -> Vec<String> {
        self.identities.keys().cloned().collect()
    }
}

// ==================================================
// Unit Tests for Decentralized Identity
// ==================================================
#[cfg(test)]
mod tests {
    use super::*;
    use ed25519_dalek::Signer;

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

        let mut new_attributes = HashMap::new();
        new_attributes.insert("age".to_string(), "30".to_string());
        assert!(manager.update_attributes(&did.id, new_attributes).is_ok());

        let final_did = manager.get_did(&did.id).unwrap();
        assert_eq!(final_did.attributes.get("age"), Some(&"30".to_string()));

        assert!(manager.revoke_did(&did.id).is_ok());
        assert!(manager.get_did(&did.id).is_none());
    }
}
