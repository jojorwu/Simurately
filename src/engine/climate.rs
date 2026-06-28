use serde::{Deserialize, Serialize};
use glam::Vec2;
use crate::engine::events::WorldEvent;
use rand::Rng;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]


pub enum WeatherType {
    Sunny,
    PartlyCloudy,
    Rainy,
    Stormy,
    Drought,
    Blizzard, // Снежная буря: холодно + ветер
    Heatwave, // Жара без осадков
}

impl WeatherType {
    /// Возвращает параметры погоды: (temperature_delta, humidity_delta, sunlight, wind_speed)
    pub fn params(&self) -> WeatherParams {
        match self {
            WeatherType::Sunny =>       WeatherParams { temp_delta: 0.3,  humidity_delta: -0.05, sunlight: 1.0,  wind: 0.1 },
            WeatherType::PartlyCloudy =>WeatherParams { temp_delta: 0.1,  humidity_delta: 0.0,   sunlight: 0.65, wind: 0.2 },
            WeatherType::Rainy =>       WeatherParams { temp_delta: -0.1, humidity_delta: 0.4,   sunlight: 0.3,  wind: 0.3 },
            WeatherType::Stormy =>      WeatherParams { temp_delta: -0.2, humidity_delta: 0.6,   sunlight: 0.1,  wind: 0.9 },
            WeatherType::Drought =>     WeatherParams { temp_delta: 0.5,  humidity_delta: -0.6,  sunlight: 0.9,  wind: 0.15},
            WeatherType::Blizzard =>    WeatherParams { temp_delta: -0.8, humidity_delta: 0.2,   sunlight: 0.05, wind: 0.95},
            WeatherType::Heatwave =>    WeatherParams { temp_delta: 0.9,  humidity_delta: -0.5,  sunlight: 0.95, wind: 0.05},
        }
    }

    pub fn display_name(&self) -> &'static str {
        match self {
            WeatherType::Sunny => "☀ Ясно",
            WeatherType::PartlyCloudy => "⛅ Переменная облачность",
            WeatherType::Rainy => "🌧 Дождь",
            WeatherType::Stormy => "⚡ Гроза",
            WeatherType::Drought => "🔥 Засуха",
            WeatherType::Blizzard => "❄ Буран",
            WeatherType::Heatwave => "🌡 Жаркая волна",
        }
    }
}

#[derive(Clone, Copy)]
pub struct WeatherParams {
    pub temp_delta: f32,
    pub humidity_delta: f32,
    pub sunlight: f32,
    pub wind: f32,
}

/// Планетарная климатическая система — управляет плавной сменой погоды
#[derive(Clone, Serialize, Deserialize)]
pub struct Climate {
    pub current_weather: WeatherType,
    pub next_weather: WeatherType,
    pub transition_progress: f32, // 0.0..1.0
    pub duration_timer: u32,      // Сколько тиков текущая погода
    pub season: u32,              // 0=Весна, 1=Лето, 2=Осень, 3=Зима
    pub season_timer: u32,
    pub base_temperature: f32,    // -1..1 — сезонная температура
    pub global_humidity: f32,     // 0..1 — общая влажность

    // Параметры, актуальные прямо сейчас (интерполированы)
    pub temperature: f32,
    pub humidity: f32,
    pub sunlight: f32,
    pub wind_speed: f32,

    // Катастрофы
    pub flood_level: f32,                      // Уровень паводка (0..1)
    pub lightning_strike: Option<(Vec2, u32)>,
    pub wildfire_pos: Option<Vec2>,
}

impl Climate {
    pub fn new() -> Self {
        Self {
            current_weather: WeatherType::Sunny,
            next_weather: WeatherType::PartlyCloudy,
            transition_progress: 0.0,
            duration_timer: 0,
            season: 0,
            season_timer: 0,
            base_temperature: 0.3, // Весна — тепло
            global_humidity: 0.4,
            temperature: 0.3,
            humidity: 0.4,
            sunlight: 1.0,
            wind_speed: 0.1,
            lightning_strike: None,
            wildfire_pos: None,
            flood_level: 0.0,
        }
    }

    /// Следующий сезон в порядке: Весна → Лето → Осень → Зима
    fn advance_season(&mut self) {
        self.season = (self.season + 1) % 4;
        self.season_timer = 0;
        self.base_temperature = match self.season {
            0 => 0.2,  // Весна — тепло
            1 => 0.7,  // Лето — жарко
            2 => 0.0,  // Осень — прохладно
            _ => -0.6, // Зима — холодно
        };
        self.global_humidity = match self.season {
            0 => 0.6,  // Весна — влажно
            1 => 0.35, // Лето — суховато
            2 => 0.5,  // Осень — умеренно
            _ => 0.3,  // Зима — сухой воздух
        };
    }

    /// Выбирает следующую погоду в зависимости от сезона и случайности
    fn pick_next_weather(&self) -> WeatherType {
        let mut rng = rand::thread_rng();
        let roll = rng.gen_range(0.0f32..1.0);
        match self.season {
            0 => { // Весна
                if roll < 0.30 { WeatherType::Sunny }
                else if roll < 0.55 { WeatherType::PartlyCloudy }
                else if roll < 0.75 { WeatherType::Rainy }
                else if roll < 0.85 { WeatherType::Stormy }
                else { WeatherType::PartlyCloudy }
            }
            1 => { // Лето
                if roll < 0.35 { WeatherType::Sunny }
                else if roll < 0.55 { WeatherType::Heatwave }
                else if roll < 0.70 { WeatherType::Drought }
                else if roll < 0.80 { WeatherType::Stormy }
                else { WeatherType::PartlyCloudy }
            }
            2 => { // Осень
                if roll < 0.25 { WeatherType::Rainy }
                else if roll < 0.45 { WeatherType::PartlyCloudy }
                else if roll < 0.65 { WeatherType::Stormy }
                else if roll < 0.80 { WeatherType::Sunny }
                else { WeatherType::Rainy }
            }
            _ => { // Зима
                if roll < 0.35 { WeatherType::Blizzard }
                else if roll < 0.55 { WeatherType::PartlyCloudy }
                else if roll < 0.70 { WeatherType::Sunny }
                else if roll < 0.85 { WeatherType::Drought }
                else { WeatherType::Blizzard }
            }
        }
    }

    /// Обновление климата в тик
    pub fn tick(&mut self, tick: u64) -> Vec<WorldEvent> {
        let mut events = Vec::new();
        self.duration_timer += 1;
        self.season_timer += 1;
        
        // Смена сезона каждые 3000 тиков
        if self.season_timer >= 3000 {
            self.advance_season();
            let season_names = ["🌱 Весна", "☀ Лето", "🍂 Осень", "❄ Зима"];
            events.push(WorldEvent::SeasonChanged(season_names[self.season as usize].to_string()));
        }
        
        // Погодный переход
        self.transition_progress += 0.005; // ~200 тиков на переход
        if self.transition_progress >= 1.0 {
            self.transition_progress = 1.0;
            self.current_weather = self.next_weather;
        }
        
        // Смена погоды через случайное время (300–800 тиков)
        let weather_duration = rand::thread_rng().gen_range(300..=800);
        if self.duration_timer >= weather_duration && self.transition_progress >= 1.0 {
            self.duration_timer = 0;
            let old = self.current_weather;
            self.next_weather = self.pick_next_weather();
            self.transition_progress = 0.0;
            
            if old != self.next_weather {
                events.push(WorldEvent::WeatherChanged(format!("{} → {}", old.display_name(), self.next_weather.display_name())));
            }
        }
        
        // Интерполяция параметров погоды
        let cp = self.current_weather.params();
        let np = self.next_weather.params();
        let t = self.transition_progress;
        let lerp = |a: f32, b: f32| a + (b - a) * t;
        
        self.temperature = self.base_temperature + lerp(cp.temp_delta, np.temp_delta);
        self.humidity = (self.global_humidity + lerp(cp.humidity_delta, np.humidity_delta)).clamp(0.0, 1.0);
        self.sunlight = lerp(cp.sunlight, np.sunlight);
        self.wind_speed = lerp(cp.wind, np.wind);
        
        // Молнии при шторме
        if self.current_weather == WeatherType::Stormy && rand::thread_rng().gen_range(0.0f32..1.0) < 0.015 {
            // Теперь World решит, где именно ударит молния
            events.push(WorldEvent::LightningStrike(Vec2::new(0.0, 0.0))); // Заглушка, World заменит
        }
        
        // Паводок при долгом дожде
        if matches!(self.current_weather, WeatherType::Rainy | WeatherType::Stormy) {
            self.flood_level = (self.flood_level + 0.0005).min(0.3);
        } else {
            self.flood_level = (self.flood_level - 0.002).max(0.0);
        }
        
        if let Some((_, ref mut age)) = self.lightning_strike {
            if *age > 0 {
                *age = age.saturating_sub(1);
            }
        }
        if self.lightning_strike.as_ref().map_or(false, |(_, age)| *age == 0) {
            self.lightning_strike = None;
        }

        events
    }
}
