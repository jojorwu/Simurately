use glam::Vec2;
use rand::Rng;
use crate::biology::animal::{Animal, AiState, SensoryData, AnimalUpdateActions};
use crate::engine::config::*;

pub fn decide_actions(animal: &mut Animal, sensors: SensoryData) -> (Vec2, AnimalUpdateActions) {
    let mut steer = Vec2::ZERO;
    let mut actions = AnimalUpdateActions::default();
    let hunger_ratio = animal.energy / animal.genome.reproduction_threshold.max(50.0);

    // Flee
    if let Some((pred_pos, pred_dist)) = sensors.nearest_predator {
        let threat_power = 1.0;
        let my_power = animal.genome.size * animal.genome.speed;
        let danger_ratio = threat_power / (my_power * animal.genome.fear_threshold).max(0.1);
        if danger_ratio > 0.6 || pred_dist < FLEE_DIST_THRESHOLD {
            animal.current_state = AiState::Flee;
            animal.memory_threat_pos = Some(pred_pos);
            steer = (animal.position - pred_pos).normalize_or_zero() * animal.genome.speed - animal.velocity;
        }
    } else if let Some(threat_pos) = animal.memory_threat_pos {
        steer = (animal.position - threat_pos).normalize_or_zero() * animal.genome.speed * 0.5 - animal.velocity;
        if animal.age % 30 == 0 { animal.memory_threat_pos = None; }
    }

    // Hunt
    if steer == Vec2::ZERO && animal.genome.diet > 0.5 && hunger_ratio < 1.5 {
        if let Some((prey_id, prey_pos, prey_dist)) = sensors.nearest_prey {
            animal.current_state = AiState::Hunt;
            if prey_dist < animal.genome.size * ATTACK_RANGE_SIZE_MULT + ATTACK_RANGE_BASE { actions.want_to_attack = Some(prey_id); }
            else { steer = seek(animal, prey_pos); }
        }
    }

    // Forage
    if steer == Vec2::ZERO && animal.genome.diet < 0.7 && hunger_ratio < 1.2 {
        let food_target = sensors.nearest_plant.map(|(idx, pos, _)| (idx, pos)).or_else(|| animal.memory_food_pos.map(|p| (usize::MAX, p)));
        if let Some((plant_idx, plant_pos)) = food_target {
            animal.current_state = AiState::Forage;
            if plant_idx != usize::MAX { animal.memory_food_pos = Some(plant_pos); }
            if animal.position.distance(plant_pos) < animal.genome.size * EAT_RANGE_SIZE_MULT + EAT_RANGE_BASE {
                actions.want_to_eat_plant_idx = if plant_idx != usize::MAX { Some(plant_idx) } else { None };
                animal.memory_food_pos = None;
            } else { steer = seek(animal, plant_pos); }
        }
    }

    // Mate
    if steer == Vec2::ZERO && animal.energy > animal.genome.reproduction_threshold && animal.last_reproduction > animal.genome.reproduction_cooldown as u32 && !animal.is_pregnant {
        if let Some((mate_id, mate_pos, mate_dist)) = sensors.nearest_mate {
            animal.current_state = AiState::Mate;
            if mate_dist < MATE_RANGE { actions.want_to_breed_with = Some(mate_id); }
            else { steer = seek(animal, mate_pos); }
        }
    }

    // Flock
    if steer == Vec2::ZERO && animal.genome.sociality > 0.4 {
        if let Some(center) = sensors.flock_center {
            animal.current_state = AiState::Flock;
            animal.flocking_target = Some(center);
            let dist = animal.position.distance(center);
            if dist > FLOCK_DIST_TARGET { steer = seek(animal, center) * animal.genome.sociality; }
            else if dist < SEPARATION_DIST { steer = (animal.position - center).normalize_or_zero() * animal.genome.speed * 0.3; }
        }
    }

    // Rest
    if steer == Vec2::ZERO && hunger_ratio > 1.8 && animal.fatigue > 40.0 {
        animal.current_state = AiState::Rest;
        steer = -animal.velocity * 0.3;
    }

    // Wander
    if steer == Vec2::ZERO {
        animal.current_state = AiState::Wander;
        let mut rng = rand::thread_rng();
        let base_dir = if animal.velocity.length_squared() > 0.01 { animal.velocity.normalize() }
        else { let a = rng.gen_range(0.0..std::f32::consts::TAU); Vec2::new(a.cos(), a.sin()) };
        let rot = glam::Mat2::from_angle(rng.gen_range(-0.5..0.5));
        let wander_speed = animal.genome.speed * if animal.fatigue > 60.0 { 0.4 } else { 0.65 };
        steer = rot * base_dir * wander_speed - animal.velocity;
    }

    (steer, actions)
}

fn seek(animal: &Animal, target: Vec2) -> Vec2 {
    (target - animal.position).normalize_or_zero() * animal.genome.speed - animal.velocity
}
