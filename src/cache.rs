use std::path::PathBuf;

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
        let searched: Vec<String> = search_dirs
            .iter()
            .map(|d| format!("  {}", d.display()))
            .collect();
        bail!(
            "could not find any directories matching {} hints.\n\
             Searched in:\n{}\n\
             To add a search path, run: gacha-url -g {} -a <path>",
            game_config.name,
            searched.join("\n"),
            game.id()
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
            "checked {} candidate directories for {} but extraction failed in all of them.\n\
             To add a search path, run: gacha-url -g {} -a <path>",
            candidates.len(),
            game_config.name,
            game.id()
        )
    }))
}
