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

#[cfg(test)]
mod tests {
    use std::fs;

    use super::*;

    const NON_OVERSEA_URL: &str = "https://aki-gm-resources.aki-game.net/aki/gacha/index.html#/record?svr_id=76402e5b20be2c39f095a152090afddc&player_id=123&lang=en&gacha_id=1&gacha_type=1&svr_api=https://gmserver-api.aki-game2.net&authkey=AAAA&version=2";
    const OVERSEA_URL: &str = "https://aki-gm-resources-oversea.aki-game.com/aki/gacha/index.html#/record?svr_id=76402e5b20be2c39f095a152090afddc&player_id=456&lang=en";

    // -- URL_PATTERN --

    #[test]
    fn url_pattern_matches_non_oversea() {
        assert!(URL_PATTERN.is_match(NON_OVERSEA_URL));
    }

    #[test]
    fn url_pattern_matches_oversea() {
        assert!(URL_PATTERN.is_match(OVERSEA_URL));
    }

    #[test]
    fn url_pattern_does_not_match_unrelated_url() {
        assert!(!URL_PATTERN.is_match("https://example.com/aki/gacha/index.html#/record"));
        assert!(!URL_PATTERN.is_match("https://aki-gm-resources.aki-game.net/other/path"));
    }

    // -- extract_from_logs --

    fn write_log(dir: &std::path::Path, rel: &str, contents: &str) {
        let path = dir.join(rel);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, contents).unwrap();
    }

    #[test]
    fn extract_from_logs_finds_url_in_client_log() {
        let tmp = tempfile::tempdir().unwrap();
        write_log(
            tmp.path(),
            "Client/Saved/Logs/Client.log",
            &format!("some log line\n{NON_OVERSEA_URL}\nmore log"),
        );

        let result = extract_from_logs(tmp.path()).unwrap();
        assert_eq!(result, NON_OVERSEA_URL);
    }

    #[test]
    fn extract_from_logs_returns_last_url() {
        let tmp = tempfile::tempdir().unwrap();
        write_log(
            tmp.path(),
            "Client/Saved/Logs/Client.log",
            &format!("{NON_OVERSEA_URL}\nsome middle line\n{OVERSEA_URL}\nend"),
        );

        let result = extract_from_logs(tmp.path()).unwrap();
        assert_eq!(result, OVERSEA_URL);
    }

    #[test]
    fn extract_from_logs_errors_when_no_url_found() {
        let tmp = tempfile::tempdir().unwrap();
        write_log(
            tmp.path(),
            "Client/Saved/Logs/Client.log",
            "nothing useful here",
        );

        assert!(extract_from_logs(tmp.path()).is_err());
    }

    #[test]
    fn extract_from_logs_errors_when_no_log_files() {
        let tmp = tempfile::tempdir().unwrap();
        assert!(extract_from_logs(tmp.path()).is_err());
    }

    #[test]
    fn extract_from_logs_searches_nested_subdirectory() {
        let tmp = tempfile::tempdir().unwrap();
        let nested = tmp.path().join("Wuthering Waves Game");
        write_log(
            &nested,
            "Client/Saved/Logs/Client.log",
            &format!("log entry: {NON_OVERSEA_URL}"),
        );

        let result = extract_from_logs(tmp.path()).unwrap();
        assert_eq!(result, NON_OVERSEA_URL);
    }
}
