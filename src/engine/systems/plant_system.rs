use crate::biology::plant::Plant;
use crate::engine::tile::Tile;
use crate::engine::chunk::{TickContext, ChunkTickResult, world_to_tile_index};
use crate::engine::config::PLANT_UPDATE_INTERVAL;

pub fn update_plants(
    plants: &mut Vec<Plant>,
    tiles: &mut [Tile],
    chunk_id: (i32, i32),
    ctx: &TickContext,
    result: &mut ChunkTickResult
) {
    if ctx.tick_count % PLANT_UPDATE_INTERVAL != 0 { return; }

    let mut dead_indices = Vec::new();
    for (i, plant) in plants.iter_mut().enumerate() {
        let idx = world_to_tile_index(plant.position, chunk_id);
        let tile = &tiles[idx];

        let (seed, absorbed) = plant.update(
            tile.energy,
            (tile.temperature + ctx.climate.temperature) / 2.0,
            (tile.moisture + ctx.climate.humidity) / 2.0,
            ctx.climate.sunlight
        );

        tiles[idx].energy = (tiles[idx].energy - absorbed).max(0.0);

        if let Some(s) = seed {
            result.spawned_seeds.push(s);
        }

        if plant.is_dead() {
            dead_indices.push(i);
        }
    }

    dead_indices.sort_unstable_by(|a, b| b.cmp(a));
    for idx in dead_indices {
        plants.swap_remove(idx);
    }
}
