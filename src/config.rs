use serde::Deserialize;
use std::fs;

use crate::Error;
use crate::Result;
use crate::APP_NAME;

#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    pub host: String,
    pub port: usize,
    pub cert_path: String,
    pub key_path: String,
    pub secret: String,
    pub hooks: Vec<Hook>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Hook {
    pub id: String,
    pub event: String,
    pub exec: String,
}

impl Config {
    pub fn acquire() -> Result<Self> {
        let mut config_path = match dirs::config_dir() {
            Some(config_path) => config_path,
            None => return Err(Error::new("unable to determine config path!")),
        };
        config_path.push(APP_NAME);
        config_path.push("config.toml");
        let raw = fs::read_to_string(&config_path)?;
        let config: Self = toml::from_str(&raw)?;
        Ok(config)
    }
}
