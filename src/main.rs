pub mod engine;
pub mod biology;
pub mod ui;

use eframe::egui;

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("🧬 Эволюционная Симуляция Жизни v2")
            .with_inner_size(egui::vec2(1400.0, 860.0))
            .with_min_inner_size(egui::vec2(1000.0, 600.0)),
        ..Default::default()
    };

    eframe::run_native(
        "Life Simulation v2",
        options,
        Box::new(|cc| Box::new(ui::app::LifeSimApp::new(cc)) as Box<dyn eframe::App>),
    )
}

#[cfg(test)]
mod tests {
    use super::ui::app::LifeSimApp;

    #[test]
    fn test_app_creation() {
        let mut app = LifeSimApp::default();
        app.start_simulation();
        assert!(app.world.chunks.len() > 0);
    }

    #[test]
    fn test_app_tick() {
        let mut app = LifeSimApp::default();
        app.world.tick();
    }
}
