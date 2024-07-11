// src/network/mod.rs

pub mod network;
pub mod packet;

pub use network::{Node, Network};
pub use packet::{Packet, PacketType};