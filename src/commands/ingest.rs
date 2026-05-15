use anyhow::{Context, Result};
use inquire::{Confirm, Select, Text};
use std::path::{Path, PathBuf};

use crate::config;
use crate::roll::{self, RollMetadata, ScanMetadata};

pub fn run(source: Option<String>) -> Result<()> {
    let cfg = config::load()?;

    let source_input = match source {
        Some(s) => s,
        None => Text::new("Source directory (containing TIFF files):").prompt()?,
    };
    let source_dir = PathBuf::from(&source_input);

    if !source_dir.exists() {
        anyhow::bail!("Directory not found: {}", source_dir.display());
    }

    let mut tiffs = collect_tiffs(&source_dir)?;
    if tiffs.is_empty() {
        anyhow::bail!("No TIFF files found in {}", source_dir.display());
    }
    tiffs.sort();
    println!("Found {} TIFF file(s)", tiffs.len());

    let all_rolls = roll::find_rolls(&cfg.archive_root)?;
    if all_rolls.is_empty() {
        anyhow::bail!("No rolls found. Run `halide new` first.");
    }

    let mut empty: Vec<PathBuf> = vec![];
    let mut nonempty: Vec<PathBuf> = vec![];

    for r in all_rolls {
        if roll::raw_scans_count(&r) == 0 {
            empty.push(r);
        } else {
            nonempty.push(r);
        }
    }

    let mut options: Vec<(String, PathBuf)> = vec![];
    for r in &empty {
        let name = r.file_name().unwrap_or_default().to_string_lossy();
        options.push((format!("[empty]    {}", name), r.clone()));
    }
    for r in &nonempty {
        let count = roll::raw_scans_count(r);
        let name = r.file_name().unwrap_or_default().to_string_lossy();
        options.push((format!("[{} frames]  {}", count, name), r.clone()));
    }

    let display: Vec<String> = options.iter().map(|(s, _)| s.clone()).collect();
    let selected = Select::new("Target roll:", display.clone()).prompt()?;
    let idx = display.iter().position(|s| s == &selected).unwrap();
    let target_roll = options[idx].1.clone();

    let existing = roll::raw_scans_count(&target_roll);
    if existing > 0 {
        println!("Warning: {} already has {} frame(s).", target_roll.file_name().unwrap_or_default().to_string_lossy(), existing);
        let confirm = Confirm::new("Append to existing frames?")
            .with_default(false)
            .prompt()?;
        if !confirm {
            println!("Aborted.");
            return Ok(());
        }
    }

    let meta = RollMetadata::load(&target_roll)?;
    let uid = &meta.uid;
    let start_frame = roll::next_frame_number(&target_roll, uid);
    let raw_scans_dir = target_roll.join("raw_scans");

    println!("\nIngesting {} file(s) starting at f{:02}...", tiffs.len(), start_frame);

    for (i, src) in tiffs.iter().enumerate() {
        let frame_n = start_frame + i;
        let ext = src
            .extension()
            .map(|e| e.to_string_lossy().to_lowercase())
            .unwrap_or_else(|| "tif".into());
        let dest_name = format!("{}_f{:02}.{}", uid, frame_n, ext);
        let dest = raw_scans_dir.join(&dest_name);

        move_file(src, &dest)
            .with_context(|| format!("Failed to move {}", src.display()))?;

        println!(
            "  {} → {}",
            src.file_name().unwrap_or_default().to_string_lossy(),
            dest_name
        );
    }

    let scan_meta = ScanMetadata {
        scanner: cfg.scanner.scanner.clone(),
        scan_software: cfg.scanner.scan_software.clone(),
        dpi: cfg.scanner.dpi,
        bit_depth: cfg.scanner.bit_depth,
        infrared_cleaning: cfg.scanner.infrared_cleaning,
        multi_sampling: cfg.scanner.multi_sampling,
    };
    std::fs::write(
        target_roll.join("metadata").join("scan.toml"),
        toml::to_string_pretty(&scan_meta)?,
    )?;

    println!("\nDone. {} frame(s) ingested into {}.", tiffs.len(), uid);

    Ok(())
}

fn collect_tiffs(dir: &Path) -> Result<Vec<PathBuf>> {
    let mut result = vec![];
    for entry in std::fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_file() {
            let ext = path
                .extension()
                .map(|e| e.to_string_lossy().to_lowercase())
                .unwrap_or_default();
            if ext == "tif" || ext == "tiff" {
                result.push(path);
            }
        }
    }
    Ok(result)
}

fn move_file(src: &Path, dest: &Path) -> Result<()> {
    if std::fs::rename(src, dest).is_err() {
        std::fs::copy(src, dest)?;
        std::fs::remove_file(src)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_collect_tiffs_filters_by_extension() {
        let tmp = TempDir::new().unwrap();
        let dir = tmp.path();

        std::fs::write(dir.join("scan01.tif"), b"").unwrap();
        std::fs::write(dir.join("scan02.tiff"), b"").unwrap();
        std::fs::write(dir.join("scan03.TIF"), b"").unwrap();
        std::fs::write(dir.join("scan04.jpg"), b"").unwrap();
        std::fs::write(dir.join("notes.txt"), b"").unwrap();

        let tiffs = collect_tiffs(dir).unwrap();
        assert_eq!(tiffs.len(), 3);
    }

    #[test]
    fn test_collect_tiffs_empty_dir() {
        let tmp = TempDir::new().unwrap();
        let tiffs = collect_tiffs(tmp.path()).unwrap();
        assert!(tiffs.is_empty());
    }

    #[test]
    fn test_move_file_moves_and_removes_source() {
        let tmp = TempDir::new().unwrap();
        let src = tmp.path().join("source.tif");
        let dest = tmp.path().join("dest.tif");

        std::fs::write(&src, b"fake tiff").unwrap();
        move_file(&src, &dest).unwrap();

        assert!(!src.exists());
        assert!(dest.exists());
        assert_eq!(std::fs::read(&dest).unwrap(), b"fake tiff");
    }
}
