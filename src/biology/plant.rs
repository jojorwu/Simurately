use serde::{Deserialize, Serialize};
use glam::Vec2;
use super::genome::Genome;
use rand::Rng;
use crate::engine::config::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PlantType {
    Grass,
    Shrub,
    Tree,
    Mushroom, // Растёт в тёмных / влажных местах, питает животных
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Plant {
    pub id: u64,
    pub plant_type: PlantType,
    pub genome: Genome,
    pub position: Vec2,
    pub energy: f32,
    pub health: f32,
    pub age: u32,
    pub leaf_size: f32,
    pub root_depth: f32,
    pub is_poisonous: bool, // мутация: растение ядовитое — животные теряют здоровье при поедании
}

impl super::Entity for Plant {
    fn id(&self) -> u64 { self.id }
    fn position(&self) -> Vec2 { self.position }
    fn health(&self) -> f32 { self.health }
    fn age(&self) -> u32 { self.age }
    fn is_dead(&self) -> bool {
        let max_age = match self.plant_type {
            PlantType::Grass => 300,
            PlantType::Shrub => 1000,
            PlantType::Tree => 5000,
            PlantType::Mushroom => 200,
        };
        self.health <= 0.0 || self.age >= max_age
    }
    fn genome(&self) -> &Genome { &self.genome }
}

impl Plant {
    pub fn new(id: u64, plant_type: PlantType, genome: Genome, position: Vec2) -> Self {
        let initial_energy = match plant_type {
            PlantType::Grass => 25.0,
            PlantType::Shrub => 60.0,
            PlantType::Tree => 200.0,
            PlantType::Mushroom => 40.0,
        };

        let leaf_size = genome.size * 0.7;
        let root_depth = match plant_type {
            PlantType::Grass => 0.15 * genome.size,
            PlantType::Shrub => 0.7 * genome.size,
            PlantType::Tree => 2.0 * genome.size,
            PlantType::Mushroom => 0.1 * genome.size,
        };

        let is_poisonous = rand::thread_rng().gen_range(0.0f32..1.0) < 0.05;

        Self {
            id,
            plant_type,
            genome,
            position,
            energy: initial_energy,
            health: 100.0,
            age: 0,
            leaf_size,
            root_depth,
            is_poisonous,
        }
    }

    pub fn update(
        &mut self,
        soil_energy: f32,
        temperature: f32,
        humidity: f32,
        sunlight: f32,
    ) -> (Option<(Vec2, PlantType, Genome)>, f32) {
        self.age += 1;

        let photosynthesis = self.calculate_photosynthesis(sunlight);
        let actual_absorption = self.handle_soil_absorption(soil_energy, temperature, humidity, sunlight);

        self.handle_energy_balance(photosynthesis, actual_absorption, temperature);
        self.handle_health_regen();

        let spawned_seed = self.try_reproduce();

        (spawned_seed, actual_absorption)
    }

    fn calculate_photosynthesis(&self, sunlight: f32) -> f32 {
        sunlight * self.leaf_size * match self.plant_type {
            PlantType::Grass => PHOTOSYNTHESIS_GRASS,
            PlantType::Shrub => PHOTOSYNTHESIS_SHRUB,
            PlantType::Tree => PHOTOSYNTHESIS_TREE,
            PlantType::Mushroom => 0.0,
        }
    }

    fn handle_soil_absorption(&mut self, soil_energy: f32, temperature: f32, humidity: f32, sunlight: f32) -> f32 {
        let mut absorption_mult = self.leaf_size * (0.5 + humidity * 0.5);
        if self.plant_type == PlantType::Mushroom {
            absorption_mult = humidity * 1.5 * (1.0 - sunlight * 0.5);
        }
        if humidity < 0.15 {
            absorption_mult *= self.root_depth * 0.3;
        }
        if temperature > 0.7 && humidity < 0.2 {
            absorption_mult *= 0.2;
            if self.root_depth < 0.5 { self.health -= 0.4; }
        }

        let growth_speed = match self.plant_type {
            PlantType::Grass => PLANT_GROWTH_GRASS,
            PlantType::Shrub => PLANT_GROWTH_SHRUB,
            PlantType::Tree => PLANT_GROWTH_TREE,
            PlantType::Mushroom => PLANT_GROWTH_MUSHROOM,
        };
        (soil_energy * growth_speed * absorption_mult).min(10.0).min(soil_energy).max(0.0)
    }

    fn handle_energy_balance(&mut self, photosynthesis: f32, actual_absorption: f32, temperature: f32) {
        let base_cost = 0.05 * self.genome.metabolism * self.genome.size;
        self.energy = (self.energy + photosynthesis + actual_absorption - base_cost).max(0.0);

        if temperature < -0.5 && self.plant_type != PlantType::Mushroom {
            self.health -= 0.3;
        }
        if self.energy <= 0.0 {
            self.health -= 1.5;
        }
    }

    fn handle_health_regen(&mut self) {
        if self.energy > 20.0 && self.health < 100.0 {
            self.health = (self.health + 0.4).min(100.0);
            self.energy -= 0.1;
        }
    }

    fn try_reproduce(&mut self) -> Option<(Vec2, PlantType, Genome)> {
        let repro_threshold = match self.plant_type {
            PlantType::Grass => self.genome.reproduction_threshold * 0.30,
            PlantType::Shrub => self.genome.reproduction_threshold * 0.65,
            PlantType::Tree => self.genome.reproduction_threshold * 1.2,
            PlantType::Mushroom => self.genome.reproduction_threshold * 0.45,
        };

        if self.energy > repro_threshold && self.health > 65.0 {
            self.energy -= repro_threshold * 0.40;
            let mut rng = rand::thread_rng();
            let spread_dist = match self.plant_type {
                PlantType::Grass => rng.gen_range(5.0..30.0),
                PlantType::Shrub => rng.gen_range(15.0..55.0),
                PlantType::Tree => rng.gen_range(40.0..110.0),
                PlantType::Mushroom => rng.gen_range(10.0..40.0),
            };
            let angle = rng.gen_range(0.0..std::f32::consts::TAU);
            let child_pos = self.position + Vec2::new(angle.cos() * spread_dist, angle.sin() * spread_dist);

            let child_type = if rng.gen_range(0.0f32..1.0) < 0.02 {
                match self.plant_type {
                    PlantType::Grass => PlantType::Shrub,
                    PlantType::Shrub => if rng.gen_bool(0.5) { PlantType::Tree } else { PlantType::Grass },
                    PlantType::Tree => PlantType::Shrub,
                    _ => self.plant_type,
                }
            } else { self.plant_type };

            let mut child_genome = self.genome;
            child_genome.mutate_in_place(0.04);
            return Some((child_pos, child_type, child_genome));
        }
        None
    }

    pub fn is_dead(&self) -> bool {
        <Self as super::Entity>::is_dead(self)
    }

    pub fn nutritional_value(&self) -> f32 {
        match self.plant_type {
            PlantType::Grass => 8.0,
            PlantType::Shrub => 18.0,
            PlantType::Tree => 35.0,
            PlantType::Mushroom => 25.0,
        }
    }
}
