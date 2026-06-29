use glam::Vec2;
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WorldEvent {
    LightningStrike(Vec2),
    WeatherChanged(String),
    SeasonChanged(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BioEvent {
    AnimalDied { id: u64, reason: String },
    AnimalBorn { id: u64, parent_id: Option<u64>, pos: Vec2 },
    AnimalAte { hunter_id: u64, target_id: u64 },
    PlantEaten { animal_id: u64, plant_id: u64, energy: f32 },
    SpeciesExtinct { species_id: u32, name: String },
    MutationOccurred { entity_id: u64, description: String },
}
