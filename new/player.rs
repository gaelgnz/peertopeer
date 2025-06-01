#[derive(Debug)]
pub struct Player {
    pub id: u32,
    pub x: f32,
    pub y: f32,
    pub vx: f32,
    pub vy: f32,
    pub dir: bool,
    pub on_ground: bool,
    pub is_still: bool,
    pub message: String,
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
            on_ground: false,
            is_still: false,
            message: String::new(),
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
            dir: packet.dir,
            on_ground: false,
            is_still: false,
            message: packet.message.clone(),
        }
    }
}
