use std::path::{Path, PathBuf};

use anyhow::{bail, Result};
use walkdir::WalkDir;

use crate::config::GameConfig;
use crate::game::GachaGame;

pub fn auto_detect(
    game: &dyn GachaGame,
    game_config: &GameConfig,
    search_dirs: &[PathBuf],
) -> Result<String> {
    let mut candidates: Vec<PathBuf> = Vec::new();

    for dir in search_dirs {
        if !dir.is_dir() {
            continue;
        }

        if game_config.matches_path(dir) {
            candidates.push(dir.clone());
        }

        let walker = WalkDir::new(dir)
            .follow_links(true)
            .into_iter()
            .filter_entry(|e| e.file_name().to_string_lossy() != "dosdevices");

        for entry in walker.filter_map(|e| e.ok()) {
            if !entry.file_type().is_dir() {
                continue;
            }

            let path = entry.path();

            if path == dir.as_path() {
                continue;
            }

            if game_config.matches_path(path) {
                candidates.push(path.to_owned());
            }
        }
    }

    if candidates.is_empty() {
        bail!(
            "could not find any directories matching {} hints. \
             Try passing the game path manually with --path.",
            game_config.name
        );
    }

    let mut last_err = None;
    for candidate in &candidates {
        eprintln!("  trying: {}", candidate.display());
        match game.extract_url(candidate) {
            Ok(url) => return Ok(url),
            Err(e) => last_err = Some(e),
        }
    }

    Err(last_err.unwrap_or_else(|| {
        anyhow::anyhow!(
            "checked {} candidate directories for {} but extraction failed in all of them.",
            candidates.len(),
            game_config.name
        )
    }))
}

pub fn from_path(game: &dyn GachaGame, path: &Path) -> Result<String> {
    game.extract_url(path)
}
