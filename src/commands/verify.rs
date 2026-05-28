use anyhow::Result;
use std::collections::HashSet;

use crate::config;
use crate::roll::{self, RollMetadata};

pub fn run() -> Result<()> {
    let cfg = config::load()?;

    let all_rolls = roll::find_rolls(&cfg.archive_root)?;
    if all_rolls.is_empty() {
        println!("No rolls found.");
        return Ok(());
    }

    let mut errors = 0;
    let mut warnings = 0;
    let mut seen_uids: HashSet<String> = HashSet::new();

    for roll_dir in &all_rolls {
        let name = roll_dir.file_name().unwrap_or_default().to_string_lossy();
        let toml_path = roll_dir.join("roll.toml");

        if !toml_path.exists() {
            println!("[ERROR] {}: missing roll.toml", name);
            errors += 1;
            continue;
        }

        let meta = match RollMetadata::load(roll_dir) {
            Ok(m) => m,
            Err(e) => {
                println!("[ERROR] {}: {}", name, e);
                errors += 1;
                continue;
            }
        };

        if !seen_uids.insert(meta.uid.clone()) {
            println!("[ERROR] {}: duplicate UID {}", name, meta.uid);
            errors += 1;
        }

        if !roll_dir.join("raw_scans").exists() {
            println!("[WARN]  {}: missing raw_scans/", name);
            warnings += 1;
        }

        let bad = roll::nonconforming_scans(roll_dir, &meta.uid);
        for f in &bad {
            println!("[WARN]  {}: non-conforming filename in raw_scans: {}", name, f);
            warnings += 1;
        }

        let frames = roll::get_frame_numbers(roll_dir, &meta.uid);
        if !frames.is_empty() {
            let expected: Vec<u32> = (1..=frames.len() as u32).collect();
            if frames != expected {
                println!("[WARN]  {}: frame sequence has gaps: {:?}", name, frames);
                warnings += 1;
            }
            println!("[OK]    {} ({} frames)", name, frames.len());
        } else {
            println!("[OK]    {} (no frames ingested)", name);
        }
    }

    println!();
    if errors == 0 && warnings == 0 {
        println!("All {} roll(s) OK.", all_rolls.len());
    } else {
        if errors > 0 {
            println!("{} error(s)", errors);
        }
        if warnings > 0 {
            println!("{} warning(s)", warnings);
        }
    }

    Ok(())
}
