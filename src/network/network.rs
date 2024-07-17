use std::collections::HashMap;
use serde::{Serialize, Deserialize};
use crate::blockchain::Block;
use crate::network::node::NodeType; // Ensure correct import

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
    }

    pub fn synchronize_blockchain(&self, _blockchain: &[Block]) {
        println!("Synchronizing blockchain across all nodes");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_network_operations() {
        let mut network = Network::new();

        let node1 = Node::new("node1", NodeType::PersonalDevice, "192.168.1.1");
        let node2 = Node::new("node2", NodeType::CooperativeServer, "192.168.1.2");
        network.add_node(node1.clone());
        network.add_node(node2.clone());

        assert_eq!(network.nodes.len(), 2);

        let retrieved_node = network.get_node("node1").unwrap();
        assert_eq!(retrieved_node.id, "node1");
        assert_eq!(retrieved_node.address, "192.168.1.1");

        network.remove_node("node1");
        assert_eq!(network.nodes.len(), 1);
        assert!(network.get_node("node1").is_none());

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
