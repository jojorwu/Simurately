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
    let r_screen = (radius * app.camera_zoom).clamp(2.0, 45.0);
    let stroke_color = if plant.is_poisonous { egui::Color32::from_rgb(150, 0, 200) } else { egui::Color32::from_rgba_unmultiplied(0, 0, 0, 80) };
    let stroke = egui::Stroke::new(1.0, stroke_color);

    // Рисуем детали растения в зависимости от типа
    match plant.plant_type {
        PlantType::Grass => {
            painter.circle(sp, r_screen, fill, stroke);
            if app.camera_zoom > 2.0 {
                for i in 0..3 {
                    let angle = (i as f32 * 1.2).sin() * 0.5;
                    let p2 = sp + egui::vec2(angle.sin(), -angle.cos()) * r_screen * 1.3;
                    painter.line_segment([sp, p2], egui::Stroke::new(1.0, fill.linear_multiply(0.8)));
                }
            }
        }
        PlantType::Shrub => {
            painter.circle(sp, r_screen, fill, stroke);
            if app.camera_zoom > 1.5 {
                for i in 0..5 {
                    let angle = i as f32 * 1.25;
                    let p2 = sp + egui::vec2(angle.cos(), angle.sin()) * r_screen * 0.8;
                    painter.circle_filled(p2, r_screen * 0.4, fill.linear_multiply(0.9));
                }
            }
        }
        PlantType::Tree => {
            // Ствол
            let trunk_w = (r_screen * 0.3).max(1.0);
            painter.rect_filled(egui::Rect::from_center_size(sp, egui::vec2(trunk_w, r_screen * 2.5)), 0.0, egui::Color32::from_rgb(100, 70, 40));
            // Крона
            painter.circle(sp - egui::vec2(0.0, r_screen * 0.5), r_screen, fill, stroke);
            if app.camera_zoom > 1.0 {
                painter.circle_filled(sp - egui::vec2(r_screen * 0.4, r_screen * 0.8), r_screen * 0.6, fill.linear_multiply(1.1));
            }
        }
        PlantType::Mushroom => {
            // Ножка
            painter.rect_filled(egui::Rect::from_center_size(sp + egui::vec2(0.0, r_screen * 0.5), egui::vec2(r_screen * 0.6, r_screen)), 2.0, egui::Color32::from_rgb(220, 210, 190));
            // Шляпка
            painter.circle(sp, r_screen, fill, stroke);
            if app.camera_zoom > 2.0 {
                painter.circle_filled(sp + egui::vec2(r_screen * 0.3, -r_screen * 0.2), r_screen * 0.2, egui::Color32::WHITE);
                painter.circle_filled(sp + egui::vec2(-r_screen * 0.2, -r_screen * 0.4), r_screen * 0.15, egui::Color32::WHITE);
            }
        }
    }
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
    let r_screen = (size * app.camera_zoom).clamp(2.0, 45.0);

    // Рисуем тело в форме капли/овала, вытянутого по направлению движения
    let velocity_len = animal.velocity.length();
    let dir = if velocity_len > 0.1 { animal.velocity / velocity_len } else { Vec2::X };
    let forward = egui::vec2(dir.x, dir.y);
    let side = egui::vec2(-dir.y, dir.x);

    // Хвост/плавники
    if app.camera_zoom > 0.8 {
        let tail_p = sp - forward * r_screen * 0.8;
        let tail_side1 = tail_p + side * r_screen * 0.6 - forward * r_screen * 0.4;
        let tail_side2 = tail_p - side * r_screen * 0.6 - forward * r_screen * 0.4;
        painter.add(egui::Shape::convex_polygon(vec![tail_p, tail_side1, tail_side2], fill.linear_multiply(0.7), egui::Stroke::NONE));
    }

    // Тело
    painter.circle(sp, r_screen, fill, egui::Stroke::new(1.5, stroke));

    // Глаза
    if app.camera_zoom > 1.2 {
        let eye_offset = forward * r_screen * 0.5 + side * r_screen * 0.4;
        let eye_offset2 = forward * r_screen * 0.5 - side * r_screen * 0.4;
        painter.circle_filled(sp + eye_offset, r_screen * 0.2, egui::Color32::WHITE);
        painter.circle_filled(sp + eye_offset2, r_screen * 0.2, egui::Color32::WHITE);
        painter.circle_filled(sp + eye_offset + forward * r_screen * 0.05, r_screen * 0.1, egui::Color32::BLACK);
        painter.circle_filled(sp + eye_offset2 + forward * r_screen * 0.05, r_screen * 0.1, egui::Color32::BLACK);
    }

    if animal.velocity.length_squared() > 0.01 {
        let p_end = sp + forward * (r_screen + 3.0);
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
