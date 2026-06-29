use eframe::egui;
use glam::Vec2;
use rand::Rng;
use crate::ui::app::LifeSimApp;
use crate::engine::climate::WeatherType;

pub fn draw_weather_effects(app: &LifeSimApp, painter: &egui::Painter, rect: egui::Rect, to_screen: &impl Fn(Vec2) -> egui::Pos2) {
    match app.world.climate.current_weather {
        WeatherType::Rainy | WeatherType::Stormy => draw_rain_effects(app, painter, rect),
        WeatherType::Drought | WeatherType::Heatwave => draw_drought_effects(painter, rect),
        WeatherType::Blizzard => draw_blizzard_effects(painter, rect),
        _ => {}
    }
    draw_lightning(app, painter, rect, to_screen);
}

fn draw_rain_effects(app: &LifeSimApp, painter: &egui::Painter, rect: egui::Rect) {
    let mut rng = rand::thread_rng();
    let is_storm = app.world.climate.current_weather == WeatherType::Stormy;
    let count = if is_storm { 55 } else { 30 };
    let wind_tilt = if is_storm { -0.4 } else { -0.1 };
    for _ in 0..count {
        let rx = rng.gen_range(rect.left()..rect.right());
        let ry = rng.gen_range(rect.top()..rect.bottom());
        let length = rng.gen_range(10.0..22.0);
        let p1 = egui::pos2(rx, ry);
        let p2 = egui::pos2(rx + wind_tilt * length, ry + length);
        painter.line_segment([p1, p2], egui::Stroke::new(1.0, egui::Color32::from_rgba_unmultiplied(100, 140, 230, 70)));
    }
    painter.rect_filled(rect, 0.0, egui::Color32::from_rgba_unmultiplied(20, 40, 100, 15));
}

fn draw_drought_effects(painter: &egui::Painter, rect: egui::Rect) {
    painter.rect_filled(rect, 0.0, egui::Color32::from_rgba_unmultiplied(200, 90, 10, 18));
    let mut rng = rand::thread_rng();
    for _ in 0..8 {
        let rx = rng.gen_range(rect.left()..rect.right());
        let ry = rng.gen_range(rect.top()..rect.bottom());
        let r = rng.gen_range(20.0..60.0);
        painter.circle_stroke(egui::pos2(rx, ry), r, egui::Stroke::new(1.0, egui::Color32::from_rgba_unmultiplied(255, 200, 100, 12)));
    }
}

fn draw_blizzard_effects(painter: &egui::Painter, rect: egui::Rect) {
    painter.rect_filled(rect, 0.0, egui::Color32::from_rgba_unmultiplied(180, 200, 255, 22));
    let mut rng = rand::thread_rng();
    for _ in 0..60 {
        let rx = rng.gen_range(rect.left()..rect.right());
        let ry = rng.gen_range(rect.top()..rect.bottom());
        painter.circle_filled(egui::pos2(rx, ry), rng.gen_range(1.0..3.5), egui::Color32::from_rgba_unmultiplied(255, 255, 255, 120));
    }
}

fn draw_lightning(app: &LifeSimApp, painter: &egui::Painter, rect: egui::Rect, to_screen: &impl Fn(Vec2) -> egui::Pos2) {
    if let Some((strike_pos, age)) = app.world.climate.lightning_strike {
        let mut rng = rand::thread_rng();
        painter.rect_filled(rect, 0.0, egui::Color32::from_rgba_unmultiplied(255, 255, 200, (((age as f32 * 30.0) as u8).min(220)) / 6));
        let ss = to_screen(strike_pos);
        let mut p = egui::pos2(ss.x + (rng.gen_range(0.0..1.0) - 0.5) * 60.0, rect.top());
        for step in 1..=7 {
            let t = step as f32 / 7.0;
            let np = egui::pos2(ss.x + (rng.gen_range(0.0..1.0) - 0.5) * 30.0 * (1.0 - t), rect.top() + (ss.y - rect.top()) * t);
            painter.line_segment([p, np], egui::Stroke::new(2.5, egui::Color32::from_rgb(255, 240, 100)));
            p = np;
        }
    }
}
