use serde::{Deserialize, Serialize};
use crate::biology::genome::Genome;
use crate::biology::animal::AnimalType;

#[derive(Clone, Serialize, Deserialize)]
pub struct Species {
    pub id: u32,
    pub name: String,
    pub avg_genome: Genome,
    pub color: [u8; 3],
    pub population: usize,
    pub total_born: u64,   // Всего рождено
    pub total_died: u64,   // Всего погибло
    pub active: bool,
    pub base_type: AnimalType,
    pub founded_at_tick: u64,
}

/// Генерация процедурных названий видов
pub fn generate_species_name(genome: &Genome, animal_type: AnimalType) -> String {
    let size_adj = if genome.size > 4.0 { "Колосс" }
        else if genome.size > 2.0 { "Великан" }
        else if genome.size < 0.5 { "Карлик" }
        else { "Обычный" };

    let _speed_adj = if genome.speed > 12.0 { "Молния" }
        else if genome.speed > 7.0 { "Быстрый" }
        else if genome.speed < 2.0 { "Медлительный" }
        else { "Средний" };

    let (r, g, b) = (
        (genome.color_r * 255.0) as u8,
        (genome.color_g * 255.0) as u8,
        (genome.color_b * 255.0) as u8,
    );
    let color_adj = if r > 160 && g < 100 && b < 100 { "Алый" }
        else if g > 160 && r < 100 && b < 100 { "Изумрудный" }
        else if b > 160 && r < 100 && g < 100 { "Лазурный" }
        else if r > 150 && g > 150 && b < 80 { "Янтарный" }
        else if r > 150 && b > 150 && g < 80 { "Пурпурный" }
        else if g > 150 && b > 150 && r < 80 { "Бирюзовый" }
        else if r > 180 && g > 180 && b > 180 { "Серебристый" }
        else if r < 70 && g < 70 && b < 70 { "Тёмный" }
        else { "Пёстрый" };

    let type_suffix = match animal_type {
        AnimalType::Insect => if genome.diet > 0.6 { "Охотник" } else if genome.sociality > 0.7 { "Рой" } else { "Бродяга" },
        AnimalType::Fish => if genome.diet > 0.6 { "Хищник" } else if genome.sociality > 0.7 { "Стая" } else { "Скиталец" },
    };

    format!("{} {} {}", size_adj, color_adj, type_suffix)
}
