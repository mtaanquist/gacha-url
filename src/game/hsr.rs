use std::path::Path;

use anyhow::Result;

use super::hoyoverse;
use super::GachaGame;

pub struct HonkaiStarRail;

const URL_PATTERNS: &[&str] = &["getGachaLog", "getLdGachaLog"];
const RETAINED_PARAMS: &[&str] = &["authkey", "authkey_ver", "sign_type", "game_biz", "lang"];

impl GachaGame for HonkaiStarRail {
    fn id(&self) -> &'static str {
        "hsr"
    }

    fn extract_url(&self, game_dir: &Path) -> Result<String> {
        hoyoverse::extract_from_cache(game_dir, URL_PATTERNS, RETAINED_PARAMS)
    }

    fn extra_search_dirs(&self) -> Vec<String> {
        let mut dirs = Vec::new();

        if cfg!(target_os = "linux") {
            dirs.extend([
                // Flatpak Steam
                ".var/app/com.valvesoftware.Steam/.local/share/Steam/steamapps/compatdata".into(),
            ]);
        }

        if cfg!(target_os = "windows") {
            dirs.extend([
                r"C:\Program Files (x86)\Steam\steamapps\compatdata".into(),
                r"C:\Program Files\HoYoPlay\games".into(),
                r"D:\Program Files\HoYoPlay\games".into(),
            ]);
        }

        // Steam library folders discovered at runtime
        for lib in crate::steam::discover_library_folders() {
            let compatdata = lib.join("steamapps/compatdata");
            dirs.push(compatdata.to_string_lossy().into_owned());
        }

        dirs
    }
}
