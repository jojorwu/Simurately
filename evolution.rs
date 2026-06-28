use std::collections::HashMap;
use crate::biology::genome::Genome;
use crate::biology::animal::AnimalType;
use crate::engine::species::Species;

pub struct EvolutionManager {
    pub species_registry: HashMap<u32, Species>,
    pub species_by_type: HashMap<AnimalType, Vec<u32>>,
    pub next_species_id: u32,
}

impl EvolutionManager {
    pub fn new() -> Self {
        Self {
            species_registry: HashMap::new(),
            species_by_type: HashMap::new(),
            next_species_id: 1,
        }
    }

    pub fn register_or_match_species(&mut self, genome: &mut Genome, animal_type: AnimalType) -> u32 {
        let parent_id = genome.species_id;

        // Сначала сверяем с родительским видом
        if parent_id != 0 {
            if let Some(spec) = self.species_registry.get(&parent_id) {
                if spec.avg_genome.genetic_distance(genome) < 0.22 {
                    return parent_id;
                }
            }
        }

        // Сверяем со всеми активными видами
        let best = self.species_registry.values()
            .filter(|s| s.active && s.base_type == animal_type)
            .map(|s| (s.id, s.avg_genome.genetic_distance(genome)))
            .min_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));

        if let Some((id, dist)) = best {
            if dist < 0.22 {
                return id;
            }
        }

        // Новый вид!
        let new_id = self.next_species_id;
        self.next_species_id += 1;

        let name = crate::engine::species::generate_species_name(genome, animal_type);
        let color = [(genome.color_r * 255.0) as u8, (genome.color_g * 255.0) as u8, (genome.color_b * 255.0) as u8];
        
        self.species_registry.insert(new_id, Species {
            id: new_id,
            name: name.clone(),
            avg_genome: *genome,
            color,
            population: 0,
            total_born: 0,
            total_died: 0,
            active: true,
            base_type: animal_type,
            founded_at_tick: 0, // Will be set by World
        });

        new_id
    }

    pub fn update_populations(&mut self, tick_count: u64) -> Vec<String> {
        let mut logs = Vec::new();
        for spec in self.species_registry.values_mut() {
            if spec.population == 0 && spec.active {
                spec.active = false;
                logs.push(format!(
                    "[Тик {}] ВЫМИРАНИЕ: Вид '{}' исчез! (прожил {} тиков)",
                    tick_count, spec.name, tick_count - spec.founded_at_tick
                ));
            } else if spec.population > 0 && !spec.active {
                spec.active = true;
            }
        }
        logs
    }
}
