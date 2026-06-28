#![allow(dead_code, unused_variables)]
use serde::{Deserialize, Serialize};
use super::genome::Genome;
use glam::Vec2;
use rand::Rng;

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
    ///
    /// Параметры среды:
    /// - `is_water_tile` — существо на воде?
    /// - `plants` — список позиций и индексов растений в зоне видимости
    /// - `nearby_animals` — соседние существа (id, pos, type, genome-snapshot, energy)
    /// - `temperature`, `humidity`, `wind_speed` — параметры погоды
    pub fn update(
        &mut self,
        is_water_tile: bool,
        plants: &[(usize, Vec2, f32, bool)], // (idx, pos, energy, is_poisonous)
        nearby_animals: &[(u64, Vec2, AnimalType, f32, f32, f32, f32, u32, f32)], // (id, pos, type, size, diet, aggression, energy, species_id, aquatic)
        temperature: f32,
        humidity: f32,
        wind_speed: f32,
    ) -> AnimalUpdateResult {
        self.age += 1;
        self.last_reproduction += 1;

        if self.is_pregnant {
            self.pregnancy_timer += 1;
        }

        // ---- 1. ЗАТРАТЫ ЭНЕРГИИ И ЭФФЕКТЫ ПОГОДЫ ----
        let move_spd = self.velocity.length();

        // Усталость накапливается при быстром движении
        if move_spd > self.genome.speed * 0.6 {
            self.fatigue = (self.fatigue + 0.3).min(100.0);
        } else {
            self.fatigue = (self.fatigue - 0.5).max(0.0);
        }

        // Базовый метаболизм (зависит от размера и скорости)
        let base_metabolism = self.genome.metabolism * (0.5 + 0.5 * self.genome.size)
            + move_spd * move_spd * 0.008;

        // Погода влияет на метаболизм
        let weather_mult = match self.animal_type {
            AnimalType::Insect => {
                let storm_pen = wind_speed * (1.0 - self.genome.storm_resistance) * 0.5;
                let drought_pen = if humidity < 0.1 { (1.0 - self.genome.drought_resistance) * 0.3 } else { 0.0 };
                1.0 + storm_pen + drought_pen
            }
            AnimalType::Fish => {
                // Рыбы при шторме тратят больше на плавание в течениях
                let storm_boost = wind_speed * (1.0 - self.genome.storm_resistance) * 0.3;
                1.0 + storm_boost
            }
        };

        let energy_cost = base_metabolism * weather_mult;
        self.energy -= energy_cost;

        // ---- 2. НЕПРАВИЛЬНЫЙ ЛАНДШАФТ ----
        let wrong_terrain = match self.animal_type {
            AnimalType::Insect => is_water_tile && self.genome.aquatic_adaptation < 0.4,
            AnimalType::Fish => !is_water_tile && self.genome.aquatic_adaptation > 0.6,
        };
        if wrong_terrain {
            self.health -= 2.5 * (1.0 - if is_water_tile { self.genome.aquatic_adaptation } else { 1.0 - self.genome.aquatic_adaptation });
        }

        // ---- 3. ГОЛОДАНИЕ ----
        if self.energy <= 0.0 {
            self.energy = 0.0;
            self.health -= 1.0;
        }

        // ---- 4. ВОССТАНОВЛЕНИЕ ЗДОРОВЬЯ ----
        let max_hp = self.genome.max_health();
        if self.energy > self.genome.reproduction_threshold * 0.5 && self.health < max_hp {
            let regen = 0.4 * self.genome.digestion_efficiency;
            self.health = (self.health + regen).min(max_hp);
            self.energy -= regen * 0.3;
        }

        // ---- 5. ПРОВЕРКА НА СМЕРТЬ ----
        if self.health <= 0.0 || self.is_dead() {
            return AnimalUpdateResult { died: true, ..Default::default() };
        }

        // ---- 6. ПЕРЕБЕРЕМЕННОСТЬ — РОДЫ ----
        let mut offspring_count = 0u32;
        if self.is_pregnant && self.pregnancy_timer >= self.gestation_period() {
            self.is_pregnant = false;
            self.pregnancy_timer = 0;
            offspring_count = self.genome.offspring_count.round() as u32;
        }

        // ---- 7. ИИ: СБОР СЕНСОРНОЙ ИНФОРМАЦИИ ----
        let my_id = self.id;
        let my_pos = self.position;
        let my_type = self.animal_type;
        let vision = self.genome.vision_range;

        // Ближайший хищник (угроза)
        let nearest_predator: Option<(Vec2, f32)> = nearby_animals.iter()
            .filter(|(id, pos, t, size, diet, aggression, _, _, _)| {
                *id != my_id
                && pos.distance(my_pos) < vision
                && self.is_threatened_by(*t, *size, *diet, *aggression)
            })
            .map(|(_, pos, _, _, _, _, _, _, _)| {
                let dist = pos.distance(my_pos);
                (*pos, dist)
            })
            .min_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal))
            .map(|(pos, dist)| (pos, dist));

        // Ближайшая добыча (для хищников)
        let nearest_prey: Option<(u64, Vec2, f32)> = if self.genome.diet > 0.4 {
            nearby_animals.iter()
                .filter(|(id, pos, t, size, _, _, prey_energy, _, _)| {
                    *id != my_id
                    && pos.distance(my_pos) < vision
                    && self.can_eat_animal(*t, *size)
                    && *prey_energy > 0.0
                })
                .map(|(id, pos, _, _, _, _, _, _, _)| {
                    let dist = pos.distance(my_pos);
                    (*id, *pos, dist)
                })
                .min_by(|a, b| a.2.partial_cmp(&b.2).unwrap_or(std::cmp::Ordering::Equal))
        } else {
            None
        };

        // Ближайшее растение (для травоядных и всеядных)
        let nearest_plant: Option<(usize, Vec2, f32)> = if self.genome.diet < 0.7 {
            plants.iter()
                .filter(|(_, pos, energy, _)| {
                    pos.distance(my_pos) < vision && *energy > 5.0
                })
                .map(|(idx, pos, energy, _)| {
                    let dist = pos.distance(my_pos);
                    (*idx, *pos, dist)
                })
                .min_by(|a, b| a.2.partial_cmp(&b.2).unwrap_or(std::cmp::Ordering::Equal))
        } else {
            None
        };

        // Ближайший партнёр того же вида
        let nearest_mate: Option<(u64, Vec2, f32)> = nearby_animals.iter()
            .filter(|(id, pos, t, _, _, _, mate_energy, spec_id, _)| {
                *id != my_id
                && *t == my_type
                && *spec_id == self.genome.species_id
                && pos.distance(my_pos) < vision
                && *mate_energy > self.genome.reproduction_threshold * 0.6
            })
            .map(|(id, pos, _, _, _, _, _, _, _)| {
                let dist = pos.distance(my_pos);
                (*id, *pos, dist)
            })
            .min_by(|a, b| a.2.partial_cmp(&b.2).unwrap_or(std::cmp::Ordering::Equal));

        // Центр стаи своего вида (для flocking)
        let flock_center: Option<Vec2> = if self.genome.sociality > 0.5 {
            let same_species: Vec<Vec2> = nearby_animals.iter()
                .filter(|(id, pos, t, _, _, _, _, spec_id, _)| {
                    *id != my_id && *t == my_type && *spec_id == self.genome.species_id
                    && pos.distance(my_pos) < vision * 0.8
                })
                .map(|(_, pos, _, _, _, _, _, _, _)| *pos)
                .collect();

            if same_species.len() >= 2 {
                let sum: Vec2 = same_species.iter().sum();
                Some(sum / same_species.len() as f32)
            } else {
                None
            }
        } else {
            None
        };

        // ---- 8. ИИ: ПРИНЯТИЕ РЕШЕНИЙ (ИЕРАРХИЯ ПРИОРИТЕТОВ) ----
        let mut steer = Vec2::ZERO;
        let mut want_to_eat_plant_idx = None;
        let mut want_to_attack = None;
        let mut want_to_breed_with = None;
        let hunger_ratio = self.energy / self.genome.reproduction_threshold.max(50.0);

        // Приоритет 1: БЕГСТВО от хищника
        if let Some((pred_pos, pred_dist)) = nearest_predator {
            let threat_power = 1.0; // упрощённо
            let my_power = self.genome.size * self.genome.speed;
            let danger_ratio = threat_power / (my_power * self.genome.fear_threshold).max(0.1);

            if danger_ratio > 0.6 || pred_dist < 60.0 {
                self.current_state = AiState::Flee;
                self.memory_threat_pos = Some(pred_pos);

                let flee_dir = (my_pos - pred_pos).normalize_or_zero();
                // При бегстве — максимальная скорость
                steer = flee_dir * self.genome.speed - self.velocity;
            }
        } else if let Some(threat_pos) = self.memory_threat_pos {
            // Помним угрозу ещё несколько тиков
            let flee_dir = (my_pos - threat_pos).normalize_or_zero();
            steer = flee_dir * self.genome.speed * 0.5 - self.velocity;
            if self.age % 30 == 0 {
                self.memory_threat_pos = None;
            }
        }

        // Приоритет 2: ОХОТА на добычу (хищники с высоким diet)
        if steer == Vec2::ZERO && self.genome.diet > 0.5 && hunger_ratio < 1.5 {
            if let Some((prey_id, prey_pos, prey_dist)) = nearest_prey {
                self.current_state = AiState::Hunt;
                let attack_range = self.genome.size * 8.0 + 10.0;
                if prey_dist < attack_range {
                    want_to_attack = Some(prey_id);
                } else {
                    steer = self.seek(prey_pos);
                }
            }
        }

        // Приоритет 3: ПОИСК РАСТИТЕЛЬНОЙ ПИЩи (травоядные / при голоде)
        if steer == Vec2::ZERO && self.genome.diet < 0.7 && hunger_ratio < 1.2 {
            // Используем запомненную позицию еды или ищем новую
            let food_target = nearest_plant
                .as_ref()
                .map(|(idx, pos, _)| (*idx, *pos))
                .or_else(|| self.memory_food_pos.map(|p| (usize::MAX, p)));

            if let Some((plant_idx, plant_pos)) = food_target {
                self.current_state = AiState::Forage;
                let eat_range = self.genome.size * 6.0 + 12.0;

                if plant_idx != usize::MAX {
                    self.memory_food_pos = Some(plant_pos);
                }

                if my_pos.distance(plant_pos) < eat_range {
                    want_to_eat_plant_idx = if plant_idx != usize::MAX { Some(plant_idx) } else { None };
                    self.memory_food_pos = None;
                } else {
                    steer = self.seek(plant_pos);
                }
            }
        }

        // Приоритет 4: РАЗМНОЖЕНИЕ
        if steer == Vec2::ZERO
            && self.energy > self.genome.reproduction_threshold
            && self.last_reproduction > self.genome.reproduction_cooldown as u32
            && !self.is_pregnant
        {
            if let Some((mate_id, mate_pos, mate_dist)) = nearest_mate {
                self.current_state = AiState::Mate;
                let mate_range = 15.0;
                if mate_dist < mate_range {
                    want_to_breed_with = Some(mate_id);
                } else {
                    steer = self.seek(mate_pos);
                }
            }
        }

        // Приоритет 5: СТАЙНОЕ ПОВЕДЕНИЕ
        if steer == Vec2::ZERO && self.genome.sociality > 0.4 {
            if let Some(center) = flock_center {
                self.current_state = AiState::Flock;
                self.flocking_target = Some(center);
                // Коhesion: тянемся к центру, но не слишком близко
                let dist_to_center = my_pos.distance(center);
                if dist_to_center > 60.0 {
                    steer = self.seek(center) * self.genome.sociality;
                } else if dist_to_center < 20.0 {
                    steer = (my_pos - center).normalize_or_zero() * self.genome.speed * 0.3; // Separation
                }
            }
        }

        // Приоритет 6: ОТДЫХ при высокой сытости
        if steer == Vec2::ZERO && hunger_ratio > 1.8 && self.fatigue > 40.0 {
            self.current_state = AiState::Rest;
            steer = -self.velocity * 0.3; // Торможение
        }

        // Приоритет 7: БЛУЖДАНИЕ (нет других целей)
        if steer == Vec2::ZERO {
            self.current_state = AiState::Wander;
            let mut rng = rand::thread_rng();
            let wander_noise = rng.gen_range(-0.5f32..0.5);
            let base_dir = if self.velocity.length_squared() > 0.01 {
                self.velocity.normalize()
            } else {
                let a = rng.gen_range(0.0..std::f32::consts::TAU);
                Vec2::new(a.cos(), a.sin())
            };
            let rot = glam::Mat2::from_angle(wander_noise);
            let wander_speed = self.genome.speed * if self.fatigue > 60.0 { 0.4 } else { 0.65 };
            steer = rot * base_dir * wander_speed - self.velocity;
        }

        // ---- 9. ПРИМЕНЕНИЕ СИЛ РУЛЕВОГО УПРАВЛЕНИЯ ----
        let max_force = self.genome.speed * 0.25 * if self.fatigue > 70.0 { 0.5 } else { 1.0 };
        let steering = steer.clamp_length_max(max_force);
        self.velocity = (self.velocity + steering).clamp_length_max(self.genome.speed);

        // Коэффициент скорости по типу
        let terrain_speed_mult = match self.animal_type {
            AnimalType::Insect => if is_water_tile { 0.3 } else { 1.0 },
            AnimalType::Fish => if is_water_tile { 1.0 } else { 0.3 },
        };
        // Ветер замедляет насекомых
        let wind_penalty = match self.animal_type {
            AnimalType::Insect => 1.0 - wind_speed * (1.0 - self.genome.storm_resistance) * 0.35,
            AnimalType::Fish => 1.0,
        };
        self.position += self.velocity * terrain_speed_mult * wind_penalty.max(0.1);

        AnimalUpdateResult {
            died: false,
            want_to_breed_with,
            want_to_eat_plant_idx,
            want_to_attack,
            offspring_count,
        }
    }

    /// Вспомогательная функция: рулевое стремление к точке
    fn seek(&self, target: Vec2) -> Vec2 {
        let desired = (target - self.position).normalize_or_zero() * self.genome.speed;
        desired - self.velocity
    }

    /// Проверяет, является ли другое существо угрозой
    fn is_threatened_by(&self, other_type: AnimalType, other_size: f32, other_diet: f32, other_aggression: f32) -> bool {
        let other_is_predator = other_diet > 0.5 && other_aggression > 0.3;
        let other_bigger = other_size > self.genome.size * 0.9;
        match self.animal_type {
            AnimalType::Insect => {
                (other_type == AnimalType::Fish && other_bigger) ||
                (other_type == AnimalType::Insect && other_is_predator && other_bigger)
            }
            AnimalType::Fish => {
                other_type == AnimalType::Fish && other_is_predator && other_bigger
            }
        }
    }

    /// Проверяет, может ли существо съесть другое животное
    fn can_eat_animal(&self, other_type: AnimalType, other_size: f32) -> bool {
        match self.animal_type {
            AnimalType::Insect => other_type == AnimalType::Insect && other_size < self.genome.size,
            AnimalType::Fish => {
                other_type == AnimalType::Insect ||
                (other_type == AnimalType::Fish && other_size < self.genome.size * 0.75)
            }
        }
    }

    /// Период беременности в тиках
    pub fn gestation_period(&self) -> u32 {
        (60.0 + self.genome.offspring_count * 20.0 + self.genome.size * 15.0) as u32
    }

    pub fn is_dead(&self) -> bool {
        let max_age = match self.animal_type {
            AnimalType::Insect => (500.0 + self.genome.vitality * 2.0) as u32,
            AnimalType::Fish => (1000.0 + self.genome.vitality * 4.0) as u32,
        };
        self.health <= 0.0 || self.age >= max_age
    }

    /// Максимальное здоровье из генома
    pub fn max_health(&self) -> f32 {
        self.genome.max_health()
    }
}
