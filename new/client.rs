use crate::map::{Map, ObjectKind, TileKind};
use crate::packet::{self, PlayerPacket, send_packet};
use crate::player::Player;
use ::rand;
use macroquad::audio::{load_sound, play_sound};
use macroquad::prelude::*;
use std::default::Default;
use std::io::{Read, Write};
use std::time::Duration;
use std::{
    net::TcpStream,
    sync::{Arc, Mutex},
};
use std::{process, thread};

static PLAYER_WIDTH: f32 = 55.0;
static PLAYER_HEIGHT: f32 = 64.0;

enum GameInputState {
    Chat,
    Movement,
    Menu,
}

#[macroquad::main("Momentum")]
pub async fn main() {
    let mut game_input_state = GameInputState::Chat;

    println!("Loading assets...");
    let chat_sound = load_sound("res/chat.wav").await.unwrap();
    let _ = load_sound("res/stal.wav").await.unwrap();

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

    let map: Map = match bincode::decode_from_slice(&map_buf, bincode::config::standard()) {
        Ok((map, _)) => map,
        Err(e) => {
            panic!(
                "Failed to decode map: {}\nProbably server and client version mismatch",
                e
            )
        }
    };
    println!("Map fetched");

    let random_id = rand::random::<u32>();
    let mut player = Player::new(random_id, 0.0, 0.0);

    let mut will_send: u8 = 5;
    let player_packets: Arc<Mutex<Vec<PlayerPacket>>> = Arc::new(Mutex::new(Vec::new()));
    let mut time_played = 0.0;

    let mut pre_message = String::new();
    let mut delete_message_timer = 0.0;

    loop {
        if delete_message_timer <= 0.0 {
            player.message.clear();
            pre_message.clear();
        } else {
            delete_message_timer -= get_frame_time();
        }

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

        // Receive packets
        match stream.try_clone() {
            Ok(mut clone) => {
                clone.set_nonblocking(true).ok();
                match packet::receive_packet(&mut clone) {
                    Ok(packet) => {
                        let mut packets = player_packets.lock().unwrap();
                        if let Some(existing) = packets.iter_mut().find(|p| p.id == packet.id) {
                            println!("Received packet from {}, {:?}", packet.id, packet);
                            *existing = packet;
                        } else {
                            println!("Received packet from {}, {:?}", packet.id, packet);
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

        draw_text(
            pre_message.as_str(),
            player.x - 40.0,
            player.y - 50.0,
            20.0,
            GRAY,
        );
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
        for object in &map.objects {
            match object.kind {
                ObjectKind::StartLine => {
                    draw_rectangle(object.x - 32.0, object.y - 32.0, 32.0, 32.0, BLUE)
                }
                ObjectKind::FinishLine => {
                    draw_rectangle(object.x - 32.0, object.y - 32.0, 32.0, 32.0, BLUE)
                }
            }
        }

        // INPUT
        match game_input_state {
            GameInputState::Movement => {
                if is_key_pressed(KeyCode::T) {
                    game_input_state = GameInputState::Chat;
                }

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
                        player.vx *= 0.8;
                        if player.vx.abs() < 0.6 {
                            player.vx = 0.0;
                        }
                    } else {
                        player.is_still = true;
                        player.vx *= 0.9;
                        if player.vx.abs() < 0.6 {
                            player.vx = 0.0;
                        }
                    }
                }

                if is_key_pressed(KeyCode::Space) && player.on_ground {
                    player.vy = -12.0;
                    player.on_ground = false;
                }

                if player.vx == 0.0 {
                    player.is_still = true;
                } else if player.vx.abs() > 0.0 {
                    player.is_still = false;
                }

                if is_key_pressed(KeyCode::Escape) {
                    game_input_state = GameInputState::Menu;
                }
            }

            GameInputState::Chat => {
                if is_key_pressed(KeyCode::Backspace) {
                    pre_message.pop();
                }
                if is_key_pressed(KeyCode::Space) {
                    pre_message.push(' ');
                }

                let shift = is_key_down(KeyCode::LeftShift) || is_key_down(KeyCode::RightShift);

                {
                    if is_key_pressed(KeyCode::A) {
                        pre_message.push(if shift { 'A' } else { 'a' });
                    }
                    if is_key_pressed(KeyCode::B) {
                        pre_message.push(if shift { 'B' } else { 'b' });
                    }
                    if is_key_pressed(KeyCode::C) {
                        pre_message.push(if shift { 'C' } else { 'c' });
                    }
                    if is_key_pressed(KeyCode::D) {
                        pre_message.push(if shift { 'D' } else { 'd' });
                    }
                    if is_key_pressed(KeyCode::E) {
                        pre_message.push(if shift { 'E' } else { 'e' });
                    }
                    if is_key_pressed(KeyCode::F) {
                        pre_message.push(if shift { 'F' } else { 'f' });
                    }
                    if is_key_pressed(KeyCode::G) {
                        pre_message.push(if shift { 'G' } else { 'g' });
                    }
                    if is_key_pressed(KeyCode::H) {
                        pre_message.push(if shift { 'H' } else { 'h' });
                    }
                    if is_key_pressed(KeyCode::I) {
                        pre_message.push(if shift { 'I' } else { 'i' });
                    }
                    if is_key_pressed(KeyCode::J) {
                        pre_message.push(if shift { 'J' } else { 'j' });
                    }
                    if is_key_pressed(KeyCode::K) {
                        pre_message.push(if shift { 'K' } else { 'k' });
                    }
                    if is_key_pressed(KeyCode::L) {
                        pre_message.push(if shift { 'L' } else { 'l' });
                    }
                    if is_key_pressed(KeyCode::M) {
                        pre_message.push(if shift { 'M' } else { 'm' });
                    }
                    if is_key_pressed(KeyCode::N) {
                        pre_message.push(if shift { 'N' } else { 'n' });
                    }
                    if is_key_pressed(KeyCode::O) {
                        pre_message.push(if shift { 'O' } else { 'o' });
                    }
                    if is_key_pressed(KeyCode::P) {
                        pre_message.push(if shift { 'P' } else { 'p' });
                    }
                    if is_key_pressed(KeyCode::Q) {
                        pre_message.push(if shift { 'Q' } else { 'q' });
                    }
                    if is_key_pressed(KeyCode::R) {
                        pre_message.push(if shift { 'R' } else { 'r' });
                    }
                    if is_key_pressed(KeyCode::S) {
                        pre_message.push(if shift { 'S' } else { 's' });
                    }
                    if is_key_pressed(KeyCode::T) {
                        pre_message.push(if shift { 'T' } else { 't' });
                    }
                    if is_key_pressed(KeyCode::U) {
                        pre_message.push(if shift { 'U' } else { 'u' });
                    }
                    if is_key_pressed(KeyCode::V) {
                        pre_message.push(if shift { 'V' } else { 'v' });
                    }
                    if is_key_pressed(KeyCode::W) {
                        pre_message.push(if shift { 'W' } else { 'w' });
                    }
                    if is_key_pressed(KeyCode::X) {
                        pre_message.push(if shift { 'X' } else { 'x' });
                    }
                    if is_key_pressed(KeyCode::Y) {
                        pre_message.push(if shift { 'Y' } else { 'y' });
                    }
                    if is_key_pressed(KeyCode::Z) {
                        pre_message.push(if shift { 'Z' } else { 'z' });
                    }
                }

                if is_key_pressed(KeyCode::Enter) {
                    player.message = pre_message.clone();
                    pre_message.clear();
                    play_sound(&chat_sound, Default::default());
                    game_input_state = GameInputState::Movement;
                }

                delete_message_timer = 5.0;
            }
            GameInputState::Menu => {
                if is_key_pressed(KeyCode::Escape) {
                    game_input_state = GameInputState::Movement;
                }
                egui_macroquad::ui(|egui_ctx| {
                    egui::Window::new("Momentum").show(egui_ctx, |ui| {
                        ui.label(format!("Time played: {:.2}s", time_played));
                        ui.label(format!("FPS: {}", get_fps()));
                        if ui.button("Return to Game").clicked() {
                            game_input_state = GameInputState::Movement;
                        }
                        if ui.button("Quit game").clicked() {
                            process::exit(0);
                        };
                    });
                });
                egui_macroquad::draw();
            }
        }

        player.vy += 0.5;

        handle_collisions(&mut player, &map);
        time_played += get_frame_time();

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

    if let Some(_) = player.message.chars().nth(0) {
        draw_text(
            &player.message,
            player.x - 40.0,
            player.y - 50.0,
            20.0,
            BLACK,
        );
    }

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
                if tile.collision {
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
                if tile.collision {
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
