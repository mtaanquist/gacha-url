use std::fs;
use std::path::PathBuf;

/// Discover Steam library folders by parsing `libraryfolders.vdf`.
///
/// Returns all library root paths found (e.g. `/home/user/.local/share/Steam`,
/// `/mnt/games/SteamLibrary`). Returns an empty vec on any failure.
pub fn discover_library_folders() -> Vec<PathBuf> {
    for vdf_path in vdf_paths() {
        if let Ok(contents) = fs::read_to_string(&vdf_path) {
            let folders = parse_library_paths(&contents);
            if !folders.is_empty() {
                return folders;
            }
        }
    }
    Vec::new()
}

fn vdf_paths() -> Vec<PathBuf> {
    let mut paths = Vec::new();

    if let Some(home) = dirs::home_dir() {
        paths.push(home.join(".steam/steam/config/libraryfolders.vdf"));
        paths.push(home.join(".local/share/Steam/config/libraryfolders.vdf"));
        // Flatpak Steam
        paths.push(
            home.join(".var/app/com.valvesoftware.Steam/.steam/steam/config/libraryfolders.vdf"),
        );
        paths.push(
            home.join(".var/app/com.valvesoftware.Steam/.local/share/Steam/config/libraryfolders.vdf"),
        );
    }

    if cfg!(target_os = "windows") {
        paths.push(PathBuf::from(
            r"C:\Program Files (x86)\Steam\config\libraryfolders.vdf",
        ));
    }

    paths
}

/// Minimal VDF parser that extracts `"path"` values from `libraryfolders.vdf`.
fn parse_library_paths(contents: &str) -> Vec<PathBuf> {
    let mut paths = Vec::new();

    for line in contents.lines() {
        let trimmed = line.trim();
        // Lines look like: "path"		"/some/path"
        // Splitting on `"` gives: ["", "path", "\t\t", "/some/path", ""]
        let parts: Vec<&str> = trimmed.split('"').collect();
        if parts.len() >= 4 && parts[1] == "path" {
            paths.push(PathBuf::from(parts[3]));
        }
    }

    paths
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_library_paths_extracts_paths() {
        let vdf = r#"
"libraryfolders"
{
    "0"
    {
        "path"		"/home/user/.local/share/Steam"
        "label"		""
    }
    "1"
    {
        "path"		"/mnt/games/SteamLibrary"
        "label"		"Games"
    }
}
"#;
        let paths = parse_library_paths(vdf);
        assert_eq!(paths.len(), 2);
        assert_eq!(paths[0], PathBuf::from("/home/user/.local/share/Steam"));
        assert_eq!(paths[1], PathBuf::from("/mnt/games/SteamLibrary"));
    }

    #[test]
    fn parse_library_paths_empty_on_invalid_input() {
        assert!(parse_library_paths("").is_empty());
        assert!(parse_library_paths("not a vdf file").is_empty());
    }

    #[test]
    fn parse_library_paths_handles_windows_paths() {
        let vdf = r#"
"libraryfolders"
{
    "0"
    {
        "path"		"C:\Program Files (x86)\Steam"
    }
    "1"
    {
        "path"		"D:\SteamLibrary"
    }
}
"#;
        let paths = parse_library_paths(vdf);
        assert_eq!(paths.len(), 2);
        assert_eq!(
            paths[0],
            PathBuf::from(r"C:\Program Files (x86)\Steam")
        );
        assert_eq!(paths[1], PathBuf::from(r"D:\SteamLibrary"));
    }
}
