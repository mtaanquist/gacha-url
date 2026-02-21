use std::fs;
use std::path::Path;

use anyhow::{bail, Result};
use regex::Regex;

use super::GachaGame;

pub struct WutheringWaves;

/// Relative paths from the game root to the log files that may contain the
/// convene history URL, in order of preference.
const LOG_PATHS: &[&str] = &[
    "Client/Saved/Logs/Client.log",
    "Client/Binaries/Win64/ThirdParty/KrPcSdk_Global/KRSDKRes/KRSDKWebView/debug.log",
];

impl GachaGame for WutheringWaves {
    fn name(&self) -> &'static str {
        "Wuthering Waves"
    }

    fn id(&self) -> &'static str {
        "wuwa"
    }

fn extract_url(&self, game_dir: &Path) -> Result<String> {
        extract_from_logs(game_dir)
    }
}

/// Search the known log files for a convene history URL.
fn extract_from_logs(game_dir: &Path) -> Result<String> {
    let pattern = Regex::new(
        r#"https://aki-gm-resources(?:-oversea)?\.aki-game\.(?:net|com)/aki/gacha/index\.html#/record[^\s"]*"#,
    )?;

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
            if let Some(m) = pattern.find_iter(&contents).last() {
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
