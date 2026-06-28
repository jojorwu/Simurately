#![allow(dead_code)]
use std::sync::atomic::{AtomicU64, Ordering};
use std::collections::HashMap;
use rayon::prelude::*;
use glam::Vec2;
use rand::Rng;

use crate::biology::genome::Genome;
use crate::biology::plant::{Plant, PlantType};
use crate::biology::animal::{Animal, AnimalType};
use crate::engine::tile::TileType;
use crate::engine::climate::{Climate, WeatherType};
use crate::engine::events::WorldEvent;
use crate::stats::StatsManager;
use crate::engine::chunk::{Chunk, ChunkTickResult, CHUNK_WORLD_SIZE, world_to_tile_index};
use crate::engine::evolution::EvolutionManager;

pub fn world_to_chunk_coords(pos: Vec2) -> (i32, i32) {
    (
        (pos.x / CHUNK_WORLD_SIZE).floor() as i32,
        (pos.y / CHUNK_WORLD_SIZE).floor() as i32,
    )
}

// RenderData is removed as rendering culling is handled via visible chunks.

// =====================================================================
//  МИР
// =====================================================================
pub struct World {
    pub chunks: HashMap<(i32, i32), Chunk>,
    pub next_entity_id: AtomicU64,
    pub mutation_rate: f32,
    pub tick_count: u64,
    pub logs: Vec<String>,
    pub climate: Climate,
    pub evolution_manager: EvolutionManager,
    
    // Статистика
    pub stats: StatsManager,
    pub migration_buffer: Vec<u64>,
    pub spawn_seeds_buffer: Vec<(f32, f32, PlantType, Genome)>,
    pub spawn_animals_buffer: Vec<u64>,
}

// =====================================================================
//  РЕАЛИЗАЦИЯ МИРА
// =====================================================================
impl World {
    pub fn new() -> Self {
        let mut world = Self {
            chunks: HashMap::new(),
            next_entity_id: AtomicU64::new(1),
            mutation_rate: 0.05,
            tick_count: 0,
            logs: vec!["Симуляция запущена".to_string()],
            climate: Climate::new(),
            evolution_manager: EvolutionManager::new(),
            stats: StatsManager::new(),
            migration_buffer: Vec::new(),
            spawn_seeds_buffer: Vec::new(),
            spawn_animals_buffer: Vec::new(),
        };

        
        let mut ins_genome = Genome::default_insect();
        let ins_id = world.evolution_manager.register_or_match_species(&mut ins_genome, AnimalType::Insect);
        ins_genome.species_id = ins_id;
        let _ = ins_genome;
        
        let mut fish_genome = Genome::default_fish();
        let fish_id = world.evolution_manager.register_or_match_species(&mut fish_genome, AnimalType::Fish);
        fish_genome.species_id = fish_id;
        let _ = fish_genome;

        
        world
    }


    pub fn get_visible_chunks(&self, min: Vec2, max: Vec2) -> Vec<&Chunk> {
        let min_cx = (min.x / CHUNK_WORLD_SIZE).floor() as i32;
        let max_cx = (max.x / CHUNK_WORLD_SIZE).floor() as i32;
        let min_cy = (min.y / CHUNK_WORLD_SIZE).floor() as i32;
        let max_cy = (max.y / CHUNK_WORLD_SIZE).floor() as i32;

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
        let after_count = self.evolution_manager.species_registry.len();
        if after_count > before_count {
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
            chunk.plants.push(Plant::new(id, plant_type, gen_data, (pos.x, pos.y)));
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
        let animal = Animal::new(id, animal_type, animal_genome, pos);
        if let Some(chunk) = self.chunks.get_mut(&(cx, cy)) {
            chunk.animals.push(animal);
        }
    }

    /// Главный такт симуляции
    pub fn tick(&mut self) {
        self.tick_count += 1;

        // 1. Обновление климата и обработка событий
        let climate_events = self.climate.tick(self.tick_count);
        for event in climate_events {
            match event {
                WorldEvent::WeatherChanged(msg) => self.logs.push(msg),
                WorldEvent::SeasonChanged(msg) => self.logs.push(msg),
                WorldEvent::LightningStrike(pos) => {
                    let lx = pos.x * CHUNK_WORLD_SIZE + rand::thread_rng().gen_range(0.0..CHUNK_WORLD_SIZE);
                    let ly = pos.y * CHUNK_WORLD_SIZE + rand::thread_rng().gen_range(0.0..CHUNK_WORLD_SIZE);
                    let lpos = Vec2::new(lx, ly);
                    self.log(format!("⚡ Удар молнии в ({:.0}, {:.0})!", lx, ly));
                    self.climate.lightning_strike = Some((lpos, 6));
                    
                    // Молния наносит урон
                    for chunk in self.chunks.values_mut() {
                        for plant in &mut chunk.plants {
                            if Vec2::new(plant.position.0, plant.position.1).distance(lpos) < 50.0 {
                                plant.health -= 40.0;
                            }
                        }
                        for animal in &mut chunk.animals {
                            if animal.position.distance(lpos) < 50.0 {
                                animal.health -= 50.0;
                            }
                        }
                    }
                }
            }
        }
        if self.logs.len() > 150 { self.logs.drain(0..50); }


        // 3. Параллельный обсчёт чанков (parallel — safe)
                let climate_snapshot = self.climate.clone();
                let mutation_rate = self.mutation_rate;
                let id_gen = &self.next_entity_id;
                let bucket_index = (self.tick_count % 4) as usize;

                let all_results: Vec<((i32, i32), ChunkTickResult)> = self.chunks
                    .iter_mut()
                    .par_bridge()
                    .map(|(coords, chunk)| {
                        let res = chunk.tick(mutation_rate, id_gen, &climate_snapshot, self.tick_count, bucket_index);
                        (*coords, res)
                    })
                    .collect();


        // 4. Сбор результатов
        let mut seeds_buf: Vec<(f32, f32, PlantType, Genome)> = Vec::new();
        let mut animals_buf: Vec<Animal> = Vec::new();
        let mut died_count = 0u64;
        
        // Группировка результатов по чанкам для пакетной обработки
        let mut animal_batches: std::collections::HashMap<(i32, i32), Vec<Animal>> = std::collections::HashMap::new();
        let mut seed_batches: std::collections::HashMap<(i32, i32), Vec<(f32, f32, PlantType, Genome)>> = std::collections::HashMap::new();

        for (_, res) in all_results {
            seeds_buf.extend(res.spawned_seeds);
            animals_buf.extend(res.migrated_animals);
            animals_buf.extend(res.spawned_animals);
            died_count += res.died_animal_ids.len() as u64;
            for ev in res.events { self.log(ev); }
        }
                self.stats.total_deaths += died_count;
        
        // Группируем животных по чанкам
        let mut births_this_tick = 0u64;
        for mut animal in animals_buf {
            if animal.age == 0 {
                births_this_tick += 1;
                    let spec_id = self.evolution_manager.register_or_match_species(&mut animal.genome, animal.animal_type);
                animal.genome.species_id = spec_id;
                if let Some(spec) = self.evolution_manager.species_registry.get_mut(&spec_id) {
                    spec.total_born += 1;
                }
            }
            let coords = world_to_chunk_coords(animal.position);
            animal_batches.entry(coords).or_default().push(animal);
        }
                self.stats.total_births += births_this_tick;

        // Группируем семена по чанкам
        for (sx, sy, ptype, genome) in seeds_buf {
            let coords = world_to_chunk_coords(Vec2::new(sx, sy));
            seed_batches.entry(coords).or_default().push((sx, sy, ptype, genome));
        }

        // Пакетное обновление чанков для животных
        for (coords, animals) in animal_batches {
            self.add_chunk(coords.0, coords.1);
            if let Some(chunk) = self.chunks.get_mut(&coords) {
                chunk.animals.extend(animals);
            }
        }

        // Пакетное обновление чанков для семян
        for (coords, seeds) in seed_batches {
            self.add_chunk(coords.0, coords.1);
            if let Some(chunk) = self.chunks.get_mut(&coords) {
                for (sx, sy, ptype, genome) in seeds {
                    let tile_idx = world_to_tile_index(Vec2::new(sx, sy), coords);
                    let tile_type = chunk.tiles[tile_idx].tile_type;
                    let can_grow = match ptype {
                        PlantType::Mushroom => tile_type == TileType::Soil && chunk.tiles[tile_idx].moisture > 0.5,
                        _ => tile_type == TileType::Soil || tile_type == TileType::Sand,
                    };
                    if can_grow {
                        let id = self.next_entity_id.fetch_add(1, Ordering::Relaxed);
                        chunk.plants.push(Plant::new(id, ptype, genome, (sx, sy)));
                    }
                }
            }
        }


        // 7. Статистика популяций

        let mut plants = 0usize;
        let mut insects = 0usize;
        let mut fish = 0usize;

        for spec in self.evolution_manager.species_registry.values_mut() { spec.population = 0; }

        for chunk in self.chunks.values() {
            plants += chunk.plants.len();
            for a in &chunk.animals {
                match a.animal_type {
                    AnimalType::Insect => insects += 1,
                    AnimalType::Fish => fish += 1,
                }
                if let Some(spec) = self.evolution_manager.species_registry.get_mut(&a.genome.species_id) {
                    spec.population += 1;
                }
            }
        }

        // Вымирание видов
        for spec in self.evolution_manager.species_registry.values_mut() {
            if spec.population == 0 && spec.active {
                spec.active = false;
                self.logs.push(format!(
                    "[Тик {}] ВЫМИРАНИЕ: Вид '{}' исчез! (прожил {} тиков)",
                    self.tick_count, spec.name, self.tick_count - spec.founded_at_tick
                ));
            } else if spec.population > 0 && !spec.active {
                spec.active = true;
            }
        }

        // Обновление истории
                let active_species = self.evolution_manager.species_registry.values().filter(|s| s.active).count();
                self.stats.record_history(plants, insects, fish, active_species);

    }

    pub fn log(&mut self, msg: String) {
        self.logs.push(msg);
        if self.logs.len() > 150 {
            self.logs.drain(0..20);
        }
    }

    /// Получить текущее количество существ каждого типа
    pub fn population_counts(&self) -> (usize, usize, usize) {
        let mut plants = 0;
        let mut insects = 0;
        let mut fish = 0;
        for chunk in self.chunks.values() {
            plants += chunk.plants.len();
            for a in &chunk.animals {
                match a.animal_type {
                    AnimalType::Insect => insects += 1,
                    AnimalType::Fish => fish += 1,
                }
            }
        }
        (plants, insects, fish)
    }
}
