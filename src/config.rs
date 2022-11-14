use serde::Deserialize;
use std::fs;
use std::path::Path;
use std::path::PathBuf;

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
    pub branch: String,
    pub event: String,
    pub exec: String,
}

impl Config {
    pub fn acquire() -> Result<Self> {
        let mut config_file = Self::get_global();
        let local_file = Self::get_local();
        if local_file.is_some() {
            config_file = local_file;
        }
        match config_file {
            Some(file) => {
                let raw = fs::read_to_string(&file)?;
                let config: Self = toml::from_str(&raw)?;
                Ok(config)
            }
            None => Err(Error::new("unable to determine config file!")),
        }
    }
    fn get_local() -> Option<PathBuf> {
        let local_file = Path::new("config.toml").to_path_buf();
        if !local_file.exists() {
            return None;
        }
        Some(local_file)
    }
    fn get_global() -> Option<PathBuf> {
        let mut global_file = match dirs::config_dir() {
            Some(file) => file,
            None => return None,
        };
        global_file.push(APP_NAME);
        global_file.push("config.toml");
        if !global_file.exists() {
            return None;
        }
        Some(global_file)
    }
}
