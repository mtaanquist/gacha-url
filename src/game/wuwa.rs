use std::fs;
use std::path::Path;
use std::sync::LazyLock;

use anyhow::{bail, Result};
use regex::Regex;

use super::GachaGame;

static URL_PATTERN: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r#"https://aki-gm-resources(?:-oversea)?\.aki-game\.(?:net|com)/aki/gacha/index\.html#/record[^\s"]*"#,
    )
    .expect("hardcoded regex must be valid")
});

pub struct WutheringWaves;

const LOG_PATHS: &[&str] = &[
    "Client/Saved/Logs/Client.log",
    "Client/Binaries/Win64/ThirdParty/KrPcSdk_Global/KRSDKRes/KRSDKWebView/debug.log",
];

impl GachaGame for WutheringWaves {
    fn id(&self) -> &'static str {
        "wuwa"
    }

    fn extract_url(&self, game_dir: &Path) -> Result<String> {
        extract_from_logs(game_dir)
    }
}

fn extract_from_logs(game_dir: &Path) -> Result<String> {
    // Also try the "Wuthering Waves Game" subdirectory, as some installs
    // nest the actual game data one level deeper.
    let nested = game_dir.join("Wuthering Waves Game");
    let mut roots = vec![game_dir.to_owned()];
    if nested.is_dir() {
        roots.push(nested);
    }

    for root in &roots {
        for log_rel in LOG_PATHS {
            let log_path = root.join(log_rel);

            let contents = match fs::read_to_string(&log_path) {
                Ok(c) => c,
                Err(_) => continue,
            };

            // Take the last match -- most recent URL.
            if let Some(m) = URL_PATTERN.find_iter(&contents).last() {
                return Ok(m.as_str().to_owned());
            }
        }
    }

    bail!(
        "no convene history URL found in log files under {}. \
         Make sure to open the Convene History in-game before running this.",
        game_dir.display()
    )
}
