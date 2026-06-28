#![allow(unused_imports)]
pub mod tile;
pub mod climate;
pub mod evolution;
pub mod chunk;
pub mod species;
pub mod world;
pub mod events;

pub use tile::{Tile, TileType};
pub use climate::{Climate, WeatherType, WeatherParams};
pub use species::Species;
pub use chunk::{Chunk, ChunkTickResult, CHUNK_SIZE, TILE_SIZE, CHUNK_WORLD_SIZE, world_to_tile_index};
pub use world::{World, world_to_chunk_coords};
