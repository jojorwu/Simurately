#![allow(dead_code, unused_imports)]
use eframe::egui;
use glam::Vec2;
use rand::Rng;

use crate::engine::world::World;
use crate::engine::tile::TileType;
use crate::engine::config::{CHUNK_WORLD_SIZE, TILE_SIZE, CHUNK_SIZE};
use crate::biology::plant::PlantType;
use crate::biology::animal::AnimalType;

use crate::ui::tool::Tool;
use crate::ui::tabs::BottomTab;
use crate::ui::settings::GameSettings;
use crate::ui::world_view;
use crate::ui::panels;

#[derive(PartialEq)]
pub enum AppState {
    Menu,
    Simulation,
}

pub struct LifeSimApp {
    pub(crate) state: AppState,
    pub(crate) world: World,
    pub(crate) paused: bool,
    pub(crate) ticks_per_frame: usize,

    // Камера
    pub(crate) camera_offset: Vec2,
    pub(crate) camera_zoom: f32,

    // Инструменты
    pub(crate) active_tool: Tool,
    pub(crate) brush_radius: f32,

    // UI
    pub(crate) selected_entity_id: Option<u64>,
    pub(crate) bottom_tab: BottomTab,
    pub(crate) settings: GameSettings,

    // Настройки меню
    pub(crate) world_size_chunks: i32,
}

impl Default for LifeSimApp {
    fn default() -> Self {
        Self {
            state: AppState::Menu,
            world: World::new(),
            paused: false,
            ticks_per_frame: 2,
            camera_offset: Vec2::ZERO,
            camera_zoom: 1.0,
            active_tool: Tool::Select,
            brush_radius: 35.0,
            selected_entity_id: None,
            bottom_tab: BottomTab::Populations,
            settings: GameSettings::default(),
            world_size_chunks: 3,
        }
    }
}

impl LifeSimApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let mut style = (*cc.egui_ctx.style()).clone();
        style.visuals = egui::Visuals::dark();
        style.visuals.window_rounding = egui::Rounding::same(8.0);
        style.visuals.panel_fill = egui::Color32::from_rgb(15, 18, 22);
        style.visuals.widgets.noninteractive.bg_fill = egui::Color32::from_rgb(22, 28, 35);
        cc.egui_ctx.set_style(style);
        Self::default()
    }

    pub(crate) fn start_simulation(&mut self) {
        let mut world = World::new();
        let half_size = self.world_size_chunks / 2;
        for cx in -half_size..=(self.world_size_chunks - half_size - 1) {
            for cy in -half_size..=(self.world_size_chunks - half_size - 1) {
                world.add_chunk(cx, cy);
            }
        }
        self.world = world;
        self.state = AppState::Simulation;
    }

    pub(crate) fn apply_tool(&mut self, world_pos: Vec2) {
        match self.active_tool {
            Tool::Select => {
                let mut best_id = None;
                let mut min_dist = 25.0f32;
                let (cx, cy) = crate::engine::world::world_to_chunk_coords(world_pos);
                for dx in -1..=1 {
                    for dy in -1..=1 {
                        if let Some(chunk) = self.world.chunks.get(&(cx + dx, cy + dy)) {
                            for p in &chunk.plants {
                                let d = world_pos.distance(p.position);
                                if d < min_dist { min_dist = d; best_id = Some(p.id); }
                            }
                            for a in &chunk.animals {
                                let d = world_pos.distance(a.position);
                                if d < min_dist { min_dist = d; best_id = Some(a.id); }
                            }
                        }
                    }
                }
                self.selected_entity_id = best_id;
            }
            Tool::SpawnGrass    => { self.world.spawn_plant(PlantType::Grass, world_pos, None); }
            Tool::SpawnShrub    => { self.world.spawn_plant(PlantType::Shrub, world_pos, None); }
            Tool::SpawnTree     => { self.world.spawn_plant(PlantType::Tree, world_pos, None); }
            Tool::SpawnMushroom => { self.world.spawn_plant(PlantType::Mushroom, world_pos, None); }
            Tool::SpawnInsect   => { self.world.spawn_animal(AnimalType::Insect, world_pos, None); }
            Tool::SpawnFish     => { self.world.spawn_animal(AnimalType::Fish, world_pos, None); }
            Tool::AddSoilEnergy => {
                let (cx, cy) = crate::engine::world::world_to_chunk_coords(world_pos);
                let brush_chunks = (self.brush_radius / CHUNK_WORLD_SIZE).ceil() as i32;
                for dx in -brush_chunks..=brush_chunks {
                    for dy in -brush_chunks..=brush_chunks {
                        if let Some(chunk) = self.world.chunks.get_mut(&(cx + dx, cy + dy)) {
                            let chunk_left = chunk.id.0 as f32 * CHUNK_WORLD_SIZE;
                            let chunk_top  = chunk.id.1 as f32 * CHUNK_WORLD_SIZE;
                    for ty in 0..CHUNK_SIZE {
                        for tx in 0..CHUNK_SIZE {
                            let tp = Vec2::new(chunk_left + tx as f32 * TILE_SIZE + TILE_SIZE * 0.5,
                                               chunk_top  + ty as f32 * TILE_SIZE + TILE_SIZE * 0.5);
                            if tp.distance(world_pos) <= self.brush_radius {
                                let idx = ty * CHUNK_SIZE + tx;
                                chunk.tiles[idx].energy = (chunk.tiles[idx].energy + 50.0).min(250.0);
                            }
                        }
                    }
                        }
                    }
                }
            }
            Tool::AddMoisture => {
                let (cx, cy) = crate::engine::world::world_to_chunk_coords(world_pos);
                let brush_chunks = (self.brush_radius / CHUNK_WORLD_SIZE).ceil() as i32;
                for dx in -brush_chunks..=brush_chunks {
                    for dy in -brush_chunks..=brush_chunks {
                        if let Some(chunk) = self.world.chunks.get_mut(&(cx + dx, cy + dy)) {
                            let chunk_left = chunk.id.0 as f32 * CHUNK_WORLD_SIZE;
                            let chunk_top  = chunk.id.1 as f32 * CHUNK_WORLD_SIZE;
                    for ty in 0..CHUNK_SIZE {
                        for tx in 0..CHUNK_SIZE {
                            let tp = Vec2::new(chunk_left + tx as f32 * TILE_SIZE + TILE_SIZE * 0.5,
                                               chunk_top  + ty as f32 * TILE_SIZE + TILE_SIZE * 0.5);
                            if tp.distance(world_pos) <= self.brush_radius {
                                let idx = ty * CHUNK_SIZE + tx;
                                chunk.tiles[idx].moisture = (chunk.tiles[idx].moisture + 0.3).min(1.0);
                            }
                        }
                    }
                        }
                    }
                }
            }
            Tool::Kill => {
                let br = self.brush_radius;
                let (cx, cy) = crate::engine::world::world_to_chunk_coords(world_pos);
                let brush_chunks = (self.brush_radius / CHUNK_WORLD_SIZE).ceil() as i32;
                for dx in -brush_chunks..=brush_chunks {
                    for dy in -brush_chunks..=brush_chunks {
                        if let Some(chunk) = self.world.chunks.get_mut(&(cx + dx, cy + dy)) {
                            chunk.plants.retain(|p| p.position.distance(world_pos) > br);
                            chunk.animals.retain(|a| a.position.distance(world_pos) > br);
                        }
                    }
                }
            }
        }
    }
}

impl eframe::App for LifeSimApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        match self.state {
            AppState::Menu => self.draw_menu(ctx),
            AppState::Simulation => self.draw_simulation(ctx),
        }
    }
}

impl LifeSimApp {
    fn draw_menu(&mut self, ctx: &egui::Context) {
        egui::CentralPanel::default().frame(egui::Frame::default().fill(egui::Color32::from_rgb(10, 15, 20))).show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                ui.add_space(120.0);
                ui.heading(egui::RichText::new("🧬 Эволюционная Симуляция v2")
                    .size(48.0)
                    .strong()
                    .color(egui::Color32::from_rgb(150, 220, 140)));

                ui.add_space(10.0);
                ui.label(egui::RichText::new("Жизнь найдёт выход").italics().color(egui::Color32::GRAY));

                ui.add_space(60.0);

                ui.scope(|ui| {
                    ui.visuals_mut().widgets.inactive.bg_fill = egui::Color32::from_rgb(20, 25, 35);
                    ui.visuals_mut().widgets.hovered.bg_fill = egui::Color32::from_rgb(30, 40, 55);

                    egui::Frame::group(ui.style())
                        .fill(egui::Color32::from_rgb(15, 20, 30))
                        .stroke(egui::Stroke::new(1.0, egui::Color32::from_rgb(40, 50, 70)))
                        .rounding(12.0)
                        .inner_margin(24.0)
                        .show(ui, |ui| {
                            ui.set_width(320.0);
                            ui.label(egui::RichText::new("Конфигурация мира").strong().size(18.0));
                            ui.add_space(12.0);

                            ui.horizontal(|ui| {
                                ui.label("Размер (чанки):");
                                ui.add(egui::Slider::new(&mut self.world_size_chunks, 1..=10).show_value(true));
                            });

                            ui.add_space(32.0);

                            let start_btn = ui.add_sized([280.0, 50.0], egui::Button::new(
                                egui::RichText::new("🚀 ЗАПУСТИТЬ").size(22.0).strong()
                            ).fill(egui::Color32::from_rgb(50, 120, 60)));

                            if start_btn.clicked() {
                                self.start_simulation();
                            }
                        });
                });
            });
        });
    }

    fn draw_simulation(&mut self, ctx: &egui::Context) {
        if !self.paused {
            for _ in 0..self.ticks_per_frame {
                self.world.tick();
            }
            ctx.request_repaint();
        }

        panels::draw_top_panel(self, ctx);
        panels::draw_left_panel(self, ctx);
        panels::draw_bottom_panel(self, ctx);
        world_view::draw_world(self, ctx);
    }
}
