use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub enum TileType {
    Soil,
    Water,
    Sand,
    Rock, // Новый тип: камень. Не питает растения, замедляет всех.
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Tile {
    pub tile_type: TileType,
    pub energy: f32,     // Питательность почвы
    pub moisture: f32,   // Влажность (0..1)
    pub temperature: f32, // Температура (–1..1)
}
