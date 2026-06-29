use crate::engine::config::MAX_LOG_ENTRIES;

pub struct EventLog {
    pub entries: Vec<String>,
}

impl Default for EventLog {
    fn default() -> Self {
        Self::new()
    }
}

impl EventLog {
    pub fn new() -> Self {
        Self { entries: Vec::new() }
    }

    pub fn push(&mut self, msg: String) {
        self.entries.push(msg);
        if self.entries.len() > MAX_LOG_ENTRIES {
            self.entries.drain(0..20);
        }
    }
}

pub struct StatsManager {
    pub plant_count_history: Vec<f32>,
    pub insect_count_history: Vec<f32>,
    pub fish_count_history: Vec<f32>,
    pub biodiversity_history: Vec<f32>,
    pub total_deaths: u64,
    pub total_births: u64,
    pub total_speciations: u64,
}

impl Default for StatsManager {
    fn default() -> Self {
        Self::new()
    }
}

impl StatsManager {
    pub fn new() -> Self {
        Self {
            plant_count_history: Vec::new(),
            insect_count_history: Vec::new(),
            fish_count_history: Vec::new(),
            biodiversity_history: Vec::new(),
            total_deaths: 0,
            total_births: 0,
            total_speciations: 0,
        }
    }

    pub fn record_history(&mut self, plants: usize, insects: usize, fish: usize, biodiversity: usize) {
        self.plant_count_history.push(plants as f32);
        self.insect_count_history.push(insects as f32);
        self.fish_count_history.push(fish as f32);
        self.biodiversity_history.push(biodiversity as f32);

        const MAX_HISTORY: usize = 600;
        if self.plant_count_history.len() > MAX_HISTORY {
            let excess = self.plant_count_history.len() - MAX_HISTORY;
            self.plant_count_history.drain(0..excess);
            self.insect_count_history.drain(0..excess);
            self.fish_count_history.drain(0..excess);
            self.biodiversity_history.drain(0..excess);
        }
    }
}
