/// Side length of a square tile in world units.
pub const TILE_SIZE: f32 = 10.0;
/// Number of tiles along one side of a square chunk.
pub const CHUNK_SIZE: usize = 64;
/// Interval in ticks at which plant growth logic is updated.
pub const PLANT_UPDATE_INTERVAL: u64 = 5;
pub const CHUNK_WORLD_SIZE: f32 = CHUNK_SIZE as f32 * TILE_SIZE;
pub const TILES_PER_TICK: usize = 100;
pub const MAX_LOG_ENTRIES: usize = 150;
pub const LOG_DRAIN_COUNT: usize = 50;
pub const STATS_HISTORY_SIZE: usize = 600;

pub const GRID_CELL_SIZE: f32 = 40.0;
pub const GRID_WIDTH: usize = (CHUNK_WORLD_SIZE / GRID_CELL_SIZE) as usize + 1;

// AI and Combat
pub const ATTACK_RANGE_BASE: f32 = 10.0;
pub const ATTACK_RANGE_SIZE_MULT: f32 = 8.0;
pub const EAT_RANGE_BASE: f32 = 12.0;
pub const EAT_RANGE_SIZE_MULT: f32 = 6.0;
pub const MATE_RANGE: f32 = 15.0;
pub const FLEE_DIST_THRESHOLD: f32 = 60.0;
pub const FLOCK_DIST_TARGET: f32 = 60.0;
pub const SEPARATION_DIST: f32 = 20.0;

// Metabolism and Health
pub const STARVATION_HEALTH_LOSS: f32 = 1.0;
pub const REGEN_ENERGY_THRESHOLD_FACTOR: f32 = 0.5;
pub const REGEN_BASE_RATE: f32 = 0.4;
pub const REGEN_ENERGY_COST_FACTOR: f32 = 0.3;
pub const WRONG_TERRAIN_HEALTH_LOSS: f32 = 2.5;

// Plant Growth and Photosynthesis
pub const PHOTOSYNTHESIS_GRASS: f32 = 0.4;
pub const PHOTOSYNTHESIS_SHRUB: f32 = 0.25;
pub const PHOTOSYNTHESIS_TREE: f32 = 0.12;
pub const PLANT_GROWTH_GRASS: f32 = 0.40;
pub const PLANT_GROWTH_SHRUB: f32 = 0.20;
pub const PLANT_GROWTH_TREE: f32 = 0.07;
pub const PLANT_GROWTH_MUSHROOM: f32 = 0.35;
