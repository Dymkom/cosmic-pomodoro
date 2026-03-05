use cosmic::cosmic_config::{self, cosmic_config_derive::CosmicConfigEntry, CosmicConfigEntry};

#[derive(Debug, Clone, PartialEq, Eq, CosmicConfigEntry)]
#[version = 1]
pub struct Config {
    pub long_break_interval: u64,
    pub work_time: u64,
    pub short_break_time: u64,
    pub long_break_time: u64,
    pub auto_start_work: bool,
    pub auto_start_break: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            long_break_interval: 4,
            work_time: 25,
            short_break_time: 5,
            long_break_time: 15,
            auto_start_work: true,
            auto_start_break: false,
        }
    }
}

impl Config {
    pub const VERSION: u64 = 1;
}