use eframe::egui;
use glam::Vec2;
use crate::ui::app::LifeSimApp;
use crate::biology::plant::PlantType;
use crate::biology::animal::{AnimalType, AiState};

pub fn draw_plants(
    app: &LifeSimApp,
    painter: &egui::Painter,
    visible: &[&crate::engine::chunk::Chunk],
    to_screen: &impl Fn(Vec2) -> egui::Pos2,
    rect: egui::Rect,
) {
    for chunk in visible {
        for plant in &chunk.plants {
            let sp = to_screen(plant.position);
            if rect.contains(sp) {
                draw_single_plant(app, painter, plant, sp);
            }
        }
    }
}

pub fn draw_animals(
    app: &LifeSimApp,
    painter: &egui::Painter,
    visible: &[&crate::engine::chunk::Chunk],
    to_screen: &impl Fn(Vec2) -> egui::Pos2,
    rect: egui::Rect,
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

fn draw_single_plant(app: &LifeSimApp, painter: &egui::Painter, plant: &crate::biology::plant::Plant, sp: egui::Pos2) {
    let radius = match plant.plant_type {
        PlantType::Grass => 2.5 + plant.genome.size * 0.8,
        PlantType::Shrub => 4.0 + plant.genome.size * 1.2,
        PlantType::Tree => 5.5 + plant.genome.size * 2.0,
        PlantType::Mushroom => 3.0 + plant.genome.size * 0.7,
    };
    let fill = if app.settings.show_genome_colors {
        egui::Color32::from_rgb((plant.genome.color_r * 255.0) as u8, (plant.genome.color_g * 255.0) as u8, (plant.genome.color_b * 255.0) as u8)
    } else {
        match plant.plant_type {
            PlantType::Grass => egui::Color32::from_rgb(60, 160, 50),
            PlantType::Shrub => egui::Color32::from_rgb(40, 120, 40),
            PlantType::Tree => egui::Color32::from_rgb(30, 90, 30),
            PlantType::Mushroom => egui::Color32::from_rgb(200, 140, 180),
        }
    };
    let r_screen = (radius * app.camera_zoom).clamp(2.0, 35.0);
    let stroke = if plant.is_poisonous { egui::Color32::from_rgb(150, 0, 200) } else { egui::Color32::from_rgba_unmultiplied(0, 0, 0, 80) };
    painter.circle(sp, r_screen, fill, egui::Stroke::new(1.0, stroke));
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

fn draw_single_animal(app: &LifeSimApp, painter: &egui::Painter, animal: &crate::biology::animal::Animal, sp: egui::Pos2) {
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
    let stroke = match animal.current_state {
        AiState::Flee => egui::Color32::RED,
        AiState::Hunt => egui::Color32::from_rgb(255, 50, 50),
        AiState::Forage => egui::Color32::from_rgb(200, 200, 50),
        _ => egui::Color32::from_rgba_unmultiplied(0, 0, 0, 120),
    };
    let r_screen = (size * app.camera_zoom).clamp(2.0, 35.0);
    painter.circle(sp, r_screen, fill, egui::Stroke::new(1.5, stroke));
    if animal.velocity.length_squared() > 0.01 {
        let dir = animal.velocity.normalize();
        let p_end = sp + egui::vec2(dir.x, dir.y) * (r_screen + 3.0);
        painter.line_segment([sp, p_end], egui::Stroke::new(1.5, stroke));
    }
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
