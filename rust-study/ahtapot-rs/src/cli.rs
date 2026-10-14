use crate::aht::AhtError;
use clap::Parser;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

#[derive(Parser, Debug)]
#[command(
    version = "0.1.0",
    about = "A CLI tool for bulk image renaming, resizing, and format conversion.",
    long_about = "A CLI tool for bulk image renaming, resizing, and format conversion."
)]
pub struct Args {
    #[arg(short, long)]
    pub path: String,

    #[arg(short, long)]
    pub name: String,

    #[arg(short, long)]
    pub width: u32,

    #[arg(short = 'e', long)]
    pub height: u32,

    #[arg(short, long)]
    pub output: PathBuf,
}

impl Args {
    pub fn new() -> Result<Self, AhtError> {
        let args = Self::try_parse()?;

        Ok(args)
    }

    pub fn get_dir(&self) -> Result<Vec<PathBuf>, AhtError> {
        let dir = WalkDir::new(Path::new(&self.path))
            .into_iter()
            .map(|res| res.map(|e| e.into_path()))
            .collect::<Result<Vec<_>, walkdir::Error>>()?;

        Ok(dir)
    }
}
