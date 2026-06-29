use eframe::egui;
use glam::Vec2;
use crate::ui::app::LifeSimApp;
use crate::engine::tile::TileType;
use crate::engine::config::{CHUNK_WORLD_SIZE, TILE_SIZE, CHUNK_SIZE};

pub fn draw_tiles(
    app: &LifeSimApp,
    painter: &egui::Painter,
    visible: &[&crate::engine::chunk::Chunk],
    to_screen: &impl Fn(Vec2) -> egui::Pos2,
    rect: egui::Rect,
) {
    for chunk in visible {
        let cx = chunk.id.0 as f32 * CHUNK_WORLD_SIZE;
        let cy = chunk.id.1 as f32 * CHUNK_WORLD_SIZE;
        let chunk_rect = egui::Rect::from_two_pos(to_screen(Vec2::new(cx, cy)), to_screen(Vec2::new(cx + CHUNK_WORLD_SIZE, cy + CHUNK_WORLD_SIZE)));
        if app.camera_zoom < 0.20 {
            painter.rect_filled(chunk_rect, 0.0, egui::Color32::from_rgb(20, 28, 20));
        } else {
            draw_chunk_tiles(app, painter, chunk, cx, cy, rect, to_screen);
            painter.rect_stroke(chunk_rect, 0.0, egui::Stroke::new(0.5, egui::Color32::from_rgba_unmultiplied(60, 60, 80, 60)));
        }
    }
}

fn draw_chunk_tiles(
    app: &LifeSimApp,
    painter: &egui::Painter,
    chunk: &crate::engine::chunk::Chunk,
    cx: f32,
    cy: f32,
    rect: egui::Rect,
    to_screen: &impl Fn(Vec2) -> egui::Pos2,
) {
    for ty in 0..CHUNK_SIZE {
        for tx in 0..CHUNK_SIZE {
            let tile_tl = to_screen(Vec2::new(cx + tx as f32 * TILE_SIZE, cy + ty as f32 * TILE_SIZE));
            let tile_br = to_screen(Vec2::new(cx + (tx + 1) as f32 * TILE_SIZE, cy + (ty + 1) as f32 * TILE_SIZE));
            let tile_rect = egui::Rect::from_two_pos(tile_tl, tile_br);
            if rect.intersects(tile_rect) {
                let tile = &chunk.tiles[ty * CHUNK_SIZE + tx];
                painter.rect_filled(tile_rect, 0.0, get_tile_color(app, tile));
                if app.camera_zoom > 0.8 {
                    painter.rect_stroke(tile_rect, 0.0, egui::Stroke::new(0.5, egui::Color32::from_rgba_unmultiplied(255, 255, 255, 20)));
                }
            }
        }
    }
}

fn get_tile_color(app: &LifeSimApp, tile: &crate::engine::tile::Tile) -> egui::Color32 {
    if app.settings.show_tile_energy && tile.tile_type == TileType::Soil {
        let f = (tile.energy / 200.0).clamp(0.0, 1.0);
        egui::Color32::from_rgb((20.0 + f * 60.0) as u8, (60.0 + f * 100.0) as u8, (20.0 + f * 30.0) as u8)
    } else if app.settings.show_tile_moisture {
        let f = tile.moisture.clamp(0.0, 1.0);
        egui::Color32::from_rgb(10, (40.0 + f * 80.0) as u8, (80.0 + f * 120.0) as u8)
    } else {
        match tile.tile_type {
            TileType::Water => {
                let wf = app.world.climate.flood_level;
                egui::Color32::from_rgb((20.0 + wf * 10.0) as u8, (50.0 + wf * 20.0) as u8, (120.0 + wf * 40.0) as u8)
            }
            TileType::Sand => egui::Color32::from_rgb(180, 165, 110),
            TileType::Rock => egui::Color32::from_rgb(80, 75, 70),
            TileType::Soil => {
                let f = (tile.energy / 200.0).clamp(0.0, 1.0);
                let m = tile.moisture.clamp(0.0, 1.0);
                egui::Color32::from_rgb((25.0 + f * 15.0) as u8, (40.0 + f * 40.0 + m * 15.0) as u8, (20.0 + m * 20.0) as u8)
            }
        }
    }
}
