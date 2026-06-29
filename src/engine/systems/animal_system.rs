use std::collections::{HashSet, HashMap};
use std::sync::atomic::Ordering;
use glam::Vec2;
use rand::Rng;

use crate::biology::animal::{Animal, AnimalUpdateResult, AnimalSnapshot};
use crate::biology::plant::Plant;
use crate::biology::genome::Genome;
use crate::engine::tile::{Tile, TileType};
use crate::engine::chunk::{TickContext, ChunkTickResult, world_to_tile_index};
use crate::engine::config::*;

pub fn update_animals(
    animals: &mut Vec<Animal>,
    plants: &mut Vec<Plant>,
    tiles: &[Tile],
    chunk_id: (i32, i32),
    animal_spatial_grid: &[Vec<usize>],
    plant_spatial_grid: &[Vec<usize>],
    ctx: &TickContext,
    result: &mut ChunkTickResult
) {
    let animal_snaps: Vec<AnimalSnapshot> = animals.iter().map(|a| (a.id, a.position, a.animal_type, a.genome.size, a.genome.diet, a.genome.aggression, a.energy, a.genome.species_id, a.genome.aquatic_adaptation)).collect();
    let plant_snaps: Vec<_> = plants.iter().enumerate().map(|(i, p)| (i, p.position, p.energy, p.is_poisonous)).collect();

    let chunk_left = chunk_id.0 as f32 * CHUNK_WORLD_SIZE;
    let chunk_top = chunk_id.1 as f32 * CHUNK_WORLD_SIZE;

    use rayon::prelude::*;

    let updates: Vec<_> = std::mem::take(animals).into_par_iter().enumerate().map(|(i, mut animal)| {
        let gx = ((animal.position.x - chunk_left) / GRID_CELL_SIZE).floor() as i32;
        let gy = ((animal.position.y - chunk_top) / GRID_CELL_SIZE).floor() as i32;

        let mut nearby_animal_indices = Vec::new();
        let mut nearby_plant_indices = Vec::new();

        for dy in -1..=1 {
            for dx in -1..=1 {
                let nx = gx + dx;
                let ny = gy + dy;
                if nx >= 0 && nx < GRID_WIDTH as i32 && ny >= 0 && ny < GRID_WIDTH as i32 {
                    let cell_idx = (ny * GRID_WIDTH as i32 + nx) as usize;
                    nearby_animal_indices.extend(&animal_spatial_grid[cell_idx]);
                    nearby_plant_indices.extend(&plant_spatial_grid[cell_idx]);
                }
            }
        }

        let filtered_animals: Vec<_> = nearby_animal_indices.into_iter().filter(|&&idx| idx != i).map(|&idx| animal_snaps[idx]).collect();
        let filtered_plants: Vec<_> = nearby_plant_indices.into_iter().map(|&idx| plant_snaps[idx]).collect();

        let tile_idx = world_to_tile_index(animal.position, chunk_id);
        let is_water = tiles[tile_idx].tile_type == TileType::Water;

        let res = animal.update(is_water, &filtered_plants, &filtered_animals, ctx.climate.temperature, ctx.climate.humidity, ctx.climate.wind_speed);
        (animal, res)
    }).collect();

    let eaten_ids = HashSet::new();
    apply_animal_actions(updates, plants, eaten_ids, chunk_id, ctx, result, animals);
}

fn apply_animal_actions(
    mut updates: Vec<(Animal, AnimalUpdateResult)>,
    plants: &mut Vec<Plant>,
    mut eaten_ids: HashSet<u64>,
    chunk_id: (i32, i32),
    ctx: &TickContext,
    result: &mut ChunkTickResult,
    animals_vec: &mut Vec<Animal>
) {
    let mut dead_plants = HashSet::new();
    for (animal, res) in &mut updates {
        if res.died || eaten_ids.contains(&animal.id) { continue; }
        if let Some(p_idx) = res.want_to_eat_plant_idx {
            if p_idx < plants.len() && !dead_plants.contains(&p_idx) {
                let plant = &mut plants[p_idx];
                let eat_amount = (plant.energy * 0.5).min(plant.nutritional_value() + animal.genome.size * 1.5);
                plant.energy -= eat_amount;
                let digestion = (1.0 - animal.genome.diet).clamp(0.2, 1.0) * animal.genome.digestion_efficiency;
                if plant.is_poisonous { animal.health -= 10.0 * (1.0 - animal.genome.digestion_efficiency); }
                else { animal.energy = (animal.energy + eat_amount * digestion).min(animal.genome.reproduction_threshold * 3.0); }
                if plant.energy <= 0.0 { dead_plants.insert(p_idx); }
            }
        }
    }
    let mut sorted_dead_plants: Vec<_> = dead_plants.into_iter().collect();
    sorted_dead_plants.sort_unstable_by(|a, b| b.cmp(a));
    for idx in sorted_dead_plants { plants.swap_remove(idx); }

    let id_to_idx: HashMap<u64, usize> = updates.iter().enumerate().map(|(i, (a, _))| (a.id, i)).collect();
    let actions: Vec<_> = updates.iter().enumerate().map(|(i, (a, res))| (i, a.id, res.want_to_attack, res.want_to_breed_with)).collect();
    let mut bred_this_tick = HashSet::new();
    for (i, id, want_attack, want_breed) in actions {
        if updates[i].1.died || eaten_ids.contains(&id) { continue; }
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
                if !bred_this_tick.contains(&id) && !bred_this_tick.contains(&m_id) && !eaten_ids.contains(&m_id) && updates[m_idx].0.health > 0.0 && updates[i].0.energy > updates[i].0.genome.reproduction_threshold * 0.5 && updates[m_idx].0.energy > updates[m_idx].0.genome.reproduction_threshold * 0.5 {
                    bred_this_tick.insert(id);
                    bred_this_tick.insert(m_id);
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

    finalize_animals(updates, eaten_ids, chunk_id, result, animals_vec);
}

fn finalize_animals(
    updates: Vec<(Animal, AnimalUpdateResult)>,
    eaten_ids: HashSet<u64>,
    chunk_id: (i32, i32),
    result: &mut ChunkTickResult,
    animals_vec: &mut Vec<Animal>
) {
    let left = chunk_id.0 as f32 * CHUNK_WORLD_SIZE;
    let top = chunk_id.1 as f32 * CHUNK_WORLD_SIZE;
    for (animal, _) in updates {
        if eaten_ids.contains(&animal.id) || animal.health <= 0.0 {
            result.died_animal_ids.push(animal.id);
            continue;
        }
        if animal.position.x < left || animal.position.x >= left + CHUNK_WORLD_SIZE ||
           animal.position.y < top || animal.position.y >= top + CHUNK_WORLD_SIZE {
            result.migrated_animals.push(animal);
        } else {
            animals_vec.push(animal);
        }
    }
}
