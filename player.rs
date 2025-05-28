#[derive(Debug)]
pub struct Player {
    pub id: u32,
    pub x: f32,
    pub y: f32,
    pub vx: f32,
    pub vy: f32,
    pub dir: bool,
    pub sliding: bool,
}
impl Player {
    pub fn new(id: u32, x: f32, y: f32) -> Self {
        Player {
            id,
            x,
            y,
            vx: 0.0,
            vy: 0.0,
            dir: false,
            sliding: false,
        }
    }
}
impl Player {
    pub fn from_player_packet(packet: &crate::packet::PlayerPacket) -> Self {
        Self {
            id: packet.id,
            x: packet.x,
            y: packet.y,
            vx: 0.0,
            vy: 0.0,
            dir: false,
            sliding: false,
        }
    }
}
