use std::sync::atomic::AtomicU64;
use glam::Vec2;

use crate::biology::genome::Genome;
use crate::biology::plant::{Plant, PlantType};
use crate::biology::animal::Animal;
use crate::engine::tile::{Tile, TileType};
use crate::engine::climate::Climate;
use crate::engine::config::*;
use crate::engine::systems;

pub fn world_to_tile_index(pos: Vec2, chunk_id: (i32, i32)) -> usize {
    let tx = ((pos.x - chunk_id.0 as f32 * CHUNK_WORLD_SIZE) / TILE_SIZE).floor() as i32;
    let ty = ((pos.y - chunk_id.1 as f32 * CHUNK_WORLD_SIZE) / TILE_SIZE).floor() as i32;
    let tx = tx.clamp(0, CHUNK_SIZE as i32 - 1) as usize;
    let ty = ty.clamp(0, CHUNK_SIZE as i32 - 1) as usize;
    ty * CHUNK_SIZE + tx
}

pub struct Chunk {
    pub id: (i32, i32),
    pub tiles: Vec<Tile>,
    pub plants: Vec<Plant>,
    pub animals: Vec<Animal>,
    pub active: bool,
    pub last_updated_tile_idx: usize,
    pub plant_spatial_grid: Vec<Vec<usize>>,
    pub animal_spatial_grid: Vec<Vec<usize>>,
}

#[derive(Default)]
pub struct ChunkTickResult {
    pub migrated_animals: Vec<Animal>,
    pub spawned_seeds: Vec<(Vec2, PlantType, Genome)>,
    pub spawned_animals: Vec<Animal>,
    pub events: Vec<String>,
    pub died_animal_ids: Vec<u64>,
}

pub struct TickContext<'a> {
    pub mutation_rate: f32,
    pub next_entity_id: &'a AtomicU64,
    pub climate: &'a Climate,
    pub tick_count: u64,
}

impl Chunk {
    pub fn new(id: (i32, i32)) -> Self {
        let mut tiles = Vec::with_capacity(CHUNK_SIZE * CHUNK_SIZE);
        for ty in 0..CHUNK_SIZE {
            for tx in 0..CHUNK_SIZE {
                let gx = id.0 as f32 * CHUNK_WORLD_SIZE + tx as f32 * TILE_SIZE;
                let gy = id.1 as f32 * CHUNK_WORLD_SIZE + ty as f32 * TILE_SIZE;
                let noise = (gx * 0.0035).sin() * (gy * 0.0035).cos() + 0.4 * (gx * 0.012 + gy * 0.009).sin() + 0.15 * (gx * 0.05 - gy * 0.04).cos() + 0.08 * (gx * 0.1).sin();
                let tile_type = if noise < -0.20 { TileType::Water } else if noise < -0.10 { TileType::Sand } else if noise > 0.55 { TileType::Rock } else { TileType::Soil };
                let (energy, moisture, temp) = match tile_type { TileType::Soil => (100.0, 0.5, 0.3), TileType::Sand => (25.0, 0.15, 0.5), TileType::Water => (0.0, 1.0, 0.1), TileType::Rock => (0.0, 0.05, 0.4) };
                tiles.push(Tile { tile_type, energy, moisture, temperature: temp });
            }
        }
        Self {
            id, tiles, plants: Vec::new(), animals: Vec::new(), active: true, last_updated_tile_idx: 0,
            plant_spatial_grid: vec![Vec::new(); GRID_WIDTH * GRID_WIDTH],
            animal_spatial_grid: vec![Vec::new(); GRID_WIDTH * GRID_WIDTH],
        }
    }

    fn update_spatial_grids(&mut self) {
        use rayon::prelude::*;
        self.plant_spatial_grid.par_iter_mut().for_each(|cell| cell.clear());
        self.animal_spatial_grid.par_iter_mut().for_each(|cell| cell.clear());

        let left = self.id.0 as f32 * CHUNK_WORLD_SIZE;
        let top = self.id.1 as f32 * CHUNK_WORLD_SIZE;

        // Для растений (обычно их много)
        for (i, p) in self.plants.iter().enumerate() {
            let gx = ((p.position.x - left) / GRID_CELL_SIZE).floor() as i32;
            let gy = ((p.position.y - top) / GRID_CELL_SIZE).floor() as i32;
            if gx >= 0 && gx < GRID_WIDTH as i32 && gy >= 0 && gy < GRID_WIDTH as i32 {
                self.plant_spatial_grid[(gy * GRID_WIDTH as i32 + gx) as usize].push(i);
            }
        }
        // Для животных
        for (i, a) in self.animals.iter().enumerate() {
            let gx = ((a.position.x - left) / GRID_CELL_SIZE).floor() as i32;
            let gy = ((a.position.y - top) / GRID_CELL_SIZE).floor() as i32;
            if gx >= 0 && gx < GRID_WIDTH as i32 && gy >= 0 && gy < GRID_WIDTH as i32 {
                self.animal_spatial_grid[(gy * GRID_WIDTH as i32 + gx) as usize].push(i);
            }
        }
    }

    pub fn tick(&mut self, mutation_rate: f32, next_entity_id: &AtomicU64, climate: &Climate, tick_count: u64, _bucket_index: usize) -> ChunkTickResult {
        if !self.active { return ChunkTickResult::default(); }
        let ctx = TickContext { mutation_rate, next_entity_id, climate, tick_count };
        let mut result = ChunkTickResult::default();

        self.update_spatial_grids();
        systems::tile_system::update_tiles(&mut self.tiles, &mut self.last_updated_tile_idx, ctx.climate);
        systems::plant_system::update_plants(&mut self.plants, &mut self.tiles, self.id, &ctx, &mut result);
        systems::animal_system::update_animals(&mut self.animals, &mut self.plants, &self.tiles, self.id, &self.animal_spatial_grid, &self.plant_spatial_grid, &ctx, &mut result);

        result
    }
}
