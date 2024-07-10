// File: src/network.rs

use std::collections::HashMap;
use serde::{Serialize, Deserialize};

use crate::blockchain::Block;

// Enum to represent the type of a node in the network
#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum NodeType {
    PersonalDevice,
    CooperativeServer,
    GovernmentServer,
}

// Struct to represent a node in the network
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Node {
    pub id: String,
    pub node_type: NodeType,
    pub address: String,
}

impl Node {
    // Function to create a new node
    pub fn new(id: &str, node_type: NodeType, address: &str) -> Self {
        Node {
            id: id.to_string(),
            node_type,
            address: address.to_string(),
        }
    }
}

// Struct to represent the network
pub struct Network {
    nodes: HashMap<String, Node>,
}

impl Network {
    pub fn new() -> Self {
        Network {
            nodes: HashMap::new(),
        }
    }

    // Function to add a node to the network
    pub fn add_node(&mut self, node: Node) {
        self.nodes.insert(node.id.clone(), node);
    }

    // Function to remove a node from the network
    pub fn remove_node(&mut self, node_id: &str) {
        self.nodes.remove(node_id);
    }

    // Function to get a node from the network
    pub fn get_node(&self, node_id: &str) -> Option<&Node> {
        self.nodes.get(node_id)
    }

    // Function to broadcast a block to all nodes in the network
    pub fn broadcast_block(&self, block: &Block) {
        println!("Broadcasting block {} to all nodes", block.index);
        // In a real implementation, this would send the block to all nodes in the network
    }

    // Function to synchronize the blockchain across all nodes in the network
    pub fn synchronize_blockchain(&self, _blockchain: &[Block]) {
        println!("Synchronizing blockchain across all nodes");
        // In a real implementation, this would ensure all nodes have the same blockchain state
    }
}
