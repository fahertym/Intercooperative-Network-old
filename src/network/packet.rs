#[derive(Debug, Clone, PartialEq)]
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
