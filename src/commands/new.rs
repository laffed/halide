use anyhow::Result;
use chrono::{Local, NaiveDate};
use inquire::{Select, Text};

use crate::config;
use crate::roll::{self, RollMetadata};

pub fn run() -> Result<()> {
    let cfg = config::load()?;

    let today = Local::now().date_naive().format("%Y-%m-%d").to_string();
    let date_input = Text::new("Development date (YYYY-MM-DD):")
        .with_default(&today)
        .prompt()?;

    let dev_date = NaiveDate::parse_from_str(&date_input, "%Y-%m-%d")
        .map_err(|_| anyhow::anyhow!("Invalid date. Use YYYY-MM-DD."))?;

    let year = dev_date.format("%Y").to_string();
    let rolls_year_dir = cfg.archive_root.join("Rolls").join(&year);
    let next_n = roll::next_roll_number(&rolls_year_dir, &dev_date);

    let n_input = Text::new("Roll number:")
        .with_default(&next_n.to_string())
        .prompt()?;
    let n: u32 = n_input.trim().parse().unwrap_or(next_n);

    let uid = roll::uid_from_date_and_number(dev_date, n);

    let mut stocks = cfg.film_stocks.clone();
    stocks.push("Other...".to_string());
    let film_selection = Select::new("Film stock:", stocks).prompt()?;
    let film = if film_selection == "Other..." {
        Text::new("Film stock:").prompt()?
    } else {
        film_selection
    };

    let ei_input = Text::new("Rated ISO (EI):").prompt()?;
    let ei: u32 = ei_input.trim().parse().unwrap_or(400);

    let camera = Text::new("Camera:").prompt()?;

    let lens = Text::new("Lens (optional):")
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

    let folder_name = roll::roll_dir_name(&uid, &film, ei);
    let roll_dir = rolls_year_dir.join(&folder_name);

    println!("\nCreating {}", folder_name);

    std::fs::create_dir_all(&roll_dir)?;
    for subdir in &["raw_scans", "edits", "exports", "contact_sheet", "metadata"] {
        std::fs::create_dir_all(roll_dir.join(subdir))?;
    }

    let metadata = RollMetadata {
        uid: uid.clone(),
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
    };

    metadata.save(&roll_dir)?;
    std::fs::write(roll_dir.join("notes.md"), format!("# {}\n\n", uid))?;

    println!("UID:  {}", uid);
    println!("Path: {}", roll_dir.display());

    Ok(())
}
