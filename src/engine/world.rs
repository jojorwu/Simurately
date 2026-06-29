use std::sync::atomic::{AtomicU64, Ordering};
use std::collections::HashMap;
use rayon::prelude::*;
use glam::Vec2;
use rand::Rng;

use crate::biology::genome::Genome;
use crate::biology::plant::PlantType;
use crate::biology::animal::AnimalType;
use crate::engine::tile::TileType;
use crate::engine::climate::Climate;
use crate::engine::events::WorldEvent;
use crate::engine::stats::StatsManager;
use crate::engine::chunk::{Chunk, ChunkTickResult, world_to_tile_index};
use crate::engine::evolution::EvolutionManager;
use crate::engine::config::*;

pub fn world_to_chunk_coords(pos: Vec2) -> (i32, i32) {
    (
        (pos.x / CHUNK_WORLD_SIZE).floor() as i32,
        (pos.y / CHUNK_WORLD_SIZE).floor() as i32,
    )
}

pub struct World {
    pub chunks: HashMap<(i32, i32), Chunk>,
    pub next_entity_id: AtomicU64,
    pub mutation_rate: f32,
    pub tick_count: u64,
    pub logs: crate::engine::stats::EventLog,
    pub climate: Climate,
    pub evolution_manager: EvolutionManager,
    pub stats: StatsManager,
}

impl Default for World {
    fn default() -> Self {
        Self::new()
    }
}

impl World {
    pub fn new() -> Self {
        let mut world = Self {
            chunks: HashMap::new(),
            next_entity_id: AtomicU64::new(1),
            mutation_rate: 0.05,
            tick_count: 0,
            logs: crate::engine::stats::EventLog::new(),
            climate: Climate::new(),
            evolution_manager: EvolutionManager::new(),
            stats: StatsManager::new(),
        };

        let mut ins_genome = Genome::default_insect();
        world.evolution_manager.register_or_match_species(&mut ins_genome, AnimalType::Insect);

        let mut fish_genome = Genome::default_fish();
        world.evolution_manager.register_or_match_species(&mut fish_genome, AnimalType::Fish);

        world
    }

    pub fn get_visible_chunks(&self, min: Vec2, max: Vec2) -> Vec<&Chunk> {
        let (min_cx, min_cy) = world_to_chunk_coords(min);
        let (max_cx, max_cy) = world_to_chunk_coords(max);

        let mut visible = Vec::new();
        for cy in min_cy..=max_cy {
            for cx in min_cx..=max_cx {
                if let Some(chunk) = self.chunks.get(&(cx, cy)) {
                    visible.push(chunk);
                }
            }
        }
        visible
    }

    pub fn register_or_match_species(&mut self, genome: &mut Genome, animal_type: AnimalType) -> u32 {
        let before_count = self.evolution_manager.species_registry.len();
        let spec_id = self.evolution_manager.register_or_match_species(genome, animal_type);
        if self.evolution_manager.species_registry.len() > before_count {
            self.stats.total_speciations += 1;
            if let Some(spec) = self.evolution_manager.species_registry.get_mut(&spec_id) {
                spec.founded_at_tick = self.tick_count;
            }
        }
        spec_id
    }

    pub fn add_chunk(&mut self, cx: i32, cy: i32) {
        self.chunks.entry((cx, cy)).or_insert_with(|| Chunk::new((cx, cy)));
    }

    pub fn spawn_plant(&mut self, plant_type: PlantType, pos: Vec2, genome: Option<Genome>) {
        let (cx, cy) = world_to_chunk_coords(pos);
        self.add_chunk(cx, cy);
        if let Some(chunk) = self.chunks.get_mut(&(cx, cy)) {
            let id = self.next_entity_id.fetch_add(1, Ordering::Relaxed);
            let gen_data = genome.unwrap_or_else(Genome::random);
        chunk.plants.push(crate::biology::plant::Plant::new(id, plant_type, gen_data, pos));
        }
    }

    pub fn spawn_animal(&mut self, animal_type: AnimalType, pos: Vec2, genome: Option<Genome>) {
        let (cx, cy) = world_to_chunk_coords(pos);
        self.add_chunk(cx, cy);
        let id = self.next_entity_id.fetch_add(1, Ordering::Relaxed);
        let mut animal_genome = genome.unwrap_or_else(|| match animal_type {
            AnimalType::Insect => Genome::default_insect(),
            AnimalType::Fish => Genome::default_fish(),
        });
        let spec_id = self.register_or_match_species(&mut animal_genome, animal_type);
        animal_genome.species_id = spec_id;
        let animal = crate::biology::animal::Animal::new(id, animal_type, animal_genome, pos);
        if let Some(chunk) = self.chunks.get_mut(&(cx, cy)) {
            chunk.animals.push(animal);
        }
    }

    pub fn tick(&mut self) {
        self.tick_count += 1;
        self.handle_climate_and_events();

        let all_results = self.tick_chunks_parallel();
        self.process_tick_results(all_results);
        self.update_statistics();
    }

    fn handle_climate_and_events(&mut self) {
        let climate_events = self.climate.tick(self.tick_count);
        for event in climate_events {
            match event {
                WorldEvent::WeatherChanged(msg) | WorldEvent::SeasonChanged(msg) => self.log(msg),
                WorldEvent::LightningStrike(pos) => self.handle_lightning_strike(pos),
            }
        }
    }

    fn handle_lightning_strike(&mut self, _pos: Vec2) {
        let mut rng = rand::thread_rng();
        if self.chunks.is_empty() { return; }
        let keys: Vec<_> = self.chunks.keys().cloned().collect();
        let coords = keys[rng.gen_range(0..keys.len())];

        let lx = coords.0 as f32 * CHUNK_WORLD_SIZE + rng.gen_range(0.0..CHUNK_WORLD_SIZE);
        let ly = coords.1 as f32 * CHUNK_WORLD_SIZE + rng.gen_range(0.0..CHUNK_WORLD_SIZE);
        let lpos = Vec2::new(lx, ly);

        self.log(format!("⚡ Удар молнии в ({:.0}, {:.0})!", lx, ly));
        self.climate.lightning_strike = Some((lpos, 6));

        let (cx, cy) = world_to_chunk_coords(lpos);
        for dx in -1..=1 {
            for dy in -1..=1 {
                if let Some(chunk) = self.chunks.get_mut(&(cx + dx, cy + dy)) {
                    for plant in &mut chunk.plants {
                        if plant.position.distance(lpos) < 50.0 { plant.health -= 40.0; }
                    }
                    for animal in &mut chunk.animals {
                        if animal.position.distance(lpos) < 50.0 { animal.health -= 50.0; }
                    }
                }
            }
        }
    }

    fn tick_chunks_parallel(&mut self) -> Vec<((i32, i32), ChunkTickResult)> {
        let climate_snapshot = self.climate.clone();
        let mutation_rate = self.mutation_rate;
        let id_gen = &self.next_entity_id;
        let bucket_index = (self.tick_count % 4) as usize;

        self.chunks.par_iter_mut().map(|(coords, chunk)| {
            let res = chunk.tick(mutation_rate, id_gen, &climate_snapshot, self.tick_count, bucket_index);
            (*coords, res)
        }).collect()
    }

    fn process_tick_results(&mut self, results: Vec<((i32, i32), ChunkTickResult)>) {
        let (animals_to_spawn, seeds_to_spawn, died_count, events) = results.into_par_iter()
            .fold(
                || (Vec::new(), Vec::new(), 0u64, Vec::new()),
                |(mut a, mut s, mut d, mut e), (_, res)| {
                    a.extend(res.migrated_animals);
                    a.extend(res.spawned_animals);
                    s.extend(res.spawned_seeds);
                    d += res.died_animal_ids.len() as u64;
                    e.extend(res.events);
                    (a, s, d, e)
                }
            )
            .reduce(
                || (Vec::new(), Vec::new(), 0u64, Vec::new()),
                |(mut a1, mut s1, mut d1, mut e1), (a2, s2, d2, e2)| {
                    a1.extend(a2);
                    s1.extend(s2);
                    d1 += d2;
                    e1.extend(e2);
                    (a1, s1, d1, e1)
                }
            );

        for ev in events { self.log(ev); }
        self.stats.total_deaths += died_count;

        for mut animal in animals_to_spawn {
            let coords = world_to_chunk_coords(animal.position);
            if animal.age == 0 {
                self.stats.total_births += 1;
                let spec_id = self.register_or_match_species(&mut animal.genome, animal.animal_type);
                animal.genome.species_id = spec_id;
                if let Some(spec) = self.evolution_manager.species_registry.get_mut(&spec_id) { spec.total_born += 1; }
            }
            self.chunks.entry(coords).or_insert_with(|| Chunk::new(coords)).animals.push(animal);
        }

        for (pos, ptype, genome) in seeds_to_spawn {
            let coords = world_to_chunk_coords(pos);
            let chunk = self.chunks.entry(coords).or_insert_with(|| Chunk::new(coords));
            {
                let tile_idx = world_to_tile_index(pos, coords);
                let tile = &chunk.tiles[tile_idx];
                let can_grow = match ptype {
                    PlantType::Mushroom => tile.tile_type == TileType::Soil && tile.moisture > 0.5,
                    _ => tile.tile_type == TileType::Soil || tile.tile_type == TileType::Sand,
                };
                if can_grow {
                    let id = self.next_entity_id.fetch_add(1, Ordering::Relaxed);
                    chunk.plants.push(crate::biology::plant::Plant::new(id, ptype, genome, pos));
                }
            }
        }
    }

    fn update_statistics(&mut self) {
        let mut plants = 0; let mut insects = 0; let mut fish = 0;
        for spec in self.evolution_manager.species_registry.values_mut() { spec.population = 0; }

        for chunk in self.chunks.values() {
            plants += chunk.plants.len();
            for a in &chunk.animals {
                match a.animal_type { AnimalType::Insect => insects += 1, AnimalType::Fish => fish += 1 }
                if let Some(spec) = self.evolution_manager.species_registry.get_mut(&a.genome.species_id) { spec.population += 1; }
            }
        }

        let logs = self.evolution_manager.update_populations(self.tick_count);
        for log in logs { self.log(log); }

        let active_species = self.evolution_manager.species_registry.values().filter(|s| s.active).count();
        self.stats.record_history(plants, insects, fish, active_species);
    }

    pub fn log(&mut self, msg: String) {
        self.logs.push(msg);
    }

    pub fn population_counts(&self) -> (usize, usize, usize) {
        let mut p = 0; let mut i = 0; let mut f = 0;
        for chunk in self.chunks.values() {
            p += chunk.plants.len();
            for a in &chunk.animals { match a.animal_type { AnimalType::Insect => i += 1, AnimalType::Fish => f += 1 } }
        }
        (p, i, f)
    }
}
