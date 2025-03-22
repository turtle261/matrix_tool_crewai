use serde::Deserialize;
use std::fs;

#[derive(Clone, Deserialize)]
pub struct Config {
    pub homeserver: HomeserverConfig,
}

#[derive(Clone, Deserialize)]
pub struct HomeserverConfig {
    pub url: String,
}

impl Config {
    pub fn from_file(path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let config_str = fs::read_to_string(path)?;
        let config = toml::from_str(&config_str)?;
        Ok(config)
    }
}