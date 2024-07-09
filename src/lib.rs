mod content_store;
mod pending_interest_table;
mod fib;
mod packet;

use content_store::ContentStore;
use pending_interest_table::PendingInterestTable;
use fib::ForwardingInformationBase;
use packet::{Packet, PacketType};

use std::collections::HashMap;
use std::net::SocketAddr;

pub struct IcnNode {
    content_store: ContentStore,
    pit: PendingInterestTable,
    fib: ForwardingInformationBase,
    interfaces: HashMap<String, SocketAddr>,
}

impl IcnNode {
    pub fn new() -> Self {
        IcnNode {
            content_store: ContentStore::new(),
            pit: PendingInterestTable::new(),
            fib: ForwardingInformationBase::new(),
            interfaces: HashMap::new(),
        }
    }

    pub fn add_interface(&mut self, name: String, addr: SocketAddr) {
        self.interfaces.insert(name, addr);
    }

    pub fn process_packet(&mut self, packet: Packet, incoming_interface: &str) -> Vec<(SocketAddr, Packet)> {
        match packet.packet_type {
            PacketType::Interest => self.process_interest(packet, incoming_interface),
            PacketType::Data => self.process_data(packet, incoming_interface),
        }
    }

    fn process_interest(&mut self, packet: Packet, incoming_interface: &str) -> Vec<(SocketAddr, Packet)> {
        let name = packet.name.clone();
        let mut actions = Vec::new();

        // Check Content Store
        if let Some(content) = self.content_store.get(&name) {
            let data_packet = Packet {
                packet_type: PacketType::Data,
                name: name.clone(),
                content: content.clone(),
            };
            if let Some(&addr) = self.interfaces.get(incoming_interface) {
                actions.push((addr, data_packet));
            }
            return actions;
        }

        // Check PIT
        if self.pit.has_pending_interest(&name) {
            self.pit.add_incoming_interface(&name, incoming_interface);
            return actions;
        }

        // Forward based on FIB
        if let Some(fib_entry) = self.fib.longest_prefix_match(&name) {
            self.pit.add_interest(name, incoming_interface);
            for next_hop in &fib_entry.next_hops {
                actions.push((*next_hop, packet.clone()));
            }
        }

        actions
    }

    fn process_data(&mut self, packet: Packet, incoming_interface: &str) -> Vec<(SocketAddr, Packet)> {
        let name = packet.name.clone();
        let mut actions = Vec::new();

        // Add to Content Store
        self.content_store.add(name.clone(), packet.content.clone());

        // Check PIT and forward
        if let Some(interfaces) = self.pit.get_incoming_interfaces(&name) {
            for interface in interfaces {
                if interface != incoming_interface {
                    if let Some(&addr) = self.interfaces.get(interface) {
                        actions.push((addr, packet.clone()));
                    }
                }
            }
            self.pit.remove_interest(&name);
        }

        actions
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_icn_node() {
        let mut node = IcnNode::new();
        let addr1: SocketAddr = "127.0.0.1:8000".parse().unwrap();
        let addr2: SocketAddr = "127.0.0.1:8001".parse().unwrap();

        node.add_interface("eth0".to_string(), addr1);
        node.add_interface("eth1".to_string(), addr2);

        node.fib.add_entry("/test".to_string(), addr2);

        let interest_packet = Packet {
            packet_type: PacketType::Interest,
            name: "/test/data".to_string(),
            content: vec![],
        };

        let actions = node.process_packet(interest_packet.clone(), "eth0");
        assert_eq!(actions.len(), 1);
        assert_eq!(actions[0].0, addr2);

        let data_packet = Packet {
            packet_type: PacketType::Data,
            name: "/test/data".to_string(),
            content: vec![1, 2, 3, 4],
        };

        let actions = node.process_packet(data_packet.clone(), "eth1");
        assert_eq!(actions.len(), 1);
        assert_eq!(actions[0].0, addr1);
    }
}