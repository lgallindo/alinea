use std::{
    fs, io,
    path::{Path, PathBuf},
};

use directories::BaseDirs;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub wrap_column: usize,
    pub auto_wrap: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            wrap_column: 80,
            auto_wrap: true,
        }
    }
}

impl Config {
    pub fn load() -> Self {
        let Some(path) = config_path() else {
            return Self::default();
        };

        fs::read_to_string(path)
            .ok()
            .and_then(|contents| toml::from_str(&contents).ok())
            .unwrap_or_default()
    }

    pub fn save(&self) -> io::Result<()> {
        let path = config_path().unwrap_or_else(|| PathBuf::from("config.toml"));
        write_config(self, &path)
    }
}

pub fn config_path() -> Option<PathBuf> {
    BaseDirs::new().map(|dirs| dirs.data_dir().join("Alinea").join("config.toml"))
}

fn write_config(config: &Config, path: &Path) -> io::Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    let contents = toml::to_string_pretty(config).map_err(io::Error::other)?;
    fs::write(path, contents)
}
