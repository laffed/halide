use anyhow::Result;
use inquire::{Confirm, Text};
use std::path::PathBuf;

use crate::config::{self, Config, ScannerDefaults, expand_tilde};

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

    let scanner = Text::new("Scanner name:")
        .with_default("Coolscan 5000")
        .prompt()?;

    let scan_software = Text::new("Scan software:")
        .with_default("VueScan")
        .prompt()?;

    let dpi_input = Text::new("Default DPI:")
        .with_default("4000")
        .prompt()?;
    let dpi: u32 = dpi_input.parse().unwrap_or(4000);

    let infrared = Confirm::new("Use infrared cleaning by default?")
        .with_default(true)
        .prompt()?;

    let cfg = Config {
        archive_root: archive_root.clone(),
        editor: std::env::var("EDITOR").ok(),
        scanner: ScannerDefaults {
            scanner,
            scan_software,
            dpi,
            bit_depth: 16,
            infrared_cleaning: infrared,
            multi_sampling: false,
        },
        film_stocks: config::default_film_stocks(),
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
