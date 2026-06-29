use std::sync::atomic::{AtomicU64, Ordering};
use glam::Vec2;
use rand::Rng;

use crate::biology::genome::Genome;
use crate::biology::plant::{Plant, PlantType};
use crate::biology::animal::{Animal, AnimalUpdateResult};
use crate::engine::tile::{Tile, TileType};
use crate::engine::climate::Climate;
use crate::engine::config::*;

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
}

#[derive(Default)]
pub struct ChunkTickResult {
    pub migrated_animals: Vec<Animal>,
    pub spawned_seeds: Vec<(f32, f32, PlantType, Genome)>,
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
        Self { id, tiles, plants: Vec::new(), animals: Vec::new(), active: true, last_updated_tile_idx: 0 }
    }

    pub fn tick(&mut self, mutation_rate: f32, next_entity_id: &AtomicU64, climate: &Climate, tick_count: u64, _bucket_index: usize) -> ChunkTickResult {
        if !self.active { return ChunkTickResult::default(); }
        let ctx = TickContext { mutation_rate, next_entity_id, climate, tick_count };
        let mut result = ChunkTickResult::default();

        self.update_tiles(ctx.climate);
        self.update_plants(&ctx, &mut result);
        self.update_animals(&ctx, &mut result);

        result
    }

    fn update_tiles(&mut self, climate: &Climate) {
        let mut i = self.last_updated_tile_idx;
        for _ in 0..TILES_PER_TICK {
            let tile = &mut self.tiles[i % (CHUNK_SIZE * CHUNK_SIZE)];
            let m_regen = match tile.tile_type { TileType::Soil => climate.humidity * 0.15 - 0.02, TileType::Sand => climate.humidity * 0.05 - 0.02, TileType::Water => 0.01, TileType::Rock => 0.0 };
            tile.moisture = (tile.moisture + m_regen).clamp(0.0, 1.0);
            tile.temperature = tile.temperature * 0.95 + climate.temperature * 0.05;
            let e_regen = match tile.tile_type { TileType::Soil => (0.1 + climate.sunlight * 0.1 + climate.humidity * 0.05).max(0.0), TileType::Sand => (0.02 + climate.sunlight * 0.02).max(0.0), _ => 0.0 };
            tile.energy = (tile.energy + e_regen).min(if tile.tile_type == TileType::Soil { 200.0 } else { 50.0 });
            i += 1;
        }
        self.last_updated_tile_idx = i % (CHUNK_SIZE * CHUNK_SIZE);
    }

    fn update_plants(&mut self, ctx: &TickContext, result: &mut ChunkTickResult) {
        if ctx.tick_count % PLANT_UPDATE_INTERVAL != 0 { return; }
        let mut dead_indices = Vec::new();
        for (i, plant) in self.plants.iter_mut().enumerate() {
            let idx = world_to_tile_index(Vec2::new(plant.position.0, plant.position.1), self.id);
            let tile = &self.tiles[idx];
            let (seed, absorbed) = plant.update(tile.energy, (tile.temperature + ctx.climate.temperature) / 2.0, (tile.moisture + ctx.climate.humidity) / 2.0, ctx.climate.sunlight);
            self.tiles[idx].energy = (self.tiles[idx].energy - absorbed).max(0.0);
            if let Some(s) = seed { result.spawned_seeds.push(s); }
            if plant.is_dead() { dead_indices.push(i); }
        }
        dead_indices.sort_unstable_by(|a, b| b.cmp(a));
        for idx in dead_indices { self.plants.swap_remove(idx); }
    }

    fn update_animals(&mut self, ctx: &TickContext, result: &mut ChunkTickResult) {
        let animal_snaps: Vec<_> = self.animals.iter().map(|a| (a.id, a.position, a.animal_type, a.genome.size, a.genome.diet, a.genome.aggression, a.energy, a.genome.species_id, a.genome.aquatic_adaptation)).collect();
        let plant_snaps: Vec<_> = self.plants.iter().enumerate().map(|(i, p)| (i, Vec2::new(p.position.0, p.position.1), p.energy, p.is_poisonous)).collect();
        let mut updates = Vec::with_capacity(self.animals.len());
        for mut animal in std::mem::take(&mut self.animals) {
            let tile = &self.tiles[world_to_tile_index(animal.position, self.id)];
            let res = animal.update(tile.tile_type == TileType::Water, &plant_snaps, &animal_snaps, ctx.climate.temperature, ctx.climate.humidity, ctx.climate.wind_speed);
            updates.push((animal, res));
        }

        let mut eaten_ids = std::collections::HashSet::new();
        self.apply_animal_actions(&mut updates, &mut eaten_ids, ctx, result);
        self.finalize_animals(updates, eaten_ids, result);
    }

    fn apply_animal_actions(&mut self, updates: &mut [(Animal, AnimalUpdateResult)], eaten_ids: &mut std::collections::HashSet<u64>, ctx: &TickContext, result: &mut ChunkTickResult) {
        let mut dead_plants = std::collections::HashSet::new();
        for i in 0..updates.len() {
            if updates[i].1.died { continue; }
            if let Some(p_idx) = updates[i].1.want_to_eat_plant_idx {
                if p_idx < self.plants.len() && !dead_plants.contains(&p_idx) {
                    let plant = &mut self.plants[p_idx];
                    let eat_amount = (plant.energy * 0.5).min(plant.nutritional_value() + updates[i].0.genome.size * 1.5);
                    plant.energy -= eat_amount;
                    let digestion = (1.0 - updates[i].0.genome.diet).clamp(0.2, 1.0) * updates[i].0.genome.digestion_efficiency;
                    if plant.is_poisonous { updates[i].0.health -= 10.0 * (1.0 - updates[i].0.genome.digestion_efficiency); }
                    else { updates[i].0.energy = (updates[i].0.energy + eat_amount * digestion).min(updates[i].0.genome.reproduction_threshold * 3.0); }
                    if plant.energy <= 0.0 { dead_plants.insert(p_idx); }
                }
            }
        }
        let mut sorted_dead_plants: Vec<_> = dead_plants.into_iter().collect();
        sorted_dead_plants.sort_unstable_by(|a, b| b.cmp(a));
        for idx in sorted_dead_plants { self.plants.swap_remove(idx); }

        let id_to_idx: std::collections::HashMap<u64, usize> = updates.iter().enumerate().map(|(i, (a, _))| (a.id, i)).collect();
        let actions: Vec<_> = updates.iter().enumerate().map(|(i, (a, res))| (i, a.id, res.want_to_attack, res.want_to_breed_with)).collect();
        for (i, id, want_attack, want_breed) in actions {
            if updates[i].1.died { continue; }
            if let Some(target_id) = want_attack {
                if let Some(&target_idx) = id_to_idx.get(&target_id) {
                    if !eaten_ids.contains(&target_id) && updates[target_idx].0.health > 0.0 {
                        let damage = (updates[i].0.genome.size * 15.0 - updates[target_idx].0.genome.size * 5.0).max(5.0);
                        updates[target_idx].0.health -= damage;
                        if updates[target_idx].0.health <= 0.0 {
                            eaten_ids.insert(target_id);
                            updates[i].0.energy = (updates[i].0.energy + (updates[target_idx].0.genome.size * 30.0) * updates[i].0.genome.digestion_efficiency).min(updates[i].0.genome.reproduction_threshold * 3.0);
                            result.events.push(format!("Животное #{} съело #{}!", id, target_id));
                        }
                    }
                }
            }
            if let Some(m_id) = want_breed {
                if let Some(&m_idx) = id_to_idx.get(&m_id) {
                    if !eaten_ids.contains(&m_id) && updates[m_idx].0.health > 0.0 && updates[i].0.energy > updates[i].0.genome.reproduction_threshold * 0.5 && updates[m_idx].0.energy > updates[m_idx].0.genome.reproduction_threshold * 0.5 {
                        updates[i].0.energy -= updates[i].0.genome.reproduction_threshold * 0.3;
                        updates[m_idx].0.energy -= updates[m_idx].0.genome.reproduction_threshold * 0.3;
                        updates[i].0.last_reproduction = 0; updates[m_idx].0.last_reproduction = 0;
                        for _ in 0..(updates[i].0.genome.offspring_count.round().max(1.0) as u32) {
                            let child = Animal::new(ctx.next_entity_id.fetch_add(1, Ordering::Relaxed), updates[i].0.animal_type, Genome::crossover(&updates[i].0.genome, &updates[m_idx].0.genome, ctx.mutation_rate), updates[i].0.position + Vec2::new(rand::thread_rng().gen_range(-15.0..15.0), rand::thread_rng().gen_range(-15.0..15.0)));
                            result.spawned_animals.push(child);
                        }
                    }
                }
            }
        }
    }

    fn finalize_animals(&mut self, updates: Vec<(Animal, AnimalUpdateResult)>, eaten_ids: std::collections::HashSet<u64>, result: &mut ChunkTickResult) {
        let left = self.id.0 as f32 * CHUNK_WORLD_SIZE;
        let top = self.id.1 as f32 * CHUNK_WORLD_SIZE;
        for (animal, _) in updates {
            if eaten_ids.contains(&animal.id) || animal.health <= 0.0 { result.died_animal_ids.push(animal.id); continue; }
            if animal.position.x < left || animal.position.x >= left + CHUNK_WORLD_SIZE || animal.position.y < top || animal.position.y >= top + CHUNK_WORLD_SIZE { result.migrated_animals.push(animal); }
            else { self.animals.push(animal); }
        }
    }
}
