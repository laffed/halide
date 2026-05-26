use anyhow::Result;
use inquire::{Confirm, Text};
use std::path::PathBuf;

use crate::config::{self, Config, expand_tilde};

pub fn run() -> Result<()> {
    let config_path = config::config_path()?;

    if config_path.exists() {
        let overwrite = Confirm::new("Config already exists. Reinitialize?")
            .with_default(false)
            .prompt()?;
        if !overwrite {
            println!("Aborted.");
            return Ok(());
        }
    }

    println!("Initializing halide...\n");

    let default_root = dirs::picture_dir()
        .unwrap_or_else(|| PathBuf::from("~/Pictures"))
        .join("Film");

    let archive_input = Text::new("Archive root path:")
        .with_default(&default_root.to_string_lossy())
        .prompt()?;

    let archive_root = expand_tilde(&archive_input);

    let default_photographer = Text::new("Default photographer:")
        .with_default("")
        .prompt()
        .unwrap_or_default();

    let cfg = Config {
        archive_root: archive_root.clone(),
        editor: std::env::var("EDITOR").ok(),
        film_stocks: config::default_film_stocks(),
        default_photographer,
    };

    config::save(&cfg)?;

    for dir in &["Rolls", "Exports", "Projects"] {
        let path = archive_root.join(dir);
        std::fs::create_dir_all(&path)?;
        println!("Created {}", path.display());
    }

    println!("\nConfig saved to {}", config_path.display());
    println!("halide initialized.");

    Ok(())
}
