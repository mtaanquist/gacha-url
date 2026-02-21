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

    eprintln!("Searching for {} ...", game.name());

    let url = match args.path {
        Some(ref p) => cache::from_path(game.as_ref(), p)?,
        None => {
            let home = home_dir()?;
            let config = config::Config::load()?;
            let game_config = config.game_config(game.id())?;
            let dirs = config.search_dirs_for(game.id(), &home)?;
            cache::auto_detect(game.as_ref(), game_config, &dirs)?
        }
    };

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
