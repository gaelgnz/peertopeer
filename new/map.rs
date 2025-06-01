use bincode::{Decode, Encode};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Encode, Decode, Debug, Copy, PartialEq)]
pub enum TileKind {
    Grass,
    Rock,
    Empty,
}
#[derive(Serialize, Deserialize, Clone, Encode, Decode, Debug, Copy, PartialEq)]
pub struct Tile {
    pub collision: bool,
    pub kind: TileKind,
}
#[derive(Serialize, Deserialize, Clone, Encode, Decode, Debug, PartialEq)]
pub enum ObjectKind {
    StartLine,
    FinishLine,
}

#[derive(Serialize, Deserialize, Clone, Encode, Decode, Debug, PartialEq)]
pub struct Object {
    pub x: f32,
    pub y: f32,
    pub kind: ObjectKind,
}

#[derive(Serialize, Deserialize, Clone, Encode, Decode, Debug, PartialEq)]
pub struct Map {
    pub height: u32,
    pub width: u32,
    pub tiles: Vec<Vec<Tile>>,
    pub objects: Vec<Object>,
}

impl Map {
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
            objects: Vec::new(),
        }
    }
}
