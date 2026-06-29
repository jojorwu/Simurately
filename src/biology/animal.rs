#![allow(dead_code, unused_variables)]
use serde::{Deserialize, Serialize};
use super::genome::Genome;
use glam::Vec2;
use crate::engine::config::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AnimalType {
    Insect,
    Fish,
}

/// Детальное состояние ИИ-конечного автомата
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AiState {
    Wander,      // Бесцельное блуждание
    Forage,      // Активный поиск пищи
    Hunt,        // Преследование добычи (хищники)
    Flee,        // Бегство от хищника
    Mate,        // Поиск партнёра
    Rest,        // Отдых при высокой сытости
    Flock,       // Стайное поведение (стадо / косяк)
}

/// Вся информация, которую существо может передать наружу после обновления
#[derive(Default, Clone)]
pub struct AnimalUpdateResult {
    pub died: bool,
    pub want_to_breed_with: Option<u64>,  // ID партнёра для спаривания
    pub want_to_eat_plant_idx: Option<usize>, // Индекс растения для поедания
    pub want_to_attack: Option<u64>,      // ID жертвы для атаки
    pub offspring_count: u32,             // Сколько детёнышей произвести (обрабатывается мировым слоем)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Animal {
    pub id: u64,
    pub animal_type: AnimalType,
    pub genome: Genome,
    pub position: Vec2,
    pub velocity: Vec2,
    pub energy: f32,
    pub health: f32,
    pub age: u32,
    pub last_reproduction: u32,
    pub current_state: AiState,
    pub is_pregnant: bool,          // Беременность: детёныши появятся через pregnancy_timer тиков
    pub pregnancy_timer: u32,
    pub memory_food_pos: Option<Vec2>, // Запомненная позиция еды
    pub memory_threat_pos: Option<Vec2>, // Запомненная позиция угрозы
    pub flocking_target: Option<Vec2>, // Центр стаи
    pub fatigue: f32,               // Усталость: накапливается при быстром движении (0..100)
}

impl super::Entity for Animal {
    fn id(&self) -> u64 { self.id }
    fn position(&self) -> Vec2 { self.position }
    fn health(&self) -> f32 { self.health }
    fn age(&self) -> u32 { self.age }
    fn is_dead(&self) -> bool {
        let max_age = match self.animal_type {
            AnimalType::Insect => (500.0 + self.genome.vitality * 2.0) as u32,
            AnimalType::Fish => (1000.0 + self.genome.vitality * 4.0) as u32,
        };
        self.health <= 0.0 || self.age >= max_age
    }
    fn genome(&self) -> &Genome { &self.genome }
}

impl Animal {
    pub fn new(id: u64, animal_type: AnimalType, genome: Genome, position: Vec2) -> Self {
        let initial_energy = match animal_type {
            AnimalType::Insect => 60.0,
            AnimalType::Fish => 100.0,
        };
        let max_hp = genome.max_health();
        Self {
            id,
            animal_type,
            genome,
            position,
            velocity: Vec2::ZERO,
            energy: initial_energy,
            health: max_hp,
            age: 0,
            last_reproduction: 0,
            current_state: AiState::Wander,
            is_pregnant: false,
            pregnancy_timer: 0,
            memory_food_pos: None,
            memory_threat_pos: None,
            flocking_target: None,
            fatigue: 0.0,
        }
    }

    /// Главный метод обновления существа.
    pub fn update(
        &mut self,
        is_water_tile: bool,
        plants: &[(usize, Vec2, f32, bool)],
        nearby_animals: &[AnimalSnapshot],
        temperature: f32,
        humidity: f32,
        wind_speed: f32,
    ) -> AnimalUpdateResult {
        self.age += 1;
        self.last_reproduction += 1;

        if self.is_pregnant {
            self.pregnancy_timer += 1;
        }

        self.handle_metabolism(wind_speed, humidity);
        self.handle_terrain_effects(is_water_tile);
        self.handle_starvation_and_regen();

        if self.is_dead() {
            return AnimalUpdateResult { died: true, ..Default::default() };
        }

        let mut offspring_count = 0u32;
        if self.is_pregnant && self.pregnancy_timer >= self.gestation_period() {
            self.is_pregnant = false;
            self.pregnancy_timer = 0;
            offspring_count = self.genome.offspring_count.round() as u32;
        }

        let sensors = self.collect_sensory_data(plants, nearby_animals);
        let (steer, actions) = super::ai::decide_actions(self, sensors);

        self.apply_movement(steer, is_water_tile, wind_speed);

        AnimalUpdateResult {
            died: false,
            offspring_count,
            want_to_breed_with: actions.want_to_breed_with,
            want_to_eat_plant_idx: actions.want_to_eat_plant_idx,
            want_to_attack: actions.want_to_attack,
        }
    }

    fn handle_metabolism(&mut self, wind_speed: f32, humidity: f32) {
        let move_spd = self.velocity.length();

        if move_spd > self.genome.speed * 0.6 {
            self.fatigue = (self.fatigue + 0.3).min(100.0);
        } else {
            self.fatigue = (self.fatigue - 0.5).max(0.0);
        }

        let base_metabolism = self.genome.metabolism * (0.5 + 0.5 * self.genome.size)
            + move_spd * move_spd * 0.008;

        let weather_mult = match self.animal_type {
            AnimalType::Insect => {
                let storm_pen = wind_speed * (1.0 - self.genome.storm_resistance) * 0.5;
                let drought_pen = if humidity < 0.1 { (1.0 - self.genome.drought_resistance) * 0.3 } else { 0.0 };
                1.0 + storm_pen + drought_pen
            }
            AnimalType::Fish => {
                let storm_boost = wind_speed * (1.0 - self.genome.storm_resistance) * 0.3;
                1.0 + storm_boost
            }
        };

        self.energy -= base_metabolism * weather_mult;
    }

    fn handle_terrain_effects(&mut self, is_water_tile: bool) {
        let wrong_terrain = match self.animal_type {
            AnimalType::Insect => is_water_tile && self.genome.aquatic_adaptation < 0.4,
            AnimalType::Fish => !is_water_tile && self.genome.aquatic_adaptation > 0.6,
        };
        if wrong_terrain {
            let adaptation = if is_water_tile { self.genome.aquatic_adaptation } else { 1.0 - self.genome.aquatic_adaptation };
            self.health -= WRONG_TERRAIN_HEALTH_LOSS * (1.0 - adaptation);
        }
    }

    fn handle_starvation_and_regen(&mut self) {
        if self.energy <= 0.0 {
            self.energy = 0.0;
            self.health -= STARVATION_HEALTH_LOSS;
        }

        let max_hp = self.genome.max_health();
        if self.energy > self.genome.reproduction_threshold * REGEN_ENERGY_THRESHOLD_FACTOR && self.health < max_hp {
            let regen = REGEN_BASE_RATE * self.genome.digestion_efficiency;
            self.health = (self.health + regen).min(max_hp);
            self.energy -= regen * REGEN_ENERGY_COST_FACTOR;
        }
    }

    fn collect_sensory_data(
        &self,
        plants: &[(usize, Vec2, f32, bool)],
        nearby_animals: &[AnimalSnapshot],
    ) -> SensoryData {
        let vision = self.genome.vision_range;

        let nearest_predator = nearby_animals.iter()
            .filter(|(id, pos, t, size, diet, aggression, _, _, _)| {
                *id != self.id && pos.distance(self.position) < vision && self.is_threatened_by(*t, *size, *diet, *aggression)
            })
            .map(|(_, pos, _, _, _, _, _, _, _)| (*pos, pos.distance(self.position)))
            .min_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));

        let nearest_prey = if self.genome.diet > 0.4 {
            nearby_animals.iter()
                .filter(|(id, pos, t, size, _, _, prey_energy, _, _)| {
                    *id != self.id && pos.distance(self.position) < vision && self.can_eat_animal(*t, *size) && *prey_energy > 0.0
                })
                .map(|(id, pos, _, _, _, _, _, _, _)| (*id, *pos, pos.distance(self.position)))
                .min_by(|a, b| a.2.partial_cmp(&b.2).unwrap_or(std::cmp::Ordering::Equal))
        } else { None };

        let nearest_plant = if self.genome.diet < 0.7 {
            plants.iter()
                .filter(|(_, pos, energy, _)| pos.distance(self.position) < vision && *energy > 5.0)
                .map(|(idx, pos, energy, _)| (*idx, *pos, pos.distance(self.position)))
                .min_by(|a, b| a.2.partial_cmp(&b.2).unwrap_or(std::cmp::Ordering::Equal))
        } else { None };

        let nearest_mate = nearby_animals.iter()
            .filter(|(id, pos, t, _, _, _, mate_energy, spec_id, _)| {
                *id != self.id && *t == self.animal_type && *spec_id == self.genome.species_id && pos.distance(self.position) < vision && *mate_energy > self.genome.reproduction_threshold * 0.6
            })
            .map(|(id, pos, _, _, _, _, _, _, _)| (*id, *pos, pos.distance(self.position)))
            .min_by(|a, b| a.2.partial_cmp(&b.2).unwrap_or(std::cmp::Ordering::Equal));

        let flock_center = if self.genome.sociality > 0.5 {
            let neighbors: Vec<Vec2> = nearby_animals.iter()
                .filter(|(id, pos, t, _, _, _, _, spec_id, _)| {
                    *id != self.id && *t == self.animal_type && *spec_id == self.genome.species_id && pos.distance(self.position) < vision * 0.8
                })
                .map(|(_, pos, _, _, _, _, _, _, _)| *pos).collect();
            if neighbors.len() >= 2 { Some(neighbors.iter().sum::<Vec2>() / neighbors.len() as f32) } else { None }
        } else { None };

        SensoryData { nearest_predator, nearest_prey, nearest_plant, nearest_mate, flock_center }
    }


    fn apply_movement(&mut self, steer: Vec2, is_water_tile: bool, wind_speed: f32) {
        let max_force = self.genome.speed * 0.25 * if self.fatigue > 70.0 { 0.5 } else { 1.0 };
        self.velocity = (self.velocity + steer.clamp_length_max(max_force)).clamp_length_max(self.genome.speed);

        let terrain_mult = match self.animal_type {
            AnimalType::Insect => if is_water_tile { 0.3 } else { 1.0 },
            AnimalType::Fish => if is_water_tile { 1.0 } else { 0.3 },
        };
        let wind_pen = match self.animal_type {
            AnimalType::Insect => 1.0 - wind_speed * (1.0 - self.genome.storm_resistance) * 0.35,
            AnimalType::Fish => 1.0,
        };
        self.position += self.velocity * terrain_mult * wind_pen.max(0.1);
    }

    fn seek(&self, target: Vec2) -> Vec2 {
        (target - self.position).normalize_or_zero() * self.genome.speed - self.velocity
    }

    fn is_threatened_by(&self, other_type: AnimalType, other_size: f32, other_diet: f32, other_aggression: f32) -> bool {
        let other_is_predator = other_diet > 0.5 && other_aggression > 0.3;
        let other_bigger = other_size > self.genome.size * 0.9;
        match self.animal_type {
            AnimalType::Insect => (other_type == AnimalType::Fish && other_bigger) || (other_type == AnimalType::Insect && other_is_predator && other_bigger),
            AnimalType::Fish => other_type == AnimalType::Fish && other_is_predator && other_bigger,
        }
    }

    fn can_eat_animal(&self, other_type: AnimalType, other_size: f32) -> bool {
        match self.animal_type {
            AnimalType::Insect => other_type == AnimalType::Insect && other_size < self.genome.size,
            AnimalType::Fish => other_type == AnimalType::Insect || (other_type == AnimalType::Fish && other_size < self.genome.size * 0.75),
        }
    }

    pub fn gestation_period(&self) -> u32 {
        (60.0 + self.genome.offspring_count * 20.0 + self.genome.size * 15.0) as u32
    }

    pub fn is_dead(&self) -> bool {
        <Self as super::Entity>::is_dead(self)
    }

    pub fn max_health(&self) -> f32 {
        self.genome.max_health()
    }
}

pub struct SensoryData {
    pub nearest_predator: Option<(Vec2, f32)>,
    pub nearest_prey: Option<(u64, Vec2, f32)>,
    pub nearest_plant: Option<(usize, Vec2, f32)>,
    pub nearest_mate: Option<(u64, Vec2, f32)>,
    pub flock_center: Option<Vec2>,
}

#[derive(Default)]
pub struct AnimalUpdateActions {
    pub want_to_breed_with: Option<u64>,
    pub want_to_eat_plant_idx: Option<usize>,
    pub want_to_attack: Option<u64>,
}

pub type AnimalSnapshot = (u64, Vec2, AnimalType, f32, f32, f32, f32, u32, f32);
