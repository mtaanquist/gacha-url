mod cache;
mod cli;
mod config;
mod game;

use std::path::PathBuf;

use anyhow::Result;
use clap::Parser;

fn main() -> Result<()> {
    let args = cli::Cli::parse();
    let game = args.game.into_game();

    let home = home_dir()?;
    let config = config::Config::load()?;
    let game_config = config.game_config(game.id())?;

    if let Some(ref new_path) = args.add_path {
        let path_str = new_path.to_string_lossy();
        config::add_search_dir(game.id(), &path_str)?;
        eprintln!(
            "Added '{}' to search_dirs for {} in {}",
            path_str,
            game_config.name,
            config::config_path().display()
        );
        return Ok(());
    }

    eprintln!("Searching for {} ...", game_config.name);

    let dirs = config.search_dirs_for(game.id(), &home)?;
    let url = cache::auto_detect(game.as_ref(), game_config, &dirs)?;

    println!("{url}");

    match copy_to_clipboard(&url) {
        Ok(()) => eprintln!("URL copied to clipboard."),
        Err(e) => eprintln!("Could not copy to clipboard: {e}"),
    }

    Ok(())
}

fn copy_to_clipboard(text: &str) -> Result<()> {
    let mut clipboard = arboard::Clipboard::new()?;
    clipboard.set_text(text)?;
    Ok(())
}

fn home_dir() -> Result<PathBuf> {
    dirs::home_dir().ok_or_else(|| anyhow::anyhow!("could not determine home directory"))
}
