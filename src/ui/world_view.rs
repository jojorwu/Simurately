use eframe::egui;
use glam::Vec2;
use rand::Rng;

use crate::ui::app::LifeSimApp;
use crate::ui::tool::Tool;
use crate::biology::plant::PlantType;
use crate::biology::animal::{AnimalType, AiState};
use crate::engine::tile::TileType;
use crate::engine::chunk::{CHUNK_WORLD_SIZE, TILE_SIZE, CHUNK_SIZE};
use crate::engine::climate::WeatherType;

pub fn handle_camera_input(app: &mut LifeSimApp, response: &egui::Response, ui: &egui::Ui) {
    if response.dragged_by(egui::PointerButton::Secondary) {
        let d = response.drag_delta();
        app.camera_offset += Vec2::new(d.x, d.y);
    }
    let scroll_y = ui.input(|i| i.smooth_scroll_delta.y);
    if scroll_y != 0.0 {
        let factor = 1.0 + scroll_y * 0.002;
        app.camera_zoom = (app.camera_zoom * factor).clamp(0.04, 15.0);
    }
}

pub fn draw_world(app: &mut LifeSimApp, ctx: &egui::Context) {
    egui::CentralPanel::default()
        .frame(egui::Frame::default().fill(egui::Color32::from_rgb(8, 12, 16)))
        .show(ctx, |ui| {
            let size = ui.available_size();
            let (response, painter) = ui.allocate_painter(size, egui::Sense::click_and_drag());
            handle_camera_input(app, &response, ui);
            let center = Vec2::new(response.rect.center().x, response.rect.center().y);
            handle_tool_clicks(app, &response, center);
            draw_elements(app, &painter, response.rect);
        });
}

fn handle_tool_clicks(app: &mut LifeSimApp, response: &egui::Response, center: Vec2) {
    if response.clicked() {
        if let Some(pp) = response.interact_pointer_pos() {
            let wp = (Vec2::new(pp.x, pp.y) - center - app.camera_offset) / app.camera_zoom;
            app.apply_tool(wp);
        }
    }
}

fn draw_elements(app: &LifeSimApp, painter: &egui::Painter, rect: egui::Rect) {
    let center = Vec2::new(rect.center().x, rect.center().y);
    let to_screen = |wp: Vec2| -> egui::Pos2 {
        let p = center + app.camera_offset + wp * app.camera_zoom;
        egui::pos2(p.x, p.y)
    };
    let min_w = (Vec2::new(rect.left(), rect.top()) - center - app.camera_offset) / app.camera_zoom;
    let max_w = (Vec2::new(rect.right(), rect.bottom()) - center - app.camera_offset) / app.camera_zoom;
    let visible = app.world.get_visible_chunks(min_w, max_w);

    draw_layers(app, painter, &visible, rect, &to_screen);
    draw_weather_effects(app, painter, rect, &to_screen);
    draw_brush_cursor(app, painter, rect);
}

fn draw_layers(
    app: &LifeSimApp,
    painter: &egui::Painter,
    visible: &[&crate::engine::chunk::Chunk],
    rect: egui::Rect,
    to_screen: &impl Fn(Vec2) -> egui::Pos2,
) {
    draw_tiles(app, painter, visible, rect, to_screen);
    draw_plants(app, painter, visible, rect, to_screen);
    draw_animals(app, painter, visible, rect, to_screen);
}

fn draw_brush_cursor(app: &LifeSimApp, painter: &egui::Painter, rect: egui::Rect) {
    if matches!(app.active_tool, Tool::AddSoilEnergy | Tool::AddMoisture | Tool::Kill) {
        if let Some(mp) = painter.ctx().input(|i| i.pointer.hover_pos()) {
            if rect.contains(mp) {
                painter.circle_stroke(mp, app.brush_radius * app.camera_zoom,
                    egui::Stroke::new(1.5, egui::Color32::from_rgba_unmultiplied(255, 255, 255, 100)));
            }
        }
    }
}

fn default_tile_color(app: &LifeSimApp, tile_type: TileType, energy: f32, moisture: f32) -> egui::Color32 {
    match tile_type {
        TileType::Water => {
            let wf = app.world.climate.flood_level;
            egui::Color32::from_rgb((20.0 + wf * 10.0) as u8, (50.0 + wf * 20.0) as u8, (120.0 + wf * 40.0) as u8)
        }
        TileType::Sand => egui::Color32::from_rgb(180, 165, 110),
        TileType::Rock => egui::Color32::from_rgb(80, 75, 70),
        TileType::Soil => {
            let f = (energy / 200.0).clamp(0.0, 1.0);
            let m = moisture.clamp(0.0, 1.0);
            egui::Color32::from_rgb((25.0 + f * 15.0) as u8, (40.0 + f * 40.0 + m * 15.0) as u8, (20.0 + m * 20.0) as u8)
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
        default_tile_color(app, tile.tile_type, tile.energy, tile.moisture)
    }
}

pub fn draw_tiles(
    app: &LifeSimApp,
    painter: &egui::Painter,
    visible: &[&crate::engine::chunk::Chunk],
    rect: egui::Rect,
    to_screen: &impl Fn(Vec2) -> egui::Pos2,
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

pub fn draw_plants(
    app: &LifeSimApp,
    painter: &egui::Painter,
    visible: &[&crate::engine::chunk::Chunk],
    rect: egui::Rect,
    to_screen: &impl Fn(Vec2) -> egui::Pos2,
) {
    for chunk in visible {
        for plant in &chunk.plants {
            let sp = to_screen(Vec2::new(plant.position.0, plant.position.1));
            if rect.contains(sp) {
                draw_single_plant(app, painter, plant, sp);
            }
        }
    }
}

fn plant_radius_and_color(app: &LifeSimApp, plant: &crate::biology::plant::Plant) -> (f32, egui::Color32) {
    let radius = match plant.plant_type {
        PlantType::Grass => 2.5 + plant.genome.size * 0.8,
        PlantType::Shrub => 4.0 + plant.genome.size * 1.2,
        PlantType::Tree => 5.5 + plant.genome.size * 2.0,
        PlantType::Mushroom => 3.0 + plant.genome.size * 0.7,
    };
    let color = if app.settings.show_genome_colors {
        egui::Color32::from_rgb((plant.genome.color_r * 255.0) as u8, (plant.genome.color_g * 255.0) as u8, (plant.genome.color_b * 255.0) as u8)
    } else {
        match plant.plant_type {
            PlantType::Grass => egui::Color32::from_rgb(60, 160, 50),
            PlantType::Shrub => egui::Color32::from_rgb(40, 120, 40),
            PlantType::Tree => egui::Color32::from_rgb(30, 90, 30),
            PlantType::Mushroom => egui::Color32::from_rgb(200, 140, 180),
        }
    };
    (radius, color)
}

fn draw_single_plant(app: &LifeSimApp, painter: &egui::Painter, plant: &crate::biology::plant::Plant, sp: egui::Pos2) {
    let (radius, fill) = plant_radius_and_color(app, plant);
    let r_screen = (radius * app.camera_zoom).clamp(2.0, 35.0);
    let stroke = if plant.is_poisonous { egui::Color32::from_rgb(150, 0, 200) } else { egui::Color32::from_rgba_unmultiplied(0, 0, 0, 80) };
    painter.circle(sp, r_screen, fill, egui::Stroke::new(1.0, stroke));
    draw_plant_bars_and_selection(app, painter, plant, sp, r_screen);
}

fn draw_plant_bars_and_selection(app: &LifeSimApp, painter: &egui::Painter, plant: &crate::biology::plant::Plant, sp: egui::Pos2, r_screen: f32) {
    if app.settings.show_health_bars && plant.health < 90.0 {
        let bar_w = r_screen * 2.0;
        let bar_tl = sp + egui::vec2(-r_screen, r_screen + 2.0);
        painter.rect_filled(egui::Rect::from_min_size(bar_tl, egui::vec2(bar_w, 2.0)), 0.0, egui::Color32::DARK_RED);
        painter.rect_filled(egui::Rect::from_min_size(bar_tl, egui::vec2(bar_w * plant.health / 100.0, 2.0)), 0.0, egui::Color32::GREEN);
    }
    if Some(plant.id) == app.selected_entity_id {
        painter.circle_stroke(sp, r_screen + 4.0, egui::Stroke::new(2.0, egui::Color32::from_rgb(255, 220, 80)));
    }
}

pub fn draw_animals(
    app: &LifeSimApp,
    painter: &egui::Painter,
    visible: &[&crate::engine::chunk::Chunk],
    rect: egui::Rect,
    to_screen: &impl Fn(Vec2) -> egui::Pos2,
) {
    for chunk in visible {
        for animal in &chunk.animals {
            let sp = to_screen(animal.position);
            if rect.contains(sp) {
                draw_single_animal(app, painter, animal, sp);
            }
        }
    }
}

fn animal_size_and_fill(app: &LifeSimApp, animal: &crate::biology::animal::Animal) -> (f32, egui::Color32) {
    let size = match animal.animal_type {
        AnimalType::Insect => 3.0 + animal.genome.size * 0.8,
        AnimalType::Fish => 4.5 + animal.genome.size * 1.5,
    };
    let fill = if app.settings.show_genome_colors {
        egui::Color32::from_rgb((animal.genome.color_r * 255.0) as u8, (animal.genome.color_g * 255.0) as u8, (animal.genome.color_b * 255.0) as u8)
    } else {
        match animal.animal_type {
            AnimalType::Insect => egui::Color32::from_rgb(180, 100, 60),
            AnimalType::Fish => egui::Color32::from_rgb(60, 120, 200),
        }
    };
    (size, fill)
}

fn animal_stroke(animal: &crate::biology::animal::Animal) -> egui::Color32 {
    match animal.current_state {
        AiState::Flee => egui::Color32::from_rgb(255, 100, 100),
        AiState::Hunt => egui::Color32::from_rgb(255, 50, 50),
        AiState::Forage => egui::Color32::from_rgb(200, 200, 50),
        _ => egui::Color32::from_rgba_unmultiplied(0, 0, 0, 120),
    }
}

fn draw_single_animal(app: &LifeSimApp, painter: &egui::Painter, animal: &crate::biology::animal::Animal, sp: egui::Pos2) {
    let (size, fill) = animal_size_and_fill(app, animal);
    let stroke = animal_stroke(animal);
    let r_screen = (size * app.camera_zoom).clamp(2.0, 35.0);
    painter.circle(sp, r_screen, fill, egui::Stroke::new(1.5, stroke));

    if animal.velocity.length_squared() > 0.01 {
        let dir = animal.velocity.normalize();
        let p_end = sp + egui::vec2(dir.x, dir.y) * (r_screen + 3.0);
        painter.line_segment([sp, p_end], egui::Stroke::new(1.5, stroke));
    }
    draw_animal_bars_and_selection(app, painter, animal, sp, r_screen);
}

fn draw_animal_bars_and_selection(app: &LifeSimApp, painter: &egui::Painter, animal: &crate::biology::animal::Animal, sp: egui::Pos2, r_screen: f32) {
    let max_hp = animal.genome.max_health();
    if app.settings.show_health_bars && animal.health < max_hp * 0.9 {
        let bar_w = r_screen * 2.0;
        let bar_tl = sp + egui::vec2(-r_screen, r_screen + 2.0);
        painter.rect_filled(egui::Rect::from_min_size(bar_tl, egui::vec2(bar_w, 2.0)), 0.0, egui::Color32::DARK_RED);
        painter.rect_filled(egui::Rect::from_min_size(bar_tl, egui::vec2(bar_w * animal.health / max_hp, 2.0)), 0.0, egui::Color32::GREEN);
    }
    if Some(animal.id) == app.selected_entity_id {
        painter.circle_stroke(sp, r_screen + 4.0, egui::Stroke::new(2.0, egui::Color32::from_rgb(255, 220, 80)));
    }
}

fn draw_rain_effects(app: &LifeSimApp, painter: &egui::Painter, rect: egui::Rect) {
    let mut rng = rand::thread_rng();
    let is_storm = app.world.climate.current_weather == WeatherType::Stormy;
    let count = if is_storm { 55 } else { 30 };
    let wind_tilt = if is_storm { -0.4 } else { -0.1 };
    for _ in 0..count {
        let rx = rng.gen_range(rect.left()..rect.right());
        let ry = rng.gen_range(rect.top()..rect.bottom());
        let length = rng.gen_range(10.0..22.0);
        let p1 = egui::pos2(rx, ry);
        let p2 = egui::pos2(rx + wind_tilt * length, ry + length);
        painter.line_segment([p1, p2], egui::Stroke::new(1.0, egui::Color32::from_rgba_unmultiplied(100, 140, 230, 70)));
    }
    painter.rect_filled(rect, 0.0, egui::Color32::from_rgba_unmultiplied(20, 40, 100, 15));
}

fn draw_drought_effects(painter: &egui::Painter, rect: egui::Rect) {
    painter.rect_filled(rect, 0.0, egui::Color32::from_rgba_unmultiplied(200, 90, 10, 18));
    let mut rng = rand::thread_rng();
    for _ in 0..8 {
        let rx = rng.gen_range(rect.left()..rect.right());
        let ry = rng.gen_range(rect.top()..rect.bottom());
        let r = rng.gen_range(20.0..60.0);
        painter.circle_stroke(egui::pos2(rx, ry), r, egui::Stroke::new(1.0, egui::Color32::from_rgba_unmultiplied(255, 200, 100, 12)));
    }
}

fn draw_blizzard_effects(painter: &egui::Painter, rect: egui::Rect) {
    painter.rect_filled(rect, 0.0, egui::Color32::from_rgba_unmultiplied(180, 200, 255, 22));
    let mut rng = rand::thread_rng();
    for _ in 0..60 {
        let rx = rng.gen_range(rect.left()..rect.right());
        let ry = rng.gen_range(rect.top()..rect.bottom());
        painter.circle_filled(egui::pos2(rx, ry), rng.gen_range(1.0..3.5), egui::Color32::from_rgba_unmultiplied(255, 255, 255, 120));
    }
}

fn draw_weather_effects(
    app: &LifeSimApp,
    painter: &egui::Painter,
    rect: egui::Rect,
    to_screen: &impl Fn(Vec2) -> egui::Pos2,
) {
    match app.world.climate.current_weather {
        WeatherType::Rainy | WeatherType::Stormy => draw_rain_effects(app, painter, rect),
        WeatherType::Drought | WeatherType::Heatwave => draw_drought_effects(painter, rect),
        WeatherType::Blizzard => draw_blizzard_effects(painter, rect),
        _ => {}
    }
    draw_lightning(app, painter, rect, to_screen);
}

fn draw_lightning(app: &LifeSimApp, painter: &egui::Painter, rect: egui::Rect, to_screen: &impl Fn(Vec2) -> egui::Pos2) {
    if let Some((strike_pos, age)) = app.world.climate.lightning_strike {
        let mut rng = rand::thread_rng();
        painter.rect_filled(rect, 0.0, egui::Color32::from_rgba_unmultiplied(255, 255, 200, (((age as f32 * 30.0) as u8).min(220)) / 6));
        let ss = to_screen(strike_pos);
        let mut p = egui::pos2(ss.x + (rng.gen_range(0.0..1.0) - 0.5) * 60.0, rect.top());
        for step in 1..=7 {
            let t = step as f32 / 7.0;
            let np = egui::pos2(ss.x + (rng.gen_range(0.0..1.0) - 0.5) * 30.0 * (1.0 - t), rect.top() + (ss.y - rect.top()) * t);
            painter.line_segment([p, np], egui::Stroke::new(2.5, egui::Color32::from_rgb(255, 240, 100)));
            p = np;
        }
    }
}
