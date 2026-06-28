use serde::{Deserialize, Serialize};
use rand::Rng;

/// Расширенный геном существа. Каждый ген закодирован как f32 в допустимом диапазоне.
/// Гены влияют на всё поведение, внешность и выживаемость особи.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Genome {
    // ---- Физические параметры ----
    /// Базовые затраты энергии в тик (0.05..5.0). Высокий метаболизм = быстрое развитие, но нужно больше еды.
    pub metabolism: f32,
    /// Максимальная скорость передвижения (0.5..18.0)
    pub speed: f32,
    /// Физический размер тела (0.1..8.0). Влияет на урон в бою, кол-во потребляемой пищи
    pub size: f32,
    /// Агрессия: насколько охотно атакует других (0.0..1.0)
    pub aggression: f32,
    /// Жизнестойкость: базовое максимальное здоровье (50..300)
    pub vitality: f32,

    // ---- Рацион и пищеварение ----
    /// Рацион: 0.0 (чистый травоядный) .. 1.0 (чистый плотоядный хищник)
    pub diet: f32,
    /// КПД пищеварения (0.4..1.0): насколько хорошо усваивается пища
    pub digestion_efficiency: f32,

    // ---- Чувства и ИИ ----
    /// Дальность зрения (40.0..350.0)
    pub vision_range: f32,
    /// Порог страха: при каком соотношении сил убегать (0.5..2.0)
    pub fear_threshold: f32,
    /// Социальность: склонность к стайному поведению (0.0..1.0)
    pub sociality: f32,

    // ---- Размножение ----
    /// Минимальная энергия для готовности к спариванию (30.0..400.0)
    pub reproduction_threshold: f32,
    /// Количество детёнышей за раз (1..4, но хранится как f32 для мутаций)
    pub offspring_count: f32,
    /// Кулдаун между размножениями в тиках (60..500)
    pub reproduction_cooldown: f32,

    // ---- Устойчивость к среде ----
    /// Устойчивость к засухе (0.0..1.0)
    pub drought_resistance: f32,
    /// Устойчивость к холоду/буре (0.0..1.0)
    pub storm_resistance: f32,
    /// Адаптация к воде (0.0..1.0): для рыб высокое, для насекомых низкое
    pub aquatic_adaptation: f32,

    // ---- Визуальный генотип ----
    pub color_r: f32,
    pub color_g: f32,
    pub color_b: f32,

    // ---- Метаданные вида ----
    pub generation: u32,
    pub species_id: u32,
}

impl Genome {
    /// Создаёт стандартный геном для насекомого
    pub fn default_insect() -> Self {
        Self {
            metabolism: 0.3,
            speed: 3.0,
            size: 1.0,
            aggression: 0.2,
            vitality: 80.0,
            diet: 0.1,
            digestion_efficiency: 0.8,
            vision_range: 120.0,
            fear_threshold: 1.0,
            sociality: 0.3,
            reproduction_threshold: 80.0,
            offspring_count: 2.0,
            reproduction_cooldown: 180.0,
            drought_resistance: 0.3,
            storm_resistance: 0.2,
            aquatic_adaptation: 0.05,
            color_r: 0.6, color_g: 0.8, color_b: 0.3,
            generation: 1,
            species_id: 0,
        }
    }

    /// Создаёт стандартный геном для рыбы
    pub fn default_fish() -> Self {
        Self {
            metabolism: 0.25,
            speed: 4.0,
            size: 1.5,
            aggression: 0.3,
            vitality: 120.0,
            diet: 0.3,
            digestion_efficiency: 0.85,
            vision_range: 150.0,
            fear_threshold: 1.2,
            sociality: 0.6,
            reproduction_threshold: 120.0,
            offspring_count: 2.0,
            reproduction_cooldown: 250.0,
            drought_resistance: 0.05,
            storm_resistance: 0.6,
            aquatic_adaptation: 0.95,
            color_r: 0.2, color_g: 0.5, color_b: 0.9,
            generation: 1,
            species_id: 0,
        }
    }

    /// Генерирует случайный геном (для начального спавна)
    pub fn random() -> Self {
        let mut rng = rand::thread_rng();
        Self {
            metabolism: rng.gen_range(0.05..2.5),
            speed: rng.gen_range(0.5..12.0),
            size: rng.gen_range(0.1..5.0),
            aggression: rng.gen_range(0.0..1.0),
            vitality: rng.gen_range(50.0..200.0),
            diet: rng.gen_range(0.0..1.0),
            digestion_efficiency: rng.gen_range(0.4..1.0),
            vision_range: rng.gen_range(40.0..280.0),
            fear_threshold: rng.gen_range(0.5..2.0),
            sociality: rng.gen_range(0.0..1.0),
            reproduction_threshold: rng.gen_range(30.0..300.0),
            offspring_count: rng.gen_range(1.0..4.0),
            reproduction_cooldown: rng.gen_range(60.0..450.0),
            drought_resistance: rng.gen_range(0.0..1.0),
            storm_resistance: rng.gen_range(0.0..1.0),
            aquatic_adaptation: rng.gen_range(0.0..1.0),
            color_r: rng.gen_range(0.1..1.0),
            color_g: rng.gen_range(0.1..1.0),
            color_b: rng.gen_range(0.1..1.0),
            generation: 1,
            species_id: 0,
        }
    }

    /// Скрещивание двух геномов с 50/50 выбором гена от каждого родителя (генотипический кроссовер)
    pub fn crossover(parent1: &Self, parent2: &Self, mutation_rate: f32) -> Self {
        let mut rng = rand::thread_rng();

        // Каждый ген выбирается от одного из родителей случайно (1-точечный кроссовер на уровне генов)
        let mut mix = |a: f32, b: f32| -> f32 {
            if rng.gen_bool(0.5) { a } else { b }
        };

        let mut child = Self {
            metabolism: mix(parent1.metabolism, parent2.metabolism),
            speed: mix(parent1.speed, parent2.speed),
            size: mix(parent1.size, parent2.size),
            aggression: mix(parent1.aggression, parent2.aggression),
            vitality: mix(parent1.vitality, parent2.vitality),
            diet: mix(parent1.diet, parent2.diet),
            digestion_efficiency: mix(parent1.digestion_efficiency, parent2.digestion_efficiency),
            vision_range: mix(parent1.vision_range, parent2.vision_range),
            fear_threshold: mix(parent1.fear_threshold, parent2.fear_threshold),
            sociality: mix(parent1.sociality, parent2.sociality),
            reproduction_threshold: mix(parent1.reproduction_threshold, parent2.reproduction_threshold),
            offspring_count: mix(parent1.offspring_count, parent2.offspring_count),
            reproduction_cooldown: mix(parent1.reproduction_cooldown, parent2.reproduction_cooldown),
            drought_resistance: mix(parent1.drought_resistance, parent2.drought_resistance),
            storm_resistance: mix(parent1.storm_resistance, parent2.storm_resistance),
            aquatic_adaptation: mix(parent1.aquatic_adaptation, parent2.aquatic_adaptation),
            // Цвет — усредняем для плавных переходов
            color_r: (parent1.color_r + parent2.color_r) / 2.0,
            color_g: (parent1.color_g + parent2.color_g) / 2.0,
            color_b: (parent1.color_b + parent2.color_b) / 2.0,
            generation: parent1.generation.max(parent2.generation) + 1,
            species_id: parent1.species_id,
        };

        child.mutate_in_place(mutation_rate);
        child
    }

    /// Применяет случайные мутации ко всем генам
    pub fn mutate_in_place(&mut self, mutation_rate: f32) {
        let mut rng = rand::thread_rng();

        let mut mg = |gene: &mut f32, min: f32, max: f32| {
            if rng.gen_range(0.0f32..1.0) < mutation_rate {
                // Случайная мутация: либо небольшое смещение (80%), либо полный сброс (20%)
                if rng.gen_range(0.0f32..1.0) < 0.8 {
                    let delta = rng.gen_range(-0.15..0.15);
                    *gene = (*gene * (1.0 + delta)).clamp(min, max);
                } else {
                    // Большой прыжок — редкое событие
                    *gene = rng.gen_range(min..max);
                }
            }
        };

        mg(&mut self.metabolism, 0.05, 5.0);
        mg(&mut self.speed, 0.5, 18.0);
        mg(&mut self.size, 0.1, 8.0);
        mg(&mut self.aggression, 0.0, 1.0);
        mg(&mut self.vitality, 50.0, 300.0);
        mg(&mut self.diet, 0.0, 1.0);
        mg(&mut self.digestion_efficiency, 0.4, 1.0);
        mg(&mut self.vision_range, 40.0, 350.0);
        mg(&mut self.fear_threshold, 0.5, 2.0);
        mg(&mut self.sociality, 0.0, 1.0);
        mg(&mut self.reproduction_threshold, 30.0, 400.0);
        mg(&mut self.offspring_count, 1.0, 4.0);
        mg(&mut self.reproduction_cooldown, 60.0, 500.0);
        mg(&mut self.drought_resistance, 0.0, 1.0);
        mg(&mut self.storm_resistance, 0.0, 1.0);
        mg(&mut self.aquatic_adaptation, 0.0, 1.0);
        mg(&mut self.color_r, 0.0, 1.0);
        mg(&mut self.color_g, 0.0, 1.0);
        mg(&mut self.color_b, 0.0, 1.0);
    }

    /// Эвклидово расстояние между двумя геномами (нормированное к 0..1)
    pub fn genetic_distance(&self, other: &Self) -> f32 {
        let diffs = [
            (self.metabolism - other.metabolism) / 5.0,
            (self.speed - other.speed) / 18.0,
            (self.size - other.size) / 8.0,
            self.aggression - other.aggression,
            (self.vitality - other.vitality) / 300.0,
            self.diet - other.diet,
            self.digestion_efficiency - other.digestion_efficiency,
            (self.vision_range - other.vision_range) / 350.0,
            (self.fear_threshold - other.fear_threshold) / 2.0,
            self.sociality - other.sociality,
            (self.reproduction_threshold - other.reproduction_threshold) / 400.0,
            self.drought_resistance - other.drought_resistance,
            self.storm_resistance - other.storm_resistance,
            self.aquatic_adaptation - other.aquatic_adaptation,
            self.color_r - other.color_r,
            self.color_g - other.color_g,
            self.color_b - other.color_b,
        ];
        let sum_sq: f32 = diffs.iter().map(|d| d * d).sum();
        sum_sq.sqrt()
    }

    /// Реальное макс. здоровье, рассчитанное из генома
    pub fn max_health(&self) -> f32 {
        self.vitality
    }
}
