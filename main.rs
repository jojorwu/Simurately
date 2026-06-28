mod engine;
mod biology;
mod engine;
mod stats;

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
        Box::new(|cc| Box::new(ui::LifeSimApp::new(cc)) as Box<dyn eframe::App>),
    )
}
