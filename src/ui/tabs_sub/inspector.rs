use eframe::egui;
use crate::biology::plant::PlantType;
use crate::biology::animal::{AnimalType, AiState};
use crate::ui::app::LifeSimApp;

pub fn draw_inspector(app: &mut LifeSimApp, ui: &mut egui::Ui) {
    if let Some(id) = app.selected_entity_id {
        let mut found = false;
        for chunk in app.world.chunks.values() {
            if let Some(plant) = chunk.plants.iter().find(|p| p.id == id) {
                draw_plant_inspector(plant, ui);
                found = true; break;
            }
            if let Some(animal) = chunk.animals.iter().find(|a| a.id == id) {
                draw_animal_inspector(app, animal, ui);
                found = true; break;
            }
        }
        if !found {
            ui.label(egui::RichText::new("Выбранная сущность погибла.").color(egui::Color32::GRAY));
            app.selected_entity_id = None;
        }
    } else {
        ui.centered_and_justified(|ui| {
            ui.label(egui::RichText::new("Нажмите 🔍 и кликните на карте, чтобы выбрать существо.").color(egui::Color32::DARK_GRAY));
        });
    }
}

fn draw_plant_inspector(plant: &crate::biology::plant::Plant, ui: &mut egui::Ui) {
    ui.horizontal(|ui| {
        ui.vertical(|ui| {
            ui.set_width(220.0);
            let icon = match plant.plant_type {
                PlantType::Grass => "🌿", PlantType::Shrub => "🌳",
                PlantType::Tree => "🌲", PlantType::Mushroom => "🍄",
            };
            ui.label(egui::RichText::new(format!("{} Растение ({:?})", icon, plant.plant_type)).strong());
            ui.label(format!("ID: {} | Возраст: {} тиков", plant.id, plant.age));
            let hp_color = if plant.health > 60.0 { egui::Color32::GREEN } else if plant.health > 30.0 { egui::Color32::YELLOW } else { egui::Color32::RED };
            ui.label(egui::RichText::new(format!("❤ Здоровье: {:.0}/100", plant.health)).color(hp_color));
            ui.label(format!("⚡ Энергия: {:.1}", plant.energy));
            if plant.is_poisonous { ui.label(egui::RichText::new("☠ ЯДОВИТОЕ").color(egui::Color32::from_rgb(200, 0, 200))); }
        });
        ui.separator();
        ui.vertical(|ui| {
            ui.label(egui::RichText::new("Геном").strong());
            let g = &plant.genome;
            ui.label(format!("Размер: {:.2} | Метаб.: {:.2}", g.size, g.metabolism));
            ui.label(format!("Листья: {:.2} | Корни: {:.2}", plant.leaf_size, plant.root_depth));
            ui.label(format!("Репрод. порог: {:.1}", g.reproduction_threshold));
            let c = egui::Color32::from_rgb((g.color_r*255.0) as u8, (g.color_g*255.0) as u8, (g.color_b*255.0) as u8);
            ui.horizontal(|ui| {
                ui.label("Цвет генома:");
                let (rect, _) = ui.allocate_exact_size(egui::vec2(20.0, 16.0), egui::Sense::hover());
                ui.painter().rect_filled(rect, 3.0, c);
            });
        });
    });
}

fn draw_animal_inspector(app: &LifeSimApp, animal: &crate::biology::animal::Animal, ui: &mut egui::Ui) {
    ui.horizontal(|ui| {
        ui.vertical(|ui| {
            ui.set_width(240.0);
            let icon = if animal.animal_type == AnimalType::Insect { "🐛" } else { "🐟" };
            ui.label(egui::RichText::new(format!("{} {:?}", icon, animal.animal_type)).strong());
            if let Some(spec) = app.world.evolution_manager.species_registry.get(&animal.genome.species_id) {
                ui.label(egui::RichText::new(format!("Вид: {}", spec.name)).color(egui::Color32::from_rgb(200, 160, 255)));
            }
            ui.label(format!("ID: {} | Возраст: {} | Поколение: {}", animal.id, animal.age, animal.genome.generation));
            let hp_max = animal.genome.max_health();
            let hp_color = if animal.health > hp_max*0.6 { egui::Color32::GREEN } else if animal.health > hp_max*0.3 { egui::Color32::YELLOW } else { egui::Color32::RED };
            ui.label(egui::RichText::new(format!("❤ HP: {:.0}/{:.0}", animal.health, hp_max)).color(hp_color));
            ui.label(format!("⚡ Энергия: {:.1}", animal.energy));
            ui.label(format!("😴 Усталость: {:.0}%", animal.fatigue));
            if animal.is_pregnant {
                ui.label(egui::RichText::new(format!("🤰 Беременна ({}/{})", animal.pregnancy_timer, animal.gestation_period())).color(egui::Color32::from_rgb(255, 180, 220)));
            }
            let state_color = match animal.current_state {
                AiState::Flee => egui::Color32::RED,
                AiState::Hunt => egui::Color32::from_rgb(255, 150, 50),
                AiState::Mate => egui::Color32::from_rgb(255, 100, 200),
                AiState::Forage => egui::Color32::YELLOW,
                AiState::Flock => egui::Color32::from_rgb(120, 200, 255),
                AiState::Rest => egui::Color32::from_rgb(150, 255, 150),
                AiState::Wander => egui::Color32::GRAY,
            };
            ui.label(egui::RichText::new(format!("🧠 Состояние: {:?}", animal.current_state)).color(state_color));
        });
        ui.separator();
        ui.vertical(|ui| {
            ui.label(egui::RichText::new("Геном").strong());
            let g = &animal.genome;
            ui.label(format!("Скорость: {:.2} | Размер: {:.2}", g.speed, g.size));
            ui.label(format!("Метаб.: {:.2} | Живучесть: {:.0}", g.metabolism, g.vitality));
            ui.label(format!("Диета: {:.2} ({})", g.diet, if g.diet > 0.6 { "Хищник" } else if g.diet > 0.35 { "Всеядный" } else { "Травоядный" }));
            ui.label(format!("Зрение: {:.0} | Социальность: {:.2}", g.vision_range, g.sociality));
            ui.label(format!("Страх: {:.2} | Агрессия: {:.2}", g.fear_threshold, g.aggression));
            ui.label(format!("Потомков: {:.0} | Кд.размнож.: {:.0}", g.offspring_count, g.reproduction_cooldown));
            ui.label(format!("Засуха-рез.: {:.2} | Буря-рез.: {:.2}", g.drought_resistance, g.storm_resistance));
            ui.label(format!("Акватич.: {:.2} | КПД пищев.: {:.2}", g.aquatic_adaptation, g.digestion_efficiency));
            let c = egui::Color32::from_rgb((g.color_r*255.0) as u8, (g.color_g*255.0) as u8, (g.color_b*255.0) as u8);
            ui.horizontal(|ui| {
                ui.label("Цвет:");
                let (rect, _) = ui.allocate_exact_size(egui::vec2(20.0, 16.0), egui::Sense::hover());
                ui.painter().rect_filled(rect, 3.0, c);
            });
        });
    });
}
