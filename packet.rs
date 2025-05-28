use crate::player::Player;
use std::io::Error;
use std::io::Read;
use std::io::Write;
use std::net::TcpStream;
#[derive(Debug, Clone)]
pub struct PlayerPacket {
    pub id: u32,
    pub x: f32,
    pub y: f32,
}

impl PlayerPacket {
    pub fn new(id: u32, x: f32, y: f32) -> Self {
        Self { id, x, y }
    }
    pub fn from_player(player: &Player) -> Self {
        Self {
            id: player.id,
            x: player.x,
            y: player.y,
        }
    }
    pub fn serialize(&self) -> Vec<u8> {
        let mut buffer = Vec::new();
        buffer.extend_from_slice(&self.id.to_be_bytes());
        buffer.extend_from_slice(&self.x.to_be_bytes());
        buffer.extend_from_slice(&self.y.to_be_bytes());
        buffer
    }
    pub fn deserialize(data: &[u8]) -> Self {
        let id = u32::from_be_bytes(data[0..4].try_into().unwrap());
        let x = f32::from_be_bytes(data[4..8].try_into().unwrap());
        let y = f32::from_be_bytes(data[8..12].try_into().unwrap());
        Self { id, x, y }
    }
}

pub fn send_packet(stream: &mut TcpStream, packet: &PlayerPacket) -> Result<(), Error> {
    let serialized_packet = packet.serialize();
    stream.write_all(&serialized_packet)?;
    Ok(())
}

pub fn receive_packet(stream: &mut TcpStream) -> Result<PlayerPacket, Error> {
    let mut buffer = [0u8; 12];
    stream.read_exact(&mut buffer)?;
    let packet = PlayerPacket::deserialize(&buffer);
    Ok(packet)
}
