use eframe::egui;

use crate::ui::app::LifeSimApp;
use crate::ui::tool::Tool;
use crate::ui::tabs::{
    BottomTab, draw_inspector, draw_species_tab, draw_populations_tab,
    draw_climate_tab, draw_events_tab,
};
use crate::engine::climate::WeatherType;

// Верхняя панель — погода и глобальная инфо
pub fn draw_top_panel(app: &mut LifeSimApp, ctx: &egui::Context) {
    egui::TopBottomPanel::top("top_panel")
        .frame(egui::Frame::default()
            .fill(egui::Color32::from_rgb(12, 16, 20))
            .inner_margin(egui::Margin::symmetric(10.0, 6.0)))
        .show(ctx, |ui| {
            ui.horizontal(|ui| {
                // Левая часть — название
                ui.add(egui::Label::new(
                    egui::RichText::new("🧬 Эволюционная Симуляция")
                        .size(16.0)
                        .color(egui::Color32::from_rgb(150, 220, 140))
                ));
                ui.separator();

                let (plants, insects, fish) = app.world.population_counts();
                ui.label(egui::RichText::new(format!("🌿 {}", plants)).color(egui::Color32::from_rgb(80, 200, 80)));
                ui.label(egui::RichText::new(format!("🐛 {}", insects)).color(egui::Color32::from_rgb(220, 210, 80)));
                ui.label(egui::RichText::new(format!("🐟 {}", fish)).color(egui::Color32::from_rgb(80, 160, 240)));
                ui.separator();

                let active_sp = app.world.evolution_manager.species_registry.values().filter(|s| s.active).count();
                ui.label(egui::RichText::new(format!("🔬 Видов: {}", active_sp)).color(egui::Color32::from_rgb(200, 160, 255)));
                ui.separator();

                // Правая часть — погода и тик
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label(egui::RichText::new(format!("⏱ Тик: {}", app.world.tick_count))
                        .color(egui::Color32::GRAY));
                    ui.separator();
                    let c = &app.world.climate;
                    let season = ["🌱Весна","☀Лето","🍂Осень","❄Зима"][c.season as usize % 4];
                    ui.label(egui::RichText::new(format!("{} | {} T:{:.2} H:{:.2}", season, c.current_weather.display_name(), c.temperature, c.humidity))
                        .color(egui::Color32::from_rgb(200, 200, 255)));
                });
            });
        });
}

// Левая боковая панель — управление
pub fn draw_left_panel(app: &mut LifeSimApp, ctx: &egui::Context) {
    egui::SidePanel::left("left_panel")
        .width_range(200.0..=260.0)
        .frame(egui::Frame::default()
            .fill(egui::Color32::from_rgb(18, 22, 28))
            .inner_margin(egui::Margin::same(8.0)))
        .show(ctx, |ui| {
            ui.add_space(4.0);
            ui.label(egui::RichText::new("▶ Управление").strong().color(egui::Color32::from_rgb(150, 220, 140)));
            ui.separator();

            ui.horizontal(|ui| {
                let btn_text = if app.paused { "▶ Запустить" } else { "⏸ Пауза" };
                if ui.button(btn_text).clicked() { app.paused = !app.paused; }
                if ui.button("⏭ Шаг").clicked() { app.world.tick(); }
            });

            ui.add(egui::Slider::new(&mut app.ticks_per_frame, 1..=30)
                .text("Тиков/кадр")
                .clamp_to_range(true));
            ui.add(egui::Slider::new(&mut app.world.mutation_rate, 0.0..=0.5)
                .text("Мутация")
                .clamp_to_range(true));

            ui.add_space(8.0);
            ui.label(egui::RichText::new("🌍 Принудительная погода").color(egui::Color32::from_rgb(180, 180, 255)));
            ui.horizontal(|ui| {
                if ui.small_button("☀").on_hover_text("Ясно").clicked() {
                    app.world.climate.next_weather = WeatherType::Sunny;
                    app.world.climate.transition_progress = 0.0;
                }
                if ui.small_button("🌧").on_hover_text("Дождь").clicked() {
                    app.world.climate.next_weather = WeatherType::Rainy;
                    app.world.climate.transition_progress = 0.0;
                }
                if ui.small_button("⚡").on_hover_text("Гроза").clicked() {
                    app.world.climate.next_weather = WeatherType::Stormy;
                    app.world.climate.transition_progress = 0.0;
                }
                if ui.small_button("🔥").on_hover_text("Засуха").clicked() {
                    app.world.climate.next_weather = WeatherType::Drought;
                    app.world.climate.transition_progress = 0.0;
                }
                if ui.small_button("❄").on_hover_text("Буран").clicked() {
                    app.world.climate.next_weather = WeatherType::Blizzard;
                    app.world.climate.transition_progress = 0.0;
                }
            });

            ui.add_space(8.0);
            ui.label(egui::RichText::new("🛠 Инструменты").strong().color(egui::Color32::from_rgb(150, 220, 140)));
            ui.separator();

            let tools = [
                Tool::Select, Tool::SpawnGrass, Tool::SpawnShrub, Tool::SpawnTree,
                Tool::SpawnMushroom, Tool::SpawnInsect, Tool::SpawnFish,
                Tool::AddSoilEnergy, Tool::AddMoisture, Tool::Kill,
            ];
            for tool in tools {
                ui.selectable_value(&mut app.active_tool, tool, tool.label());
            }

            if matches!(app.active_tool, Tool::AddSoilEnergy | Tool::AddMoisture | Tool::Kill) {
                ui.add(egui::Slider::new(&mut app.brush_radius, 5.0..=120.0).text("Радиус"));
            }

            ui.add_space(8.0);
            ui.label(egui::RichText::new("👁 Отображение").strong().color(egui::Color32::from_rgb(150, 220, 140)));
            ui.separator();
            ui.checkbox(&mut app.settings.show_genome_colors, "Генетические цвета");
            ui.checkbox(&mut app.settings.show_health_bars, "Полоски здоровья");
            ui.checkbox(&mut app.settings.show_ai_states, "Состояние ИИ");
            ui.checkbox(&mut app.settings.show_tile_energy, "Энергия почвы");
            ui.checkbox(&mut app.settings.show_tile_moisture, "Влажность почвы");

            ui.add_space(8.0);
            if ui.button("🔄 Сбросить симуляцию").clicked() {
                *app = LifeSimApp::default();
            }
            ui.label(egui::RichText::new("Перетаскивание: ПКМ\nМасштаб: колесо мыши").color(egui::Color32::GRAY).size(11.0));

            ui.add_space(8.0);
            ui.label(egui::RichText::new(format!("💀 Гибелей: {}", app.world.stats.total_deaths)).color(egui::Color32::from_rgb(255, 120, 120)));
            ui.label(egui::RichText::new(format!("👶 Рождений: {}", app.world.stats.total_births)).color(egui::Color32::from_rgb(120, 255, 120)));
            ui.label(egui::RichText::new(format!("🌿 Видообр.: {}", app.world.stats.total_speciations)).color(egui::Color32::from_rgb(200, 140, 255)));
        });
}

// Нижняя панель — вкладки мониторинга
pub fn draw_bottom_panel(app: &mut LifeSimApp, ctx: &egui::Context) {
    egui::TopBottomPanel::bottom("bottom_panel")
        .resizable(true)
        .default_height(230.0)
        .min_height(140.0)
        .frame(egui::Frame::default()
            .fill(egui::Color32::from_rgb(14, 18, 23))
            .inner_margin(egui::Margin::symmetric(8.0, 6.0)))
        .show(ctx, |ui| {
            // Вкладки
            ui.horizontal(|ui| {
                ui.selectable_value(&mut app.bottom_tab, BottomTab::Inspector,   "🔬 Инспектор");
                ui.selectable_value(&mut app.bottom_tab, BottomTab::Species,     "🧬 Виды");
                ui.selectable_value(&mut app.bottom_tab, BottomTab::Populations, "📊 Популяция");
                ui.selectable_value(&mut app.bottom_tab, BottomTab::Climate,     "🌍 Климат");
                ui.selectable_value(&mut app.bottom_tab, BottomTab::Events,      "📜 События");
                ui.selectable_value(&mut app.bottom_tab, BottomTab::Settings,    "⚙ Настройки");
            });
            ui.separator();

            match app.bottom_tab {
                BottomTab::Inspector => draw_inspector(app, ui),
                BottomTab::Species => draw_species_tab(app, ui),
                BottomTab::Populations => draw_populations_tab(app, ui),
                BottomTab::Climate => draw_climate_tab(app, ui),
                BottomTab::Events => draw_events_tab(app, ui),
                BottomTab::Settings => draw_settings_tab(app, ui),
            }
        });
}

pub fn draw_settings_tab(app: &mut LifeSimApp, ui: &mut egui::Ui) {
    ui.label(egui::RichText::new("⚙ Настройки игры").strong());
    ui.separator();
    
    ui.checkbox(&mut app.settings.show_plants, "Отображать растения");
    ui.checkbox(&mut app.settings.show_fish, "Отображать рыб");
    ui.checkbox(&mut app.settings.show_animals, "Отображать животных");
    ui.checkbox(&mut app.settings.show_rules, "Отображать правила");
    
    ui.add_space(8.0);
    ui.label(egui::RichText::new("👁 Визуализация").strong());
    ui.separator();
    ui.checkbox(&mut app.settings.show_tile_energy, "Энергия почвы");
    ui.checkbox(&mut app.settings.show_tile_moisture, "Влажность почвы");
    ui.checkbox(&mut app.settings.show_genome_colors, "Генетические цвета");
    ui.checkbox(&mut app.settings.show_health_bars, "Полоски здоровья");
    ui.checkbox(&mut app.settings.show_ai_states, "Состояние ИИ");
}
