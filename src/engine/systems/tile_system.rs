use crate::engine::tile::{Tile, TileType};
use crate::engine::climate::Climate;
use crate::engine::config::{CHUNK_SIZE, TILES_PER_TICK};

pub fn update_tiles(tiles: &mut [Tile], last_updated_tile_idx: &mut usize, climate: &Climate) {
    let mut i = *last_updated_tile_idx;
    for _ in 0..TILES_PER_TICK {
        let tile = &mut tiles[i % (CHUNK_SIZE * CHUNK_SIZE)];

        let moisture_regen = match tile.tile_type {
            TileType::Soil => climate.humidity * 0.15 - 0.02,
            TileType::Sand => climate.humidity * 0.05 - 0.02,
            TileType::Water => 0.01,
            TileType::Rock => 0.0,
        };
        tile.moisture = (tile.moisture + moisture_regen).clamp(0.0, 1.0);
        tile.temperature = tile.temperature * 0.95 + climate.temperature * 0.05;

        let energy_regen = match tile.tile_type {
            TileType::Soil => (0.1 + climate.sunlight * 0.1 + climate.humidity * 0.05).max(0.0),
            TileType::Sand => (0.02 + climate.sunlight * 0.02).max(0.0),
            _ => 0.0,
        };
        tile.energy = (tile.energy + energy_regen).min(if tile.tile_type == TileType::Soil { 200.0 } else { 50.0 });
        i += 1;
    }
    *last_updated_tile_idx = i % (CHUNK_SIZE * CHUNK_SIZE);
}
