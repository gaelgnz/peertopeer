use crate::map::{Map, TileKind};
use crate::packet::{self, PlayerPacket, send_packet};
use crate::player::Player;
use macroquad::prelude::*;
use std::io::{Read, Write};
use std::thread;
use std::time::Duration;
use std::{
    net::TcpStream,
    sync::{Arc, Mutex},
};

static PLAYER_WIDTH: f32 = 55.0;
static PLAYER_HEIGHT: f32 = 64.0;

#[macroquad::main("Momentum")]
pub async fn main() {
    println!("Connecting to server...");
    let mut stream = TcpStream::connect("127.0.0.1:8080").unwrap();

    let mut size_buf = [0u8; 4];
    stream.read_exact(&mut size_buf).unwrap();
    let map_size = u32::from_be_bytes(size_buf) as usize;
    println!("Map size: {}", map_size);
    stream.write_all(b"Ok, received map size").unwrap();
    thread::sleep(Duration::from_millis(100));
    let mut map_buf = vec![0u8; map_size];
    stream.read_exact(&mut map_buf).unwrap();

    let map: Map = bincode::decode_from_slice(&map_buf, bincode::config::standard())
        .unwrap()
        .0;
    println!("Map fetched: {:?}", map);

    let random_id = 32;
    let mut player = Player::new(random_id, 0.0, 0.0);

    let mut will_send: u8 = 5;
    let player_packets: Arc<Mutex<Vec<PlayerPacket>>> = Arc::new(Mutex::new(Vec::new()));

    loop {
        let camera = Camera2D {
            target: vec2(player.x, player.y),
            zoom: vec2(1.0 / screen_width() * 2.0, 1.0 / screen_height() * 2.0),
            ..Default::default()
        };
        set_camera(&camera);

        if will_send == 1 {
            let packet = PlayerPacket::from_player(&player);
            send_packet(&mut stream, &packet).unwrap();
            will_send = 5;
        } else {
            will_send -= 1;
        }

        clear_background(WHITE);

        // Input
        if is_key_down(KeyCode::A) {
            player.is_still = false;
            player.vx = -5.0;
            player.dir = false;
        } else if is_key_down(KeyCode::D) {
            player.is_still = false;
            player.vx = 5.0;
            player.dir = true;
        } else {
            if player.on_ground {
                player.is_still = true;
                player.vx *= 0.8;
                if player.vx.abs() < 0.6 {
                    player.vx = 0.0;
                }
            }
        }

        if is_key_pressed(KeyCode::Space) && player.on_ground {
            player.vy = -12.0;
            player.on_ground = false;
        }

        player.vy += 0.5; // gravity

        handle_collisions(&mut player, &map);

        // Receive packets
        match stream.try_clone() {
            Ok(mut clone) => {
                clone.set_nonblocking(true).ok();
                match packet::receive_packet(&mut clone) {
                    Ok(packet) => {
                        let mut packets = player_packets.lock().unwrap();
                        if let Some(existing) = packets.iter_mut().find(|p| p.id == packet.id) {
                            *existing = packet;
                        } else {
                            packets.push(packet);
                        }
                    }
                    Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {}
                    Err(e) => println!("Receive error: {}", e),
                }
                clone.set_nonblocking(false).ok();
            }
            Err(e) => println!("Stream clone error: {}", e),
        }

        render_player(&player);
        render_players(player_packets.lock().unwrap().clone());

        for (y, row) in map.tiles.iter().enumerate() {
            for (x, tile) in row.iter().enumerate() {
                match tile.kind {
                    TileKind::Grass => {
                        draw_rectangle(x as f32 * 32.0 - 32.0, y as f32 * 32.0, 32.0, 32.0, GREEN)
                    }
                    TileKind::Rock => {
                        draw_rectangle(x as f32 * 32.0 - 32.0, y as f32 * 32.0, 32.0, 32.0, BLACK)
                    }
                    TileKind::Empty => {}
                }
            }
        }

        next_frame().await;
    }
}

fn render_players(player_packets: Vec<PlayerPacket>) {
    for player_packet in player_packets {
        let player = Player::from_player_packet(&player_packet);
        render_player(&player);
    }
}

fn render_player(player: &Player) {
    draw_text(
        &format!("Player {}", player.id),
        player.x - 25.0,
        player.y - 5.0,
        20.0,
        BLACK,
    );

    let texture = Texture2D::from_file_with_format(include_bytes!("../res/player.png"), None);
    if !player.on_ground && player.vy > 0.0 {
        draw_texture_ex(
            &Texture2D::from_file_with_format(include_bytes!("../res/fall.png"), None),
            player.x - 32.0,
            player.y,
            WHITE,
            DrawTextureParams {
                flip_x: !player.dir,
                ..Default::default()
            },
        );
    } else if player.is_still {
        draw_texture_ex(
            &Texture2D::from_file_with_format(include_bytes!("../res/still.png"), None),
            player.x - 32.0,
            player.y,
            WHITE,
            DrawTextureParams {
                flip_x: !player.dir,
                ..Default::default()
            },
        );
    } else {
        draw_texture_ex(
            &texture,
            player.x - 32.0,
            player.y,
            WHITE,
            DrawTextureParams {
                flip_x: !player.dir,
                ..Default::default()
            },
        );
    }
}

fn handle_collisions(player: &mut Player, map: &Map) {
    const TILE_SIZE: f32 = 32.0;
    player.on_ground = false;

    // Horizontal movement
    player.x += player.vx;
    let (left, right) = (
        (player.x / TILE_SIZE).floor() as isize,
        ((player.x + PLAYER_WIDTH) / TILE_SIZE).ceil() as isize,
    );
    let (top, bottom) = (
        (player.y / TILE_SIZE).floor() as isize,
        ((player.y + PLAYER_HEIGHT) / TILE_SIZE).ceil() as isize,
    );

    for ty in top..bottom {
        for tx in left..right {
            if let Some(tile) = map.get_tile(tx as usize, ty as usize) {
                if matches!(tile.kind, TileKind::Rock) {
                    let tile_rect = Rect::new(
                        tx as f32 * TILE_SIZE,
                        ty as f32 * TILE_SIZE,
                        TILE_SIZE,
                        TILE_SIZE,
                    );
                    let player_rect = Rect::new(player.x, player.y, PLAYER_WIDTH, PLAYER_HEIGHT);

                    if player_rect.overlaps(&tile_rect) {
                        if player.vx > 0.0 {
                            player.x = tile_rect.x - PLAYER_WIDTH;
                        } else if player.vx < 0.0 {
                            player.x = tile_rect.x + TILE_SIZE;
                        }
                        player.vx = 0.0;
                    }
                }
            }
        }
    }

    // Vertical movement
    player.y += player.vy;
    let (left, right) = (
        (player.x / TILE_SIZE).floor() as isize,
        ((player.x + PLAYER_WIDTH) / TILE_SIZE).ceil() as isize,
    );
    let (top, bottom) = (
        (player.y / TILE_SIZE).floor() as isize,
        ((player.y + PLAYER_HEIGHT) / TILE_SIZE).ceil() as isize,
    );

    for ty in top..bottom {
        for tx in left..right {
            if let Some(tile) = map.get_tile(tx as usize, ty as usize) {
                if matches!(tile.kind, TileKind::Rock) {
                    let tile_rect = Rect::new(
                        tx as f32 * TILE_SIZE,
                        ty as f32 * TILE_SIZE,
                        TILE_SIZE,
                        TILE_SIZE,
                    );
                    let player_rect = Rect::new(player.x, player.y, PLAYER_WIDTH, PLAYER_HEIGHT);

                    if player_rect.overlaps(&tile_rect) {
                        if player.vy > 0.0 {
                            player.y = tile_rect.y - PLAYER_HEIGHT;
                            player.vy = 0.0;
                            player.on_ground = true;
                        } else if player.vy < 0.0 {
                            player.y = tile_rect.y + TILE_SIZE;
                            player.vy = 0.0;
                        }
                    }
                }
            }
        }
    }
}
