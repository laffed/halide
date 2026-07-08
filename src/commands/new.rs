use anyhow::Result;
use chrono::{Local, NaiveDate};
use inquire::{Confirm, Select, Text};

use crate::config;
use crate::roll::{self, RollMetadata, ScanMetadata};

pub fn run() -> Result<()> {
    let mut cfg = config::load()?;

    let today = Local::now().date_naive().format("%Y-%m-%d").to_string();
    let date_input = Text::new("Development date (YYYY-MM-DD):")
        .with_default(&today)
        .prompt()?;

    let dev_date = NaiveDate::parse_from_str(&date_input, "%Y-%m-%d")
        .map_err(|_| anyhow::anyhow!("Invalid date. Use YYYY-MM-DD."))?;

    let rolls_dir = cfg.archive_root.join("Rolls");
    let next_n = roll::next_roll_number(&rolls_dir, &dev_date);

    let n_input = Text::new("Roll number:")
        .with_default(&next_n.to_string())
        .prompt()?;
    let n: u32 = n_input.trim().parse().unwrap_or(next_n);

    let uid = roll::uid_from_date_and_number(dev_date, n);

    let photographer = Text::new("Photographer:")
        .with_default(&cfg.default_photographer)
        .prompt()
        .unwrap_or_default();

    let film = loop {
        match Select::new("Film stock:", cfg.film_stocks.clone())
            .with_vim_mode(true)
            .with_help_message("Esc to enter a custom stock")
            .prompt_skippable()?
        {
            Some(selected) => break selected,
            None => {
                let custom = Text::new("Custom film stock:").prompt()?;
                let custom = custom.trim().to_string();
                if !custom.is_empty() {
                    break custom;
                }
            }
        }
    };

    cfg.film_stocks.retain(|s| s != &film);
    cfg.film_stocks.insert(0, film.clone());
    config::save(&cfg)?;

    let ei_input = Text::new("Rated ISO (EI):").prompt()?;
    let ei: u32 = ei_input.trim().parse().unwrap_or(400);

    let camera = Text::new("Camera:").prompt()?;

    let lens = Text::new("Lens:")
        .with_default("")
        .prompt()
        .unwrap_or_default();

    let developer = Text::new("Developer:").prompt()?;

    let shot_range = Text::new("Shot date range (e.g. 2026-03 → 2026-04, or leave blank):")
        .with_default("")
        .prompt()
        .unwrap_or_default();

    let notes = Text::new("Notes:")
        .with_default("")
        .prompt()
        .unwrap_or_default();

    let tags_input = Text::new("Tags (comma-separated):")
        .with_default("")
        .prompt()
        .unwrap_or_default();
    let tags: Vec<String> = tags_input
        .split(',')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect();

    println!("\nScanner setup");

    let sd = &cfg.scan_defaults;
    let scanner = Text::new("Scanner:")
        .with_default(&sd.scanner)
        .prompt()?;

    let scan_software = Text::new("Scan software:")
        .with_default(&sd.scan_software)
        .prompt()?;

    let dpi: u32 = Text::new("DPI:")
        .with_default(&sd.dpi.to_string())
        .prompt()?
        .trim()
        .parse()
        .unwrap_or(sd.dpi);

    let bit_depth = loop {
        match Select::new("Bit depth:", cfg.bit_depth_options.clone())
            .with_vim_mode(true)
            .with_help_message("Esc to enter a custom bit depth")
            .prompt_skippable()?
        {
            Some(selected) => break selected,
            None => {
                let custom = Text::new("Custom bit depth:").prompt()?;
                let custom = custom.trim().to_string();
                if !custom.is_empty() {
                    break custom;
                }
            }
        }
    };

    cfg.bit_depth_options.retain(|s| s != &bit_depth);
    cfg.bit_depth_options.insert(0, bit_depth.clone());
    config::save(&cfg)?;

    let infrared_cleaning = Confirm::new("Infrared cleaning?")
        .with_default(config::is_color_bit_depth(&bit_depth) && sd.infrared_cleaning)
        .prompt()?;

    let samples: u8 = Text::new("Samples:")
        .with_default(&sd.samples.to_string())
        .prompt()?
        .trim()
        .parse()
        .unwrap_or(sd.samples);

    let folder_name = roll::roll_dir_name(&uid, &film, ei);
    let roll_dir = rolls_dir.join(&folder_name);

    println!("\nCreating {}", folder_name);

    std::fs::create_dir_all(&roll_dir)?;
    for subdir in &["raw_scans", "edits", "exports", "contact_sheet"] {
        std::fs::create_dir_all(roll_dir.join(subdir))?;
    }

    let metadata = RollMetadata {
        uid: uid.clone(),
        photographer,
        film,
        rated_iso: ei,
        camera,
        lens,
        developer,
        loaded_date: String::new(),
        unloaded_date: String::new(),
        developed_date: dev_date.format("%Y-%m-%d").to_string(),
        shot_date_range: shot_range,
        notes,
        tags,
        frames: vec![],
        scan: Some(ScanMetadata {
            scanner,
            scan_software,
            dpi,
            bit_depth,
            infrared_cleaning,
            samples,
        }),
    };

    metadata.save(&roll_dir)?;
    std::fs::write(roll_dir.join("notes.md"), format!("# {}\n\n", uid))?;

    println!("UID:  {}", uid);
    println!("Path: {}", roll_dir.display());

    Ok(())
}
