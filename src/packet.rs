use crate::map::Map;
use crate::player::Player;
use bincode::{self, Decode, Encode};
use std::io::{Error, Read, Write};
use std::net::TcpStream;

#[derive(Clone, Debug)]
pub struct MapPacket {
    pub data: Map,
}

impl MapPacket {
    pub fn serialize(&self) -> Vec<u8> {
        bincode::encode_to_vec(&self.data, bincode::config::standard()).unwrap()
    }
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, Decode, Encode)]
pub struct PlayerPacket {
    pub id: u32,
    pub x: f32,
    pub y: f32,
    pub dir: bool,
}

impl PlayerPacket {
    pub fn new(id: u32, x: f32, y: f32, dir: bool) -> Self {
        Self { id, x, y, dir }
    }

    pub fn from_player(player: &Player) -> Self {
        Self {
            id: player.id,
            x: player.x,
            y: player.y,
            dir: player.dir,
        }
    }
}

/// Sends a PlayerPacket with a 4-byte length prefix, then the bincode-encoded data.
pub fn send_packet(stream: &mut TcpStream, packet: &PlayerPacket) -> Result<(), Error> {
    let encoded = bincode::encode_to_vec(packet, bincode::config::standard()).unwrap();
    let len_bytes = (encoded.len() as u32).to_be_bytes();

    stream.write_all(&len_bytes)?;
    stream.write_all(&encoded)?;
    Ok(())
}

/// Receives a PlayerPacket by first reading 4 bytes length prefix, then that many bytes of data.
pub fn receive_packet(stream: &mut TcpStream) -> Result<PlayerPacket, Error> {
    let mut size_buf = [0u8; 4];
    stream.read_exact(&mut size_buf)?;
    let size = u32::from_be_bytes(size_buf) as usize;

    let mut buf = vec![0u8; size];
    stream.read_exact(&mut buf)?;

    let (packet, _) = bincode::decode_from_slice(&buf, bincode::config::standard()).unwrap();
    Ok(packet)
}
