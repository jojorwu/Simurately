use eframe::egui;
use crate::ui::app::LifeSimApp;

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
