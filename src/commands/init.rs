use anyhow::Result;
use inquire::{Confirm, Select, Text};
use std::path::PathBuf;

use crate::config::{self, Config, ScanDefaults, expand_tilde};

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

    println!("\nScanner defaults");

    let sd = ScanDefaults::default();
    let scanner = Text::new("Scanner:")
        .with_default(&sd.scanner)
        .prompt()
        .unwrap_or(sd.scanner);
    let scan_software = Text::new("Scan software:")
        .with_default(&sd.scan_software)
        .prompt()
        .unwrap_or(sd.scan_software);
    let dpi: u32 = Text::new("DPI:")
        .with_default(&sd.dpi.to_string())
        .prompt()
        .unwrap_or_default()
        .trim()
        .parse()
        .unwrap_or(sd.dpi);
    let bit_depth = loop {
        match Select::new("Bit depth:", config::default_bit_depth_options())
            .with_vim_mode(true)
            .with_help_message("Esc to enter a custom bit depth")
            .prompt_skippable()
            .unwrap_or(None)
        {
            Some(selected) => break selected,
            None => {
                let custom = Text::new("Custom bit depth:").prompt().unwrap_or_default();
                let custom = custom.trim().to_string();
                if !custom.is_empty() {
                    break custom;
                }
            }
        }
    };
    let infrared_cleaning = Confirm::new("Infrared cleaning?")
        .with_default(sd.infrared_cleaning)
        .prompt()
        .unwrap_or(sd.infrared_cleaning);
    let samples: u8 = Text::new("Samples:")
        .with_default(&sd.samples.to_string())
        .prompt()
        .unwrap_or_default()
        .trim()
        .parse()
        .unwrap_or(sd.samples);

    let cfg = Config {
        archive_root: archive_root.clone(),
        editor: std::env::var("EDITOR").ok(),
        film_stocks: config::default_film_stocks(),
        bit_depth_options: config::default_bit_depth_options(),
        default_photographer,
        scan_defaults: ScanDefaults { scanner, scan_software, dpi, bit_depth, infrared_cleaning, samples },
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
