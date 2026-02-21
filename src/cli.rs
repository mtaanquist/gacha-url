use std::path::PathBuf;

use clap::{Parser, ValueEnum};

use crate::game::{self, GachaGame};

#[derive(Parser)]
#[command(
    name = "gacha-url",
    about = "Extract gacha history URLs from game caches"
)]
pub struct Cli {
    /// Which game to extract the URL for.
    #[arg(short, long)]
    pub game: GameArg,

    /// Optional path to the game install directory. If omitted, the tool will
    /// search game-specific directories automatically.
    #[arg(short, long)]
    pub path: Option<PathBuf>,
}

#[derive(ValueEnum, Clone, Copy)]
pub enum GameArg {
    /// Honkai: Star Rail
    Hsr,
    /// Genshin Impact
    Gi,
    /// Wuthering Waves
    Wuwa,
}

impl GameArg {
    pub fn into_game(self) -> Box<dyn GachaGame> {
        match self {
            Self::Hsr => Box::new(game::HonkaiStarRail),
            Self::Gi => Box::new(game::GenshinImpact),
            Self::Wuwa => Box::new(game::WutheringWaves),
        }
    }
}
