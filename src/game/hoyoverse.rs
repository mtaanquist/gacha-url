use std::fs;
use std::path::Path;

use anyhow::{bail, Context, Result};
use url::Url;
use walkdir::WalkDir;

/// Locate and extract a valid gacha URL from a HoYoverse game directory.
///
/// These games store URLs in a binary cache file (`data_2`) inside a versioned
/// `webCaches` directory. The URL is validated against the API and stripped
/// down to only the retained query parameters.
pub fn extract_from_cache(
    game_dir: &Path,
    url_patterns: &[&str],
    retained_params: &[&str],
) -> Result<String> {
    let cache_path = find_data_2(game_dir)?;

    let data = fs::read(&cache_path)
        .with_context(|| format!("failed to read {}", cache_path.display()))?;

    let candidates = find_urls(&data, url_patterns);

    if candidates.is_empty() {
        bail!("no gacha URLs found in {}", cache_path.display());
    }

    let client = reqwest::blocking::Client::new();

    for raw in candidates.iter().rev() {
        let Ok(parsed) = Url::parse(raw) else {
            continue;
        };

        if validate(&client, &parsed)? {
            return Ok(strip_params(&parsed, retained_params));
        }
    }

    bail!(
        "found {} candidate URL(s) but none returned a valid response. \
         Make sure to open the warp/gacha history in-game before running this.",
        candidates.len()
    )
}

fn find_data_2(game_dir: &Path) -> Result<std::path::PathBuf> {
    let web_caches = find_web_caches_dir(game_dir).with_context(|| {
        format!(
            "could not locate a webCaches directory under {}",
            game_dir.display()
        )
    })?;

    let mut best_version: u64 = 0;
    let mut best_path = web_caches.join("Cache/Cache_Data/data_2");

    let entries = fs::read_dir(&web_caches)
        .with_context(|| format!("failed to read {}", web_caches.display()))?;

    for entry in entries.filter_map(|e| e.ok()) {
        let name = entry.file_name();
        let name = name.to_string_lossy();

        if !is_version_dir(&name) {
            continue;
        }

        let version: u64 = name.replace('.', "").parse().unwrap_or(0);
        if version >= best_version {
            best_version = version;
            best_path = web_caches
                .join(name.as_ref())
                .join("Cache/Cache_Data/data_2");
        }
    }

    if best_path.is_file() {
        Ok(best_path)
    } else {
        bail!("cache file not found at {}", best_path.display())
    }
}

fn find_web_caches_dir(base: &Path) -> Option<std::path::PathBuf> {
    let direct = base.join("webCaches");
    if direct.is_dir() {
        return Some(direct);
    }

    for entry in WalkDir::new(base)
        .max_depth(4)
        .follow_links(false)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        if entry.file_type().is_dir()
            && entry
                .file_name()
                .to_string_lossy()
                .eq_ignore_ascii_case("webCaches")
        {
            return Some(entry.path().to_owned());
        }
    }

    None
}

/// The cache is a binary file with null-delimited strings preceded by `1/0/`.
fn find_urls(data: &[u8], patterns: &[&str]) -> Vec<String> {
    let delimiter = b"1/0/";
    let mut urls = Vec::new();

    for chunk in data
        .windows(delimiter.len())
        .enumerate()
        .filter_map(|(i, w)| (w == delimiter).then_some(i))
        .map(|i| &data[i + delimiter.len()..])
    {
        if !chunk.starts_with(b"http") {
            continue;
        }

        let end = chunk.iter().position(|&b| b == 0).unwrap_or(chunk.len());
        let segment = &chunk[..end];

        let Ok(text) = std::str::from_utf8(segment) else {
            continue;
        };

        if patterns.iter().any(|p| text.contains(p)) {
            urls.push(text.to_owned());
        }
    }

    urls
}

fn validate(client: &reqwest::blocking::Client, url: &Url) -> Result<bool> {
    let resp: serde_json::Value = client
        .get(url.as_str())
        .header("Content-Type", "application/json")
        .send()
        .context("failed to reach gacha API")?
        .json()
        .context("API returned non-JSON response")?;

    Ok(resp.get("retcode").and_then(|v| v.as_i64()) == Some(0))
}

fn strip_params(url: &Url, retain: &[&str]) -> String {
    let filtered: Vec<(String, String)> = url
        .query_pairs()
        .filter(|(key, _)| retain.contains(&key.as_ref()))
        .map(|(k, v)| (k.into_owned(), v.into_owned()))
        .collect();

    let mut cleaned = url.clone();
    cleaned.set_query(None);

    if !filtered.is_empty() {
        cleaned.query_pairs_mut().extend_pairs(filtered);
    }

    cleaned.into()
}

fn is_version_dir(name: &str) -> bool {
    let parts: Vec<&str> = name.split('.').collect();
    parts.len() == 4 && parts.iter().all(|p| p.chars().all(|c| c.is_ascii_digit()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strip_params_retains_only_specified() {
        let url =
            Url::parse("https://example.com/api?authkey=abc&junk=1&lang=en&sign_type=2").unwrap();

        let result = strip_params(&url, &["authkey", "lang", "sign_type"]);
        let parsed = Url::parse(&result).unwrap();
        let pairs: Vec<_> = parsed.query_pairs().collect();

        assert_eq!(pairs.len(), 3);
        assert!(pairs.iter().any(|(k, v)| k == "authkey" && v == "abc"));
        assert!(pairs.iter().any(|(k, v)| k == "lang" && v == "en"));
        assert!(pairs.iter().any(|(k, v)| k == "sign_type" && v == "2"));
    }

    #[test]
    fn find_urls_extracts_from_binary_blob() {
        let mut data = Vec::new();
        data.extend(b"junk data here");
        data.extend(b"1/0/");
        data.extend(b"https://example.com/api/getGachaLog?authkey=test");
        data.push(0);
        data.extend(b"more junk");
        data.extend(b"1/0/");
        data.extend(b"not a url");
        data.push(0);

        let urls = find_urls(&data, &["getGachaLog"]);
        assert_eq!(urls.len(), 1);
        assert!(urls[0].contains("getGachaLog"));
    }
}
