use eframe::egui;
use egui_plot::{Line, Plot, PlotPoints, Legend};
use crate::ui::app::LifeSimApp;
use crate::biology::animal::AnimalType;

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
        for msg in app.world.logs.entries.iter().rev().take(80) {
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
