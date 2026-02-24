# gacha-url

A command-line tool that extracts gacha/convene history URLs from game caches. The URL is printed to stdout and copied to the clipboard.

## Supported games

| Game | Flag |
|---|---|
| Arknights: Endfield | `endfield` |
| Genshin Impact | `genshin` |
| Honkai: Star Rail | `hsr` |
| Wuthering Waves | `wuwa` |
| Zenless Zone Zero | `zzz` |

## Installation

```sh
cargo install --path .
```

## Usage

```sh
gacha-url -g wuwa
```

The tool searches preconfigured directories for the game's data, extracts the most recent valid URL, prints it, and copies it to the clipboard.

Make sure to open the gacha/warp/convene history screen in-game before running the tool, as the URL is only present in the cache while the session is active.

## Configuration

On first run, a default configuration file is created automatically at the platform-specific config directory:

- **Linux**: `~/.config/gacha-url/config.toml`
- **Windows**: `%APPDATA%\gacha-url\config.toml`
- **macOS**: `~/Library/Application Support/gacha-url/config.toml`

The file defines where to search for each game's data. The default config ships with common Linux paths (Steam, community launchers). You will likely need to adjust these for your setup, especially on Windows.

### Config format

```toml
[wuwa]
name = "Wuthering Waves"
search_dirs = [
    ".local/share/Steam/steamapps/common/Wuthering Waves",
    ".steam/steam/steamapps/common/Wuthering Waves",
]
path_hints = ["Wuthering Waves", "WutheringWaves", "wuwa"]
```

- **name** -- Display name used in log messages.
- **search_dirs** -- Directories to search. Relative paths are resolved against the home directory. Absolute paths are used as-is.
- **path_hints** -- Substrings matched (case-insensitively) against directory names to identify relevant game directories within the search paths.

### Adding a search path

Rather than editing the config file by hand, you can add a path from the command line:

```sh
gacha-url -g wuwa -a /path/to/game
```

This appends the path to `search_dirs` for the given game and preserves any existing formatting in the config file.

### Resetting the config

If you need a fresh config file, delete the existing one and run the tool again. A new default will be created automatically.

## How it works

### HoYoverse games (Genshin Impact, Honkai: Star Rail, Zenless Zone Zero)

These games store API URLs in a binary cache file (`webCaches/*/Cache/Cache_Data/data_2`). The tool finds the highest-versioned cache directory, scans for URLs matching known API patterns, and validates each against the live API until one returns a successful response. The URL is then stripped down to only the essential query parameters.

### Arknights: Endfield

Endfield logs the gacha history URL in its SDK webview log file (`HGWebview.log`). The tool scans the log and returns the most recent URL found.

### Wuthering Waves

Wuthering Waves logs the convene history URL in its client log files. The tool searches known log paths and returns the most recent URL found.

Note: logging must be enabled in the game. If `Client/Saved/Config/WindowsNoEditor/Engine.ini` contains `Global=off` or `Global=none` under `[Core.Log]`, the tool will not be able to find the URL. Remove or change that line to re-enable logging.
