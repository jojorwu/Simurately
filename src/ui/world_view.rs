use eframe::egui;
use glam::Vec2;

use crate::ui::app::LifeSimApp;
use crate::ui::tool::Tool;
use crate::ui::render;

pub fn handle_camera_input(app: &mut LifeSimApp, response: &egui::Response, ui: &egui::Ui) {
    if response.dragged_by(egui::PointerButton::Secondary) {
        let d = response.drag_delta();
        app.camera_offset += Vec2::new(d.x, d.y);
    }
    let scroll_y = ui.input(|i| i.smooth_scroll_delta.y);
    if scroll_y != 0.0 {
        let factor = 1.0 + scroll_y * 0.002;
        app.camera_zoom = (app.camera_zoom * factor).clamp(0.04, 15.0);
    }
}

pub fn draw_world(app: &mut LifeSimApp, ctx: &egui::Context) {
    egui::CentralPanel::default()
        .frame(egui::Frame::default().fill(egui::Color32::from_rgb(8, 12, 16)))
        .show(ctx, |ui| {
            let size = ui.available_size();
            let (response, painter) = ui.allocate_painter(size, egui::Sense::click_and_drag());
            handle_camera_input(app, &response, ui);
            let center = Vec2::new(response.rect.center().x, response.rect.center().y);
            handle_tool_clicks(app, &response, center);
            draw_elements(app, &painter, response.rect);
        });
}

fn handle_tool_clicks(app: &mut LifeSimApp, response: &egui::Response, center: Vec2) {
    if response.clicked() {
        if let Some(pp) = response.interact_pointer_pos() {
            let wp = (Vec2::new(pp.x, pp.y) - center - app.camera_offset) / app.camera_zoom;
            app.apply_tool(wp);
        }
    }
}

fn draw_elements(app: &LifeSimApp, painter: &egui::Painter, rect: egui::Rect) {
    let center = Vec2::new(rect.center().x, rect.center().y);
    let to_screen = |wp: Vec2| -> egui::Pos2 {
        let p = center + app.camera_offset + wp * app.camera_zoom;
        egui::pos2(p.x, p.y)
    };
    let min_w = (Vec2::new(rect.left(), rect.top()) - center - app.camera_offset) / app.camera_zoom;
    let max_w = (Vec2::new(rect.right(), rect.bottom()) - center - app.camera_offset) / app.camera_zoom;
    let visible = app.world.get_visible_chunks(min_w, max_w);

    render::tile_renderer::draw_tiles(app, painter, &visible, &to_screen, rect);
    render::entity_renderer::draw_plants(app, painter, &visible, &to_screen, rect);
    render::entity_renderer::draw_animals(app, painter, &visible, &to_screen, rect);
    render::weather_renderer::draw_weather_effects(app, painter, rect, &to_screen);
    draw_brush_cursor(app, painter, rect);
}

fn draw_brush_cursor(app: &LifeSimApp, painter: &egui::Painter, rect: egui::Rect) {
    if matches!(app.active_tool, Tool::AddSoilEnergy | Tool::AddMoisture | Tool::Kill) {
        if let Some(mp) = painter.ctx().input(|i| i.pointer.hover_pos()) {
            if rect.contains(mp) {
                painter.circle_stroke(mp, app.brush_radius * app.camera_zoom,
                    egui::Stroke::new(1.5, egui::Color32::from_rgba_unmultiplied(255, 255, 255, 100)));
            }
        }
    }
}
