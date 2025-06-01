use crate::map::{Map, Object, ObjectKind, TileKind};
use egui;
use macroquad::prelude::*;
use serde_json;
use std::{fs, io, process};

const TILE_SIZE: f32 = 16.0;
const MAP_WIDTH: u32 = 256;
const MAP_HEIGHT: u32 = 256;

#[macroquad::main("Mapping Tool")]
pub async fn main() {
    let mut map: Map;

    let mut buf = String::new();
    println!("Load map? y/n");
    io::stdin().read_line(&mut buf).unwrap();
    match buf.trim() {
        "y" => {
            map = serde_json::from_str(&fs::read_to_string("map.json").unwrap()).unwrap();
        }
        &_ => {
            map = Map::new(MAP_WIDTH, MAP_HEIGHT);
        }
    }

    let mut tile_kind = TileKind::Grass;
    let mut drawing = true;
    let mut can_collide = true;

    let mut cx = 0.0;
    let mut cy = 0.0;
    loop {
        clear_background(WHITE);
        let camera = Camera2D {
            target: vec2(cx, cy),
            zoom: vec2(1.0 / screen_width() * 2.0, 1.0 / screen_height() * 2.0),
            ..Default::default()
        };
        set_camera(&camera);
        if is_key_down(KeyCode::W) {
            cy -= 1.0;
        }
        if is_key_down(KeyCode::S) {
            cy += 1.0;
        }
        if is_key_down(KeyCode::A) {
            cx -= 1.0;
        }
        if is_key_down(KeyCode::D) {
            cx += 1.0;
        }
        for (y, row) in map.tiles.iter().enumerate() {
            for (x, tile) in row.iter().enumerate() {
                match tile.kind {
                    TileKind::Grass => draw_rectangle(
                        x as f32 * TILE_SIZE,
                        y as f32 * TILE_SIZE,
                        TILE_SIZE,
                        TILE_SIZE,
                        GREEN,
                    ),
                    TileKind::Rock => draw_rectangle(
                        x as f32 * TILE_SIZE,
                        y as f32 * TILE_SIZE,
                        TILE_SIZE,
                        TILE_SIZE,
                        BLACK,
                    ),
                    TileKind::Empty => {}
                }
            }
        }

        egui_macroquad::ui(|egui_ctx| {
            egui::Window::new("Mapping Tool").show(egui_ctx, |ui| {
                ui.heading("Drawing Mode");
                ui.checkbox(&mut drawing, "Enabled");
                ui.checkbox(&mut can_collide, "Can collide");
                ui.radio_value(&mut tile_kind, TileKind::Rock, "Rock");
                ui.radio_value(&mut tile_kind, TileKind::Grass, "Grass");
                ui.radio_value(&mut tile_kind, TileKind::Empty, "Empty");
                ui.label("--------------------");
                ui.checkbox(&mut drawing, "Drawing enabled");

                ui.label("--------------------");
                if ui.button("Save and quit").clicked() {
                    fs::write("map.json", serde_json::to_string(&map).unwrap()).unwrap();
                    process::exit(0);
                }
            });
        });
        egui_macroquad::draw();
        // Draw tiles

        // Mouse input

        if is_mouse_button_down(MouseButton::Left) || is_mouse_button_down(MouseButton::Right) {
            let mouse_screen = vec2(mouse_position().0, mouse_position().1);
            let mouse_world = camera.screen_to_world(mouse_screen);

            let x = (mouse_world.x / TILE_SIZE).floor() as usize;
            let y = (mouse_world.y / TILE_SIZE).floor() as usize;

            if x < MAP_WIDTH as usize && y < MAP_HEIGHT as usize {
                if drawing {
                    match tile_kind {
                        TileKind::Rock => {
                            map.tiles[y][x].kind = TileKind::Rock;
                            map.tiles[y][x].collision = can_collide;
                        }
                        TileKind::Grass => {
                            map.tiles[y][x].kind = TileKind::Grass;
                            map.tiles[y][x].collision = can_collide;
                        }
                        TileKind::Empty => {
                            map.tiles[y][x].kind = TileKind::Empty;
                            map.tiles[y][x].collision = false;
                        }
                    }
                } else {
                    map.objects.push(Object {
                        x: x as f32,
                        y: y as f32,
                        kind: ObjectKind::StartLine,
                    });
                }
            }
        }

        draw_text("[q] to save and quit", 600.0, 1000.0, 16.0, RED);

        next_frame().await;
    }
}
