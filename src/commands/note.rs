use anyhow::Result;
use inquire::{InquireError, Select, Text};
use std::path::Path;

use crate::config;
use crate::roll::{self, FrameNote, RollMetadata};

pub fn run(uid: Option<String>) -> Result<()> {
    let cfg = config::load()?;

    let all_rolls = roll::find_rolls(&cfg.archive_root)?;
    if all_rolls.is_empty() {
        anyhow::bail!("No rolls found. Run `halide new` first.");
    }

    let roll_names: Vec<String> = all_rolls
        .iter()
        .map(|p| p.file_name().unwrap_or_default().to_string_lossy().to_string())
        .collect();

    let roll_dir = if let Some(pattern) = uid {
        let matches: Vec<_> = all_rolls
            .iter()
            .zip(roll_names.iter())
            .filter(|(_, name)| name.contains(&pattern))
            .collect();
        match matches.len() {
            0 => anyhow::bail!("No rolls matching '{}'", pattern),
            1 => matches[0].0,
            _ => {
                let names: Vec<String> = matches.iter().map(|(_, n)| (*n).clone()).collect();
                let selected = Select::new("Select roll:", names.clone()).with_vim_mode(true).prompt()?;
                let idx = names.iter().position(|s| s == &selected).unwrap();
                matches[idx].0
            }
        }
    } else {
        let selected = Select::new("Select roll:", roll_names.clone()).with_vim_mode(true).prompt()?;
        let idx = roll_names.iter().position(|s| s == &selected).unwrap();
        &all_rolls[idx]
    };

    let action = Select::new(
        "Action:",
        vec![
            "Open roll.toml in $EDITOR",
            "Add frame notes interactively",
        ],
    )
    .with_vim_mode(true)
    .prompt()?;

    match action {
        "Open roll.toml in $EDITOR" => open_in_editor(roll_dir, &cfg.editor),
        "Add frame notes interactively" => add_frame_notes(roll_dir),
        _ => Ok(()),
    }
}

fn open_in_editor(roll_dir: &Path, cfg_editor: &Option<String>) -> Result<()> {
    let toml_path = roll_dir.join("roll.toml");
    let env_editor = std::env::var("EDITOR").ok();
    let editor = cfg_editor
        .as_deref()
        .or(env_editor.as_deref())
        .unwrap_or("vi");

    std::process::Command::new(editor)
        .arg(&toml_path)
        .status()
        .map_err(|e| anyhow::anyhow!("Failed to launch '{}': {}", editor, e))?;

    Ok(())
}

fn add_frame_notes(roll_dir: &Path) -> Result<()> {
    let mut meta = RollMetadata::load(roll_dir)?;
    let frame_count = roll::raw_scans_count(roll_dir);

    if frame_count == 0 {
        println!("No frames found in raw_scans. Run `halide ingest` first.");
        return Ok(());
    }

    println!("Entering notes for {} frame(s). Press Enter to skip.\n", frame_count);

    for i in 1..=frame_count {
        let frame_id = format!("f{:02}", i);
        let existing = meta
            .frames
            .iter()
            .find(|f| f.id == frame_id)
            .map(|f| f.notes.as_str())
            .unwrap_or("");

        let note = match Text::new(&format!("Frame {}:", frame_id))
            .with_default(existing)
            .prompt()
        {
            Ok(n) => n,
            Err(InquireError::OperationInterrupted | InquireError::OperationCanceled) => break,
            Err(e) => return Err(e.into()),
        };

        if let Some(frame) = meta.frames.iter_mut().find(|f| f.id == frame_id) {
            frame.notes = note;
        } else if !note.is_empty() {
            meta.frames.push(FrameNote {
                id: frame_id,
                notes: note,
            });
        }
    }

    meta.save(roll_dir)?;
    println!("\nFrame notes saved.");

    Ok(())
}
