use crate::map::{Map, TileKind};
use egui;
use macroquad::prelude::*;
use serde_json;
use std::{fs, process};

const TILE_SIZE: f32 = 16.0;
const MAP_WIDTH: u32 = 256;
const MAP_HEIGHT: u32 = 256;

#[macroquad::main("Mapping Tool")]
pub async fn main() {
    let mut map = Map::new(MAP_WIDTH, MAP_HEIGHT);
    let mut mode = 0;

    loop {
        clear_background(WHITE);

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
                if ui.button("Draw").clicked() {
                    mode = 0;
                }
                if ui.button("Delete").clicked() {
                    mode = 1;
                }
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
            let (mx, my) = mouse_position();
            let x = (mx / TILE_SIZE).floor() as usize;
            let y = (my / TILE_SIZE).floor() as usize;

            if x < MAP_WIDTH as usize && y < MAP_HEIGHT as usize {
                if mode == 0 {
                    map.tiles[y][x].kind = TileKind::Rock;
                    map.tiles[y][x].collision = true;
                }
                if mode == 1 {
                    map.tiles[y][x].kind = TileKind::Empty;
                    map.tiles[y][x].collision = false;
                }
            }
        }

        draw_text("[q] to save and quit", 600.0, 1000.0, 16.0, RED);

        next_frame().await;
    }
}
