mod genshin;
mod hoyoverse;
mod hsr;
mod wuwa;

pub use genshin::GenshinImpact;
pub use hsr::HonkaiStarRail;
pub use wuwa::WutheringWaves;

use std::path::Path;

use anyhow::Result;

pub trait GachaGame {
    fn id(&self) -> &'static str;
    fn extract_url(&self, game_dir: &Path) -> Result<String>;

    /// Additional search directories discovered at runtime (e.g. platform-specific
    /// paths, Steam library folders). Merged with the paths from config.toml.
    fn extra_search_dirs(&self) -> Vec<String> {
        vec![]
    }
}
