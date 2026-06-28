use eframe::egui;
use egui_plot::{Line, Plot, PlotPoints, Legend};

use crate::biology::{PlantType, AnimalType, AiState};
use crate::render::app::LifeSimApp;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BottomTab {
    Inspector,
    Species,
    Populations,
    Events,
    Climate,
    Settings,
}

pub fn draw_inspector(app: &mut LifeSimApp, ui: &mut egui::Ui) {
    if let Some(id) = app.selected_entity_id {
        let mut found = false;
        for chunk in app.world.chunks.values() {
            if let Some(plant) = chunk.plants.iter().find(|p| p.id == id) {
                ui.horizontal(|ui| {
                    ui.vertical(|ui| {
                        ui.set_width(220.0);
                        let type_icon = match plant.plant_type {
                            PlantType::Grass => "🌿", PlantType::Shrub => "🌳",
                            PlantType::Tree => "🌲", PlantType::Mushroom => "🍄",
                        };
                        ui.label(egui::RichText::new(format!("{} Растение ({:?})", type_icon, plant.plant_type)).strong());
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
                found = true; break;
            }
            if let Some(animal) = chunk.animals.iter().find(|a| a.id == id) {
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

pub fn draw_species_tab(app: &mut LifeSimApp, ui: &mut egui::Ui) {
    let mut species_list: Vec<_> = app.world.evolution_manager.species_registry.values().collect();
    species_list.sort_by(|a, b| b.population.cmp(&a.population));

    egui::ScrollArea::vertical().show(ui, |ui| {
        egui::Grid::new("species_grid")
            .num_columns(7)
            .striped(true)
            .spacing([10.0, 4.0])
            .show(ui, |ui| {
                ui.label(egui::RichText::new("Вид").strong());
                ui.label(egui::RichText::new("Тип").strong());
                ui.label(egui::RichText::new("Попул.").strong());
                ui.label(egui::RichText::new("Рождено").strong());
                ui.label(egui::RichText::new("Поколение").strong());
                ui.label(egui::RichText::new("Статус").strong());
                ui.label(egui::RichText::new("Цвет").strong());
                ui.end_row();

                for spec in &species_list {
                    let name_color = if spec.active { egui::Color32::WHITE } else { egui::Color32::DARK_GRAY };
                    ui.label(egui::RichText::new(&spec.name).color(name_color));
                    let type_label = match spec.base_type { AnimalType::Insect => "🐛 Насекомое", AnimalType::Fish => "🐟 Рыба" };
                    ui.label(type_label);
                    let pop_color = if spec.population == 0 { egui::Color32::DARK_GRAY } else if spec.population < 5 { egui::Color32::YELLOW } else { egui::Color32::GREEN };
                    ui.label(egui::RichText::new(format!("{}", spec.population)).color(pop_color));
                    ui.label(format!("{}", spec.total_born));
                    ui.label(format!("{}", spec.avg_genome.generation));
                    if spec.active {
                        ui.label(egui::RichText::new("● Живёт").color(egui::Color32::GREEN));
                    } else {
                        ui.label(egui::RichText::new("✕ Вымер").color(egui::Color32::RED));
                    }
                    let c = egui::Color32::from_rgb(spec.color[0], spec.color[1], spec.color[2]);
                    let (rect, _) = ui.allocate_exact_size(egui::vec2(20.0, 14.0), egui::Sense::hover());
                    ui.painter().rect_filled(rect, 3.0, c);
                    ui.end_row();
                }
            });
    });
}

pub fn draw_populations_tab(app: &mut LifeSimApp, ui: &mut egui::Ui) {
    ui.horizontal(|ui| {
        // Линейный график популяций
        ui.vertical(|ui| {
            ui.set_min_width(350.0);
            ui.label(egui::RichText::new("📈 Динамика популяций").strong());
            let p_line = Line::new(PlotPoints::from_ys_f32(&app.world.stats.plant_count_history))
                .color(egui::Color32::from_rgb(60, 200, 80)).name("Растения");
            let i_line = Line::new(PlotPoints::from_ys_f32(&app.world.stats.insect_count_history))
                .color(egui::Color32::from_rgb(230, 200, 40)).name("Насекомые");
            let f_line = Line::new(PlotPoints::from_ys_f32(&app.world.stats.fish_count_history))
                .color(egui::Color32::from_rgb(60, 140, 240)).name("Рыбы");
            Plot::new("pop_plot")
                .height(160.0)
                .legend(Legend::default())
                .show(ui, |pu| { pu.line(p_line); pu.line(i_line); pu.line(f_line); });
        });
        ui.separator();

        // График биоразнообразия
        ui.vertical(|ui| {
            ui.set_min_width(250.0);
            ui.label(egui::RichText::new("🌿 Биоразнообразие (видов)").strong());
            let bd_line = Line::new(PlotPoints::from_ys_f32(&app.world.stats.biodiversity_history))
                .color(egui::Color32::from_rgb(200, 120, 255)).name("Видов");
            Plot::new("biodiv_plot")
                .height(160.0)
                .legend(Legend::default())
                .show(ui, |pu| { pu.line(bd_line); });
        });
    });
}

pub fn draw_climate_tab(app: &LifeSimApp, ui: &mut egui::Ui) {
    let c = &app.world.climate;
    let season_names = ["🌱 Весна", "☀ Лето", "🍂 Осень", "❄ Зима"];
    ui.horizontal(|ui| {
        ui.vertical(|ui| {
            ui.set_width(200.0);
            ui.label(egui::RichText::new("Климат").strong());
            ui.label(format!("Сезон: {}", season_names[c.season as usize % 4]));
            ui.label(format!("Тик сезона: {}/3000", c.season_timer));
            ui.label(format!("Погода: {}", c.current_weather.display_name()));
            ui.label(format!("→ {}", c.next_weather.display_name()));
            ui.label(format!("Переход: {:.0}%", c.transition_progress * 100.0));
        });
        ui.separator();
        ui.vertical(|ui| {
            ui.set_width(200.0);
            ui.label(egui::RichText::new("Параметры среды").strong());
            ui.label(format!("🌡 Температура: {:.2}", c.temperature));
            ui.label(format!("💧 Влажность: {:.2}", c.humidity));
            ui.label(format!("☀ Освещённость: {:.2}", c.sunlight));
            ui.label(format!("💨 Ветер: {:.2}", c.wind_speed));
            ui.label(format!("🌊 Уровень паводка: {:.3}", c.flood_level));
        });
    });
}

pub fn draw_events_tab(app: &LifeSimApp, ui: &mut egui::Ui) {
    egui::ScrollArea::vertical().stick_to_bottom(true).show(ui, |ui| {
        for msg in app.world.logs.iter().rev().take(80) {
            let color = if msg.contains("ЭВОЛЮЦИЯ") || msg.contains("ВЫМИРАНИЕ") {
                egui::Color32::from_rgb(200, 150, 255)
            } else if msg.contains("⚡") {
                egui::Color32::from_rgb(255, 230, 80)
            } else if msg.contains("ПОГОДА") || msg.contains("СЕЗОН") {
                egui::Color32::from_rgb(120, 200, 255)
            } else {
                egui::Color32::GRAY
            };
            ui.label(egui::RichText::new(msg).color(color).size(11.0));
        }
    });
}
