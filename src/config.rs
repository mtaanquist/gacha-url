use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use serde::Deserialize;

#[derive(Deserialize)]
pub struct GameConfig {
    pub name: String,
    pub search_dirs: Vec<String>,
    pub path_hints: Vec<String>,
}

pub struct Config {
    games: HashMap<String, GameConfig>,
}

impl Config {
    pub fn load() -> Result<Self> {
        let path = config_path();

        let contents = fs::read_to_string(&path)
            .with_context(|| format!("could not read config file at {}", path.display()))?;

        let games: HashMap<String, GameConfig> = toml::from_str(&contents)
            .with_context(|| format!("failed to parse config file at {}", path.display()))?;

        Ok(Self { games })
    }

    pub fn game_config(&self, id: &str) -> Result<&GameConfig> {
        self.games.get(id).ok_or_else(|| {
            anyhow::anyhow!(
                "no configuration found for game '{}' in config file",
                id
            )
        })
    }

    pub fn search_dirs_for(&self, id: &str, home: &Path) -> Result<Vec<PathBuf>> {
        let gc = self.game_config(id)?;

        let dirs = gc
            .search_dirs
            .iter()
            .map(|d| {
                let p = PathBuf::from(d);
                if p.is_absolute() {
                    p
                } else {
                    home.join(p)
                }
            })
            .filter(|p| p.is_dir())
            .collect();

        Ok(dirs)
    }
}

impl GameConfig {
    /// Returns `true` if `path` looks like it belongs to this game.
    pub fn matches_path(&self, path: &Path) -> bool {
        let lossy = path.to_string_lossy();
        let lower = lossy.to_ascii_lowercase();
        self.path_hints
            .iter()
            .any(|hint| lower.contains(&hint.to_ascii_lowercase()))
    }
}

fn config_path() -> PathBuf {
    dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("gacha-url/config.toml")
}
