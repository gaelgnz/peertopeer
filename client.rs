use crate::packet::{self, PlayerPacket, send_packet};
use crate::player::Player;
use ::rand;
use macroquad::ui::root_ui;
use macroquad::{prelude::*, ui};
use std::thread;
use std::{
    net::{self, TcpStream, ToSocketAddrs},
    sync::{Arc, Mutex},
};

static PLAYER_WIDTH: f32 = 64.0;
static PLAYER_HEIGHT: f32 = 64.0;

#[macroquad::main("Host or Join")]
pub async fn main() {
    let mut stream = net::TcpStream::connect("127.0.0.1:8080").unwrap();

    let random_id = rand::random_range(0..323);
    println!("Generated: {}", random_id);
    let mut player = Player::new(random_id, 0.0, 0.0);

    let mut will_send: u8 = 5;
    let player_packets: Arc<Mutex<Vec<PlayerPacket>>> = Arc::new(Mutex::new(Vec::new()));

    loop {
        if will_send == 1 {
            println!("{:?}", player);
            println!("Compressing player data");
            let packet = PlayerPacket::from_player(&player);
            println!("Sending packet");
            send_packet(&mut stream, &packet).unwrap();
            println!("Packet sent");
            will_send = 5;
        } else {
            will_send -= 1;
        }

        clear_background(WHITE);

        player.vy += 0.5;

        player.y += player.vy;
        player.x += player.vx;

        if player.y > screen_height() - PLAYER_HEIGHT {
            player.y = screen_height() - PLAYER_HEIGHT;
            player.vy = 0.0;
        }

        if is_key_pressed(KeyCode::E) {
            if player.dir {
                player.vx += 40.0;
            } else {
                player.vx -= 40.0;
            }
        } else {
            if !player.sliding {
                // Apply air resistance to slow down horizontal movement
                player.vx *= 0.95; // Reduce velocity by 5% each frame

                // Stop completely if the velocity is very small
                if player.vx.abs() < 0.1 {
                    player.vx = 0.0;
                }
            }
        }
        if is_key_down(KeyCode::LeftShift) {
            player.sliding = true;
        } else {
            player.sliding = false;
        }
        if is_mouse_button_pressed(MouseButton::Middle) {
            player.vy += 20.0;
        }
        if is_key_pressed(KeyCode::Space) {
            player.vy = -10.0;
        }
        if is_key_down(KeyCode::S) {
            player.y += 5.0;
        }
        if is_key_down(KeyCode::A) {
            player.x -= 5.0;
            player.dir = false;
        }
        if is_key_down(KeyCode::D) {
            player.x += 5.0;
            player.dir = true;
        }

        match stream.try_clone() {
            Ok(mut clone) => {
                clone.set_nonblocking(true).unwrap_or_else(|e| {
                    println!("Failed to set non-blocking mode: {}", e);
                });

                match packet::receive_packet(&mut clone) {
                    Ok(packet) => {
                        println!("Received packet: {:?}", packet);
                        let mut packets = player_packets.lock().unwrap();

                        if let Some(existing) = packets.iter_mut().find(|p| p.id == packet.id) {
                            *existing = packet;
                        } else {
                            packets.push(packet);
                        }
                    }
                    Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {}
                    Err(e) => {
                        println!("Error receiving packet: {}", e);
                    }
                }

                // Reset to blocking mode
                clone.set_nonblocking(false).unwrap_or_else(|e| {
                    println!("Failed to reset blocking mode: {}", e);
                });
            }
            Err(e) => {
                println!("Failed to clone stream: {}", e);
            }
        }
        render_player(&player);
        render_players(player_packets.clone().lock().unwrap().clone());

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
        format!("Player {}", player.id).as_str(),
        player.x - 25.0,
        player.y - 5.0,
        20.0,
        BLACK,
    );
    draw_texture(
        &Texture2D::from_file_with_format(include_bytes!("../res/player.png"), None),
        player.x - 32.0,
        player.y,
        WHITE,
    );
}
