use std::sync::atomic::{AtomicU64, Ordering};
use glam::Vec2;
use rand::Rng;

use crate::biology::genome::Genome;
use crate::biology::plant::{Plant, PlantType};
use crate::biology::animal::{Animal, AnimalType, AnimalUpdateResult};
use crate::engine::tile::{Tile, TileType};
use crate::engine::climate::Climate;

pub const TILE_SIZE: f32 = 10.0;
pub const CHUNK_SIZE: usize = 64;
pub const PLANT_UPDATE_INTERVAL: u64 = 5;
pub const CHUNK_WORLD_SIZE: f32 = CHUNK_SIZE as f32 * TILE_SIZE;
pub const TILES_PER_TICK: usize = 100;
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
    pub plant_grid: Vec<Vec<usize>>,
    pub animal_grid: Vec<Vec<usize>>,
}

pub struct ChunkTickResult {
    pub migrated_animals: Vec<Animal>,
    pub spawned_seeds: Vec<(f32, f32, PlantType, Genome)>,
    pub spawned_animals: Vec<Animal>,
    pub events: Vec<String>,
    pub died_animal_ids: Vec<u64>,
}

impl Chunk {
    pub fn new(id: (i32, i32)) -> Self {
        let mut tiles = Vec::with_capacity(CHUNK_SIZE * CHUNK_SIZE);
        for ty in 0..CHUNK_SIZE {
            for tx in 0..CHUNK_SIZE {
                let gx = id.0 as f32 * CHUNK_WORLD_SIZE + tx as f32 * TILE_SIZE;
                let gy = id.1 as f32 * CHUNK_WORLD_SIZE + ty as f32 * TILE_SIZE;

                // Шум на основе нескольких частот (похоже на Perlin)
                let noise = (gx * 0.0035).sin() * (gy * 0.0035).cos()
                    + 0.4 * (gx * 0.012 + gy * 0.009).sin()
                    + 0.15 * (gx * 0.05 - gy * 0.04).cos()
                    + 0.08 * (gx * 0.1).sin();

                let tile_type = if noise < -0.20 {
                    TileType::Water
                } else if noise < -0.10 {
                    TileType::Sand
                } else if noise > 0.55 {
                    TileType::Rock
                } else {
                    TileType::Soil
                };

                let (initial_energy, init_moisture, init_temp) = match tile_type {
                    TileType::Soil => (100.0, 0.5, 0.3),
                    TileType::Sand => (25.0, 0.15, 0.5),
                    TileType::Water => (0.0, 1.0, 0.1),
                    TileType::Rock => (0.0, 0.05, 0.4),
                };

                tiles.push(Tile {
                    tile_type,
                    energy: initial_energy,
                    moisture: init_moisture,
                    temperature: init_temp,
                });
            }
        }
        Self { id, tiles, plants: Vec::new(), animals: Vec::new(), active: true, last_updated_tile_idx: 0, plant_grid: vec![Vec::new(); CHUNK_SIZE * CHUNK_SIZE], animal_grid: vec![Vec::new(); CHUNK_SIZE * CHUNK_SIZE] }

    }

    /// Обновление чанка за один тик
    pub fn tick(
        &mut self,
        mutation_rate: f32,
        next_entity_id: &AtomicU64,
        climate: &Climate,
        tick_count: u64,
        _bucket_index: usize,
    ) -> ChunkTickResult {

        let mut migrated_animals: Vec<Animal> = Vec::new();
        let mut spawned_seeds: Vec<(f32, f32, PlantType, Genome)> = Vec::new();
        let mut spawned_animals: Vec<Animal> = Vec::new();
        let mut events: Vec<String> = Vec::new();
        let mut died_animal_ids: Vec<u64> = Vec::new();

        if !self.active {
            return ChunkTickResult { migrated_animals, spawned_seeds, spawned_animals, events, died_animal_ids };
        }

        let chunk_left = self.id.0 as f32 * CHUNK_WORLD_SIZE;
        let chunk_right = chunk_left + CHUNK_WORLD_SIZE;
        let chunk_top = self.id.1 as f32 * CHUNK_WORLD_SIZE;
        let chunk_bottom = chunk_top + CHUNK_WORLD_SIZE;

        let temp = climate.temperature;
        let humid = climate.humidity;
        let sun = climate.sunlight;
        let wind = climate.wind_speed;

        // ---- 1. ОБНОВЛЕНИЕ ТАЙЛОВ ----
        let mut i = self.last_updated_tile_idx;
        let mut updated_count = 0;
        while updated_count < TILES_PER_TICK {
            let tile = &mut self.tiles[i % (CHUNK_SIZE * CHUNK_SIZE)];
            // Восстановление влажности
            let moisture_regen = match tile.tile_type {
                TileType::Soil => humid * 0.15 - 0.02,
                TileType::Sand => humid * 0.05 - 0.02,
                TileType::Water => 0.01,
                TileType::Rock => 0.0,
            };
            tile.moisture = (tile.moisture + moisture_regen).clamp(0.0, 1.0);

            // Температура тайла сглаживается к климатической
            tile.temperature = tile.temperature * 0.95 + temp * 0.05;

            // Восстановление энергии почвы
            let regen = match tile.tile_type {
                TileType::Soil => (0.1 + sun * 0.1 + humid * 0.05).max(0.0),
                TileType::Sand => (0.02 + sun * 0.02).max(0.0),
                _ => 0.0,
            };
            tile.energy = (tile.energy + regen).min(if tile.tile_type == TileType::Soil { 200.0 } else { 50.0 });
            
            i += 1;
            updated_count += 1;
        }
        self.last_updated_tile_idx = i % (CHUNK_SIZE * CHUNK_SIZE);

        // ---- 2. ОБНОВЛЕНИЕ РАСТЕНИЙ ----
        if tick_count % PLANT_UPDATE_INTERVAL == 0 {
            let mut dead_plant_indices: Vec<usize> = Vec::new();
            let mut seeds_buffer: Vec<(f32, f32, PlantType, Genome)> = Vec::new();

            for (i, plant) in self.plants.iter_mut().enumerate() {
                let tile_idx = world_to_tile_index(Vec2::new(plant.position.0, plant.position.1), self.id);
                let tile = &self.tiles[tile_idx];
                let soil_e = tile.energy;
                let local_humidity = (tile.moisture + humid) / 2.0;
                let local_temp = (tile.temperature + temp) / 2.0;

                let (seed, absorbed_energy) = plant.update(soil_e, local_temp, local_humidity, sun);
                self.tiles[tile_idx].energy = (self.tiles[tile_idx].energy - absorbed_energy).max(0.0);

                if let Some(s) = seed {
                    seeds_buffer.push(s);
                }

                if plant.is_dead() {
                    dead_plant_indices.push(i);
                }
            }

            dead_plant_indices.sort_unstable_by(|a, b| b.cmp(a));
            for idx in dead_plant_indices {
                if idx < self.plants.len() {
                    self.plants.swap_remove(idx);
                }
            }
            spawned_seeds.extend(seeds_buffer);
        }

        // ---- 3. ОБНОВЛЕНИЕ ЖИВОТНЫХ ----
        let animal_snapshots: Vec<(u64, Vec2, AnimalType, f32, f32, f32, f32, u32, f32)> = self.animals.iter().map(|a| {
            (
                a.id,
                a.position,
                a.animal_type,
                a.genome.size,
                a.genome.diet,
                a.genome.aggression,
                a.energy,
                a.genome.species_id,
                a.genome.aquatic_adaptation,
            )
        }).collect();

        let plant_snapshots: Vec<(usize, Vec2, f32, bool)> = self.plants.iter().enumerate().map(|(idx, p)| {
            (idx, Vec2::new(p.position.0, p.position.1), p.energy, p.is_poisonous)
        }).collect();

        let mut updates: Vec<(Animal, AnimalUpdateResult)> = Vec::with_capacity(self.animals.len());

        for mut animal in std::mem::take(&mut self.animals) {
            let tile_idx = world_to_tile_index(animal.position, self.id);
            let is_water = self.tiles[tile_idx].tile_type == TileType::Water;

            let res = animal.update(
                is_water,
                &plant_snapshots,
                &animal_snapshots,
                temp,
                humid,
                wind,
            );
            updates.push((animal, res));
        }

        // Фаза 2: Применяем результаты
        let mut eaten_ids: std::collections::HashSet<u64> = std::collections::HashSet::new();

        // Сначала собираем все действия (без мутирования updates)
        struct ActionRecord {
            idx: usize,
            animal_id: u64,
            died: bool,
            eat_plant_idx: Option<usize>,
            attack_prey_id: Option<u64>,
            breed_with: Option<u64>,
        }
        let actions: Vec<ActionRecord> = updates.iter().enumerate().map(|(i, (a, res))| ActionRecord {
            idx: i,
            animal_id: a.id,
            died: res.died,
            eat_plant_idx: res.want_to_eat_plant_idx,
            attack_prey_id: res.want_to_attack,
            breed_with: res.want_to_breed_with,
        }).collect();

        // Применяем действия поедания растений
        for act in &actions {
            if act.died { continue; }
            if let Some(plant_idx) = act.eat_plant_idx {
                if plant_idx < self.plants.len() {
                    let (animal_genome_size, animal_genome_diet, animal_genome_dig, animal_genome_repro) = {
                        let a = &updates[act.idx].0;
                        (a.genome.size, a.genome.diet, a.genome.digestion_efficiency, a.genome.reproduction_threshold)
                    };
                    let plant = &mut self.plants[plant_idx];
                    let eat_amount = (plant.energy * 0.5).min(plant.nutritional_value() + animal_genome_size * 1.5);
                    plant.energy -= eat_amount;
                    let is_pois = plant.is_poisonous;
                    let digestion = (1.0 - animal_genome_diet).clamp(0.2, 1.0) * animal_genome_dig;
                    let gained = eat_amount * digestion;
                    let pois_dmg = if is_pois { 10.0 * (1.0 - animal_genome_dig) } else { 0.0 };

                    if is_pois {
                        updates[act.idx].0.health -= pois_dmg;
                        if pois_dmg > 5.0 {
                            events.push(format!("Животное #{} отравилось ядовитым растением!", act.animal_id));
                        }
                    } else {
                        updates[act.idx].0.energy = (updates[act.idx].0.energy + gained).min(animal_genome_repro * 3.0);
                    }

                    if self.plants[plant_idx].energy <= 0.0 {
                        self.plants.swap_remove(plant_idx);
                    }
                }
            }
        }

        // Применяем атаки (хищничество)
        for act in &actions {
            if act.died { continue; }
            if let Some(prey_id) = act.attack_prey_id {
                if let Some(prey_idx) = updates.iter().position(|(a, _)| a.id == prey_id) {
                    let prey_health = updates[prey_idx].0.health;
                    if prey_health > 0.0 && !eaten_ids.contains(&prey_id) {
                        let hunter_size = updates[act.idx].0.genome.size;
                        let prey_size = updates[prey_idx].0.genome.size;
                        let damage = (hunter_size * 15.0 - prey_size * 5.0).max(5.0);

                        updates[prey_idx].0.health -= damage;
                        if updates[prey_idx].0.health <= 0.0 {
                            eaten_ids.insert(prey_id);
                            let gain = (prey_size * 30.0) * updates[act.idx].0.genome.digestion_efficiency;
                            let max_repro = updates[act.idx].0.genome.reproduction_threshold;
                            updates[act.idx].0.energy = (updates[act.idx].0.energy + gain).min(max_repro * 3.0);
                            events.push(format!("Животное #{} съело #{}!", act.animal_id, prey_id));
                        }
                    }
                }
            }
        }

        // Применяем спаривание
        for act in &actions {
            if act.died { continue; }
            if let Some(mate_id) = act.breed_with {
                if let Some(mate_idx) = updates.iter().position(|(a, _)| a.id == mate_id) {
                    if !eaten_ids.contains(&mate_id) && updates[mate_idx].0.health > 0.0 {
                        let (parent, mate) = if act.idx < mate_idx {
                            let (left, right) = updates.split_at_mut(mate_idx);
                            (&mut left[act.idx].0, &mut right[0].0)
                        } else {
                            let (left, right) = updates.split_at_mut(act.idx);
                            (&mut right[0].0, &mut left[mate_idx].0)
                        };

                        if parent.energy > parent.genome.reproduction_threshold * 0.5
                            && mate.energy > mate.genome.reproduction_threshold * 0.5
                        {
                            parent.energy -= parent.genome.reproduction_threshold * 0.3;
                            mate.energy -= mate.genome.reproduction_threshold * 0.3;
                            parent.last_reproduction = 0;
                            mate.last_reproduction = 0;

                            // Создаём детёнышей (обработка на стороне мира)
                            let n_offspring = parent.genome.offspring_count.round().max(1.0) as u32;
                            for _ in 0..n_offspring {
                                let child_genome = Genome::crossover(&parent.genome, &mate.genome, mutation_rate);
                                let child_id = next_entity_id.fetch_add(1, Ordering::Relaxed);
                                let child_pos = parent.position + Vec2::new(
                                    rand::thread_rng().gen_range(-15.0..15.0),
                                    rand::thread_rng().gen_range(-15.0..15.0)
                                );
                                let child = Animal::new(child_id, parent.animal_type, child_genome, child_pos);
                                spawned_animals.push(child);
                            }
                        }
                    }
                }
            }
        }

        // Финальная сборка оставшихся животных
        for (animal, _) in updates {
            if eaten_ids.contains(&animal.id) || animal.health <= 0.0 {
                died_animal_ids.push(animal.id);
                continue;
            }
            // Мигрировавшие
            if animal.position.x < chunk_left || animal.position.x >= chunk_right
                || animal.position.y < chunk_top || animal.position.y >= chunk_bottom {
                migrated_animals.push(animal);
            } else {
                self.animals.push(animal);
            }
        }

        ChunkTickResult { migrated_animals, spawned_seeds, spawned_animals, events, died_animal_ids }
    }
}
