use std::fs;
use std::path::Path;
use std::sync::LazyLock;

use anyhow::{bail, Result};
use regex::Regex;

use super::GachaGame;

static URL_PATTERN: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r#"https://[^\s"]+?\.gryphline\.com/[^\s"]+?token[^\s"]+?server[^\s"]+"#)
        .expect("hardcoded regex must be valid")
});

pub struct Endfield;

const LOG_PATHS: &[&str] = &["sdklogs/HGWebview.log"];

impl GachaGame for Endfield {
    fn id(&self) -> &'static str {
        "endfield"
    }

    fn extract_url(&self, game_dir: &Path) -> Result<String> {
        extract_from_logs(game_dir)
    }

    fn extra_search_dirs(&self) -> Vec<String> {
        let mut dirs = Vec::new();

        if cfg!(target_os = "windows") {
            if let Some(profile) = std::env::var_os("USERPROFILE") {
                let locallow = Path::new(&profile).join(r"AppData\LocalLow");
                dirs.push(locallow.join("Gryphline").to_string_lossy().into_owned());
                dirs.push(locallow.join("Hypergryph").to_string_lossy().into_owned());
            }
        }

        dirs
    }
}

fn extract_from_logs(game_dir: &Path) -> Result<String> {
    for log_rel in LOG_PATHS {
        let log_path = game_dir.join(log_rel);

        let contents = match fs::read_to_string(&log_path) {
            Ok(c) => c,
            Err(_) => continue,
        };

        // Take the last match -- most recent URL.
        if let Some(m) = URL_PATTERN.find_iter(&contents).last() {
            return Ok(m.as_str().to_owned());
        }
    }

    bail!(
        "no gacha history URL found in log files under {}. \
         Make sure to open the gacha history in-game before running this.",
        game_dir.display()
    )
}

#[cfg(test)]
mod tests {
    use std::fs;

    use super::*;

    const SAMPLE_URL: &str = "https://gachalog.gryphline.com/api/getGachaLog?token=abc123&server=prod-official";

    fn write_log(dir: &std::path::Path, rel: &str, contents: &str) {
        let path = dir.join(rel);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, contents).unwrap();
    }

    // -- URL_PATTERN --

    #[test]
    fn url_pattern_matches_sample() {
        assert!(URL_PATTERN.is_match(SAMPLE_URL));
    }

    #[test]
    fn url_pattern_does_not_match_unrelated_url() {
        assert!(!URL_PATTERN.is_match("https://example.com/gacha?token=a&server=b"));
        assert!(!URL_PATTERN.is_match("https://gryphline.com/no-token-here"));
    }

    // -- extract_from_logs --

    #[test]
    fn extract_from_logs_finds_url() {
        let tmp = tempfile::tempdir().unwrap();
        write_log(
            tmp.path(),
            "sdklogs/HGWebview.log",
            &format!("some log line\n{SAMPLE_URL}\nmore log"),
        );

        let result = extract_from_logs(tmp.path()).unwrap();
        assert_eq!(result, SAMPLE_URL);
    }

    #[test]
    fn extract_from_logs_returns_last_url() {
        let tmp = tempfile::tempdir().unwrap();
        let second_url = "https://gachalog.gryphline.com/api/getGachaLog?token=newer456&server=prod-official";
        write_log(
            tmp.path(),
            "sdklogs/HGWebview.log",
            &format!("{SAMPLE_URL}\nsome middle line\n{second_url}\nend"),
        );

        let result = extract_from_logs(tmp.path()).unwrap();
        assert_eq!(result, second_url);
    }

    #[test]
    fn extract_from_logs_errors_when_no_url_found() {
        let tmp = tempfile::tempdir().unwrap();
        write_log(
            tmp.path(),
            "sdklogs/HGWebview.log",
            "nothing useful here",
        );

        assert!(extract_from_logs(tmp.path()).is_err());
    }

    #[test]
    fn extract_from_logs_errors_when_no_log_files() {
        let tmp = tempfile::tempdir().unwrap();
        assert!(extract_from_logs(tmp.path()).is_err());
    }
}
