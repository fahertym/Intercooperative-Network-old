// src/network.rs

use std::collections::HashMap;
use serde::{Serialize, Deserialize};

use crate::blockchain::Block;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum NodeType {
    PersonalDevice,
    CooperativeServer,
    GovernmentServer,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Node {
    pub id: String,
    pub node_type: NodeType,
    pub address: String,
}

impl Node {
    pub fn new(id: &str, node_type: NodeType, address: &str) -> Self {
        Node {
            id: id.to_string(),
            node_type,
            address: address.to_string(),
        }
    }
}

pub struct Network {
    nodes: HashMap<String, Node>,
}

impl Network {
    pub fn new() -> Self {
        Network {
            nodes: HashMap::new(),
        }
    }

    pub fn add_node(&mut self, node: Node) {
        self.nodes.insert(node.id.clone(), node);
    }

    pub fn remove_node(&mut self, node_id: &str) {
        self.nodes.remove(node_id);
    }

    pub fn get_node(&self, node_id: &str) -> Option<&Node> {
        self.nodes.get(node_id)
    }

    pub fn broadcast_block(&self, block: &Block) {
        println!("Broadcasting block {} to all nodes", block.index);
        // In a real implementation, this would send the block to all nodes in the network
    }

    pub fn synchronize_blockchain(&self, _blockchain: &[Block]) {
        println!("Synchronizing blockchain across all nodes");
        // In a real implementation, this would ensure all nodes have the same blockchain state
    }
}