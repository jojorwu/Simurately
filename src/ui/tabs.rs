use crate::ui::tabs_sub;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BottomTab {
    Inspector,
    Species,
    Populations,
    Events,
    Climate,
    Settings,
}

pub use tabs_sub::inspector::draw_inspector;
pub use tabs_sub::stats_tabs::{draw_species_tab, draw_populations_tab, draw_climate_tab, draw_events_tab};
pub use tabs_sub::settings_tab::draw_settings_tab;
