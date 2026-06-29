pub mod engine;
pub mod biology;
pub mod ui;

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

    #[test]
    fn test_animal_reproduction() {
        use crate::biology::animal::{Animal, AnimalType};
        use crate::biology::genome::Genome;
        use glam::Vec2;

        let mut animal = Animal::new(1, AnimalType::Insect, Genome::default_insect(), Vec2::ZERO);
        animal.energy = 500.0;
        animal.is_pregnant = true;
        animal.pregnancy_timer = animal.gestation_period();

        let res = animal.update(false, &[], &[], 20.0, 0.5, 0.1);
        assert!(res.offspring_count > 0);
        assert!(!animal.is_pregnant);
    }
}

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
