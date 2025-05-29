use bincode::{Decode, Encode};
use serde::{Deserialize, Serialize};

use crate::packet::MapPacket;
#[derive(Serialize, Deserialize, Clone, Encode, Decode, Debug, Copy)]
pub enum TileKind {
    Grass,
    Rock,
    Empty,
}
#[derive(Serialize, Deserialize, Clone, Encode, Decode, Debug, Copy)]
pub struct Tile {
    pub collision: bool,
    pub kind: TileKind,
}
#[derive(Serialize, Deserialize, Clone, Encode, Decode, Debug)]
pub struct Map {
    pub height: u32,
    pub width: u32,
    pub tiles: Vec<Vec<Tile>>,
}

impl Map {
    pub fn from_map_packet(map_packet: MapPacket) -> Self {
        map_packet.data
    }
    pub fn get_tile(&self, x: usize, y: usize) -> Option<&Tile> {
        self.tiles.get(y).and_then(|row| row.get(x))
    }
    pub fn new(height: u32, width: u32) -> Self {
        Map {
            height,
            width,
            tiles: vec![
                vec![
                    Tile {
                        collision: false,
                        kind: TileKind::Empty
                    };
                    width as usize
                ];
                height as usize
            ],
        }
    }
}
