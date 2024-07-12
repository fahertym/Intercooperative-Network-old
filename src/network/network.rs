// File: src/network.rs

// ===============================================
// Network Node and Communication Infrastructure
// ===============================================
// This file contains the core structures and functions for managing network nodes
// and facilitating communication within our blockchain network. It includes
// definitions for different types of nodes, methods for adding and removing nodes,
// and functions for broadcasting information across the network.
//
// Key concepts:
// - Node Types: Different roles nodes can play in the network (e.g., personal devices, cooperative servers)
// - Network Topology: How nodes are connected and communicate with each other
// - Broadcasting: The process of sharing information (like new blocks) across the entire network

use std::collections::HashMap;
use serde::{Serialize, Deserialize};
use crate::blockchain::Block;

// ===============================================
// Node Type Enum
// ===============================================
// This enum represents the different types of nodes that can exist in our network.
// Each type of node may have different capabilities and responsibilities.

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum NodeType {
    PersonalDevice,    // Lightweight nodes typically run on personal computers or smartphones
    CooperativeServer, // More powerful nodes that might be run by cooperative organizations
    GovernmentServer,  // Nodes operated by governmental entities, potentially with special privileges
}

// ===============================================
// Node Struct
// ===============================================
// The Node struct represents a single node in our network. Each node has a unique
// identifier, a type, and a network address.

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Node {
    pub id: String,      // Unique identifier for the node
    pub node_type: NodeType, // The type of the node (personal, cooperative, government)
    pub address: String, // Network address of the node (e.g., IP address or URL)
}

impl Node {
    // Create a new node with the given parameters
    //
    // Parameters:
    // - id: A unique identifier for the node
    // - node_type: The type of the node (PersonalDevice, CooperativeServer, or GovernmentServer)
    // - address: The network address where the node can be reached
    //
    // Returns: A new Node instance
    pub fn new(id: &str, node_type: NodeType, address: &str) -> Self {
        Node {
            id: id.to_string(),
            node_type,
            address: address.to_string(),
        }
    }
}

// ===============================================
// Network Struct
// ===============================================
// The Network struct manages the collection of nodes in our blockchain network.
// It provides methods for adding and removing nodes, as well as broadcasting
// information to all nodes in the network.

pub struct Network {
    nodes: HashMap<String, Node>, // A map of node IDs to Node instances
}

impl Network {
    // Create a new, empty network
    //
    // Returns: A new Network instance with no nodes
    pub fn new() -> Self {
        Network {
            nodes: HashMap::new(),
        }
    }

    // Add a node to the network
    //
    // Parameters:
    // - node: The Node instance to add to the network
    //
    // This method will overwrite any existing node with the same ID
    pub fn add_node(&mut self, node: Node) {
        self.nodes.insert(node.id.clone(), node);
    }

    // Remove a node from the network
    //
    // Parameters:
    // - node_id: The ID of the node to remove
    //
    // If the node doesn't exist, this method does nothing
    pub fn remove_node(&mut self, node_id: &str) {
        self.nodes.remove(node_id);
    }

    // Get a reference to a node in the network
    //
    // Parameters:
    // - node_id: The ID of the node to retrieve
    //
    // Returns: An Option containing a reference to the Node if found, or None if not found
    pub fn get_node(&self, node_id: &str) -> Option<&Node> {
        self.nodes.get(node_id)
    }

    // Broadcast a block to all nodes in the network
    //
    // Parameters:
    // - block: The Block to broadcast to all nodes
    //
    // In a real implementation, this method would send the block to all nodes.
    // For now, it just prints a message indicating the broadcast.
    pub fn broadcast_block(&self, block: &Block) {
        println!("Broadcasting block {} to all nodes", block.index);
        // In a real implementation, we would iterate through all nodes and send the block:
        // for node in self.nodes.values() {
        //     send_block_to_node(node, block);
        // }
    }

    // Synchronize the blockchain across all nodes in the network
    //
    // Parameters:
    // - blockchain: The current state of the blockchain to synchronize
    //
    // In a real implementation, this method would ensure all nodes have the same blockchain state.
    // For now, it just prints a message indicating the synchronization.
    pub fn synchronize_blockchain(&self, _blockchain: &[Block]) {
        println!("Synchronizing blockchain across all nodes");
        // In a real implementation, we would:
        // 1. Determine the longest valid chain among all nodes
        // 2. Update any nodes with shorter chains to match the longest chain
        // 3. Resolve any conflicts or forks in the blockchain
    }
}

// ===============================================
// Helper Functions
// ===============================================

// In a real implementation, we would have helper functions for network operations, such as:
//
// fn send_block_to_node(node: &Node, block: &Block) {
//     // Implementation to send a block to a specific node
// }
//
// fn request_blockchain_from_node(node: &Node) -> Vec<Block> {
//     // Implementation to request the full blockchain from a specific node
// }

// ===============================================
// Tests
// ===============================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_network_operations() {
        let mut network = Network::new();

        // Test adding nodes
        let node1 = Node::new("node1", NodeType::PersonalDevice, "192.168.1.1");
        let node2 = Node::new("node2", NodeType::CooperativeServer, "192.168.1.2");
        network.add_node(node1.clone());
        network.add_node(node2.clone());

        assert_eq!(network.nodes.len(), 2);

        // Test getting a node
        let retrieved_node = network.get_node("node1").unwrap();
        assert_eq!(retrieved_node.id, "node1");
        assert_eq!(retrieved_node.address, "192.168.1.1");

        // Test removing a node
        network.remove_node("node1");
        assert_eq!(network.nodes.len(), 1);
        assert!(network.get_node("node1").is_none());

        // Test broadcasting (this just checks that it doesn't panic)
        let block = Block {
            index: 1,
            timestamp: 0,
            transactions: vec![],
            previous_hash: "previous_hash".to_string(),
            hash: "hash".to_string(),
            nonce: 0,
            gas_used: 0,
            smart_contract_results: HashMap::new(),
        };
        network.broadcast_block(&block);

        // Test synchronization (this just checks that it doesn't panic)
        network.synchronize_blockchain(&vec![block]);
    }
}

#[derive(Clone, Debug)]
pub enum PacketType {
    Interest,
    Data,
}

#[derive(Clone, Debug)]
pub struct Packet {
    pub packet_type: PacketType,
    pub name: String,
    pub content: Vec<u8>,
}
