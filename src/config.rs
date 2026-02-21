use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{bail, Context, Result};
use serde::Deserialize;
use toml_edit::DocumentMut;

#[derive(Deserialize)]
pub struct GameConfig {
    pub name: String,
    pub search_dirs: Vec<String>,
    pub path_hints: Vec<String>,
}

const DEFAULT_CONFIG: &str = include_str!("../config.toml");

pub struct Config {
    games: HashMap<String, GameConfig>,
}

impl Config {
    pub fn load() -> Result<Self> {
        let path = config_path();

        if !path.exists() {
            if let Some(parent) = path.parent() {
                fs::create_dir_all(parent)
                    .with_context(|| format!("failed to create {}", parent.display()))?;
            }
            fs::write(&path, DEFAULT_CONFIG)
                .with_context(|| format!("failed to write default config to {}", path.display()))?;
            eprintln!("Created default config at {}", path.display());
        }

        let contents = fs::read_to_string(&path)
            .with_context(|| format!("could not read config file at {}", path.display()))?;

        let games: HashMap<String, GameConfig> = toml::from_str(&contents)
            .with_context(|| format!("failed to parse config file at {}", path.display()))?;

        Ok(Self { games })
    }

    pub fn game_config(&self, id: &str) -> Result<&GameConfig> {
        self.games.get(id).ok_or_else(|| {
            anyhow::anyhow!("no configuration found for game '{}' in config file", id)
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
    pub fn matches_path(&self, path: &Path) -> bool {
        let lossy = path.to_string_lossy();
        let lower = lossy.to_ascii_lowercase();
        self.path_hints
            .iter()
            .any(|hint| lower.contains(&hint.to_ascii_lowercase()))
    }
}

pub fn config_path() -> PathBuf {
    dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("gacha-url/config.toml")
}

pub fn add_search_dir(game_id: &str, path: &str) -> Result<()> {
    let config_path = config_path();

    let contents = fs::read_to_string(&config_path)
        .with_context(|| format!("could not read config file at {}", config_path.display()))?;

    let mut doc: DocumentMut = contents
        .parse()
        .with_context(|| format!("failed to parse config file at {}", config_path.display()))?;

    let game_table = doc
        .get_mut(game_id)
        .and_then(|v| v.as_table_like_mut())
        .ok_or_else(|| anyhow::anyhow!("no configuration found for game '{game_id}'"))?;

    let search_dirs = game_table
        .get_mut("search_dirs")
        .and_then(|v| v.as_array_mut())
        .ok_or_else(|| {
            anyhow::anyhow!("'search_dirs' is missing or not an array for game '{game_id}'")
        })?;

    if search_dirs.iter().any(|v| v.as_str() == Some(path)) {
        bail!("'{}' is already in search_dirs for '{}'", path, game_id);
    }

    search_dirs.push(path);

    fs::write(&config_path, doc.to_string())
        .with_context(|| format!("failed to write config file at {}", config_path.display()))?;

    Ok(())
}
