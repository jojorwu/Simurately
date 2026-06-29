pub struct GameSettings {
    pub show_tile_energy: bool,
    pub show_tile_moisture: bool,
    pub show_genome_colors: bool,
    pub show_health_bars: bool,
    pub show_ai_states: bool,
    pub show_species_territory: bool,
    pub show_plants: bool,
    pub show_fish: bool,
    pub show_animals: bool,
    pub show_rules: bool,
}

impl Default for GameSettings {
    fn default() -> Self {
        Self {
            show_tile_energy: false,
            show_tile_moisture: false,
            show_genome_colors: true,
            show_health_bars: true,
            show_ai_states: false,
            show_species_territory: false,
            show_plants: true,
            show_fish: true,
            show_animals: true,
            show_rules: true,
        }
    }
}
