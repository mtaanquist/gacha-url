mod genshin;
mod hoyoverse;
mod hsr;
mod wuwa;

pub use genshin::GenshinImpact;
pub use hsr::HonkaiStarRail;
pub use wuwa::WutheringWaves;

use std::path::Path;

use anyhow::Result;

/// Trait describing a gacha game. Each game knows how to identify itself
/// and how to extract a usable URL from its data directory.
pub trait GachaGame {
    /// Human-readable name for display purposes.
    fn name(&self) -> &'static str;

    /// Short identifier used as the key in the config file.
    fn id(&self) -> &'static str;

    /// Given a game directory, extract a usable gacha/convene history URL.
    fn extract_url(&self, game_dir: &Path) -> Result<String>;
}
