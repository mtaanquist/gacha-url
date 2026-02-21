use std::path::Path;

use anyhow::Result;

use super::hoyoverse;
use super::GachaGame;

pub struct GenshinImpact;

const URL_PATTERNS: &[&str] = &["getGachaLog"];
const RETAINED_PARAMS: &[&str] = &["authkey", "authkey_ver", "sign_type", "game_biz", "lang"];

impl GachaGame for GenshinImpact {
    fn id(&self) -> &'static str {
        "gi"
    }

    fn extract_url(&self, game_dir: &Path) -> Result<String> {
        hoyoverse::extract_from_cache(game_dir, URL_PATTERNS, RETAINED_PARAMS)
    }
}
