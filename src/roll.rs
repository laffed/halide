use anyhow::{Context, Result};
use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

#[derive(Debug, Serialize, Deserialize)]
pub struct RollMetadata {
    pub uid: String,
    pub film: String,
    pub rated_iso: u32,
    pub camera: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub lens: String,
    pub developer: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub loaded_date: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub unloaded_date: String,
    pub developed_date: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub shot_date_range: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub notes: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tags: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub frames: Vec<FrameNote>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FrameNote {
    pub id: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub notes: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ScanMetadata {
    pub scanner: String,
    pub scan_software: String,
    pub dpi: u32,
    pub bit_depth: u8,
    pub infrared_cleaning: bool,
    pub multi_sampling: bool,
}

impl RollMetadata {
    pub fn load(roll_dir: &Path) -> Result<Self> {
        let path = roll_dir.join("metadata").join("roll.toml");
        let content = std::fs::read_to_string(&path)
            .with_context(|| format!("Could not read {}", path.display()))?;
        toml::from_str(&content).context("Could not parse roll.toml")
    }

    pub fn save(&self, roll_dir: &Path) -> Result<()> {
        let path = roll_dir.join("metadata").join("roll.toml");
        std::fs::write(&path, toml::to_string_pretty(self)?)?;
        Ok(())
    }
}

pub fn uid_from_date_and_number(date: NaiveDate, n: u32) -> String {
    format!("{}_{:02}", date.format("%Y-%m-%d"), n)
}

pub fn roll_dir_name(uid: &str, film: &str, ei: u32) -> String {
    let film_slug = film.replace(' ', "");
    format!("{}_{}@{}", uid, film_slug, ei)
}

pub fn find_rolls(archive_root: &Path) -> Result<Vec<PathBuf>> {
    let rolls_dir = archive_root.join("Rolls");
    if !rolls_dir.exists() {
        return Ok(vec![]);
    }

    let mut rolls = vec![];
    for year_entry in std::fs::read_dir(&rolls_dir)? {
        let year_entry = year_entry?;
        if !year_entry.file_type()?.is_dir() {
            continue;
        }
        for roll_entry in std::fs::read_dir(year_entry.path())? {
            let roll_entry = roll_entry?;
            if roll_entry.file_type()?.is_dir() {
                rolls.push(roll_entry.path());
            }
        }
    }

    rolls.sort();
    Ok(rolls)
}

pub fn raw_scans_count(roll_dir: &Path) -> usize {
    let raw_scans = roll_dir.join("raw_scans");
    if !raw_scans.exists() {
        return 0;
    }
    std::fs::read_dir(&raw_scans)
        .map(|rd| {
            rd.filter_map(|e| e.ok())
                .filter(|e| is_tiff(&e.file_name().to_string_lossy()))
                .count()
        })
        .unwrap_or(0)
}

pub fn next_frame_number(roll_dir: &Path, uid: &str) -> usize {
    let raw_scans = roll_dir.join("raw_scans");
    if !raw_scans.exists() {
        return 1;
    }
    let prefix = format!("{}_f", uid);
    let max = std::fs::read_dir(&raw_scans)
        .map(|rd| {
            rd.filter_map(|e| e.ok())
                .filter_map(|e| {
                    let name = e.file_name().to_string_lossy().to_string();
                    if name.starts_with(&prefix) && is_tiff(&name) {
                        let rest = &name[prefix.len()..];
                        rest.split('.').next()?.parse::<usize>().ok()
                    } else {
                        None
                    }
                })
                .max()
                .unwrap_or(0)
        })
        .unwrap_or(0);
    max + 1
}

pub fn get_frame_numbers(roll_dir: &Path, uid: &str) -> Vec<u32> {
    let raw_scans = roll_dir.join("raw_scans");
    let prefix = format!("{}_f", uid);
    let mut nums: Vec<u32> = std::fs::read_dir(&raw_scans)
        .map(|rd| {
            rd.filter_map(|e| e.ok())
                .filter_map(|e| {
                    let name = e.file_name().to_string_lossy().to_string();
                    if name.starts_with(&prefix) && is_tiff(&name) {
                        let rest = &name[prefix.len()..];
                        rest.split('.').next()?.parse().ok()
                    } else {
                        None
                    }
                })
                .collect()
        })
        .unwrap_or_default();
    nums.sort();
    nums
}

pub fn next_roll_number(rolls_year_dir: &Path, date: &NaiveDate) -> u32 {
    let prefix = date.format("%Y-%m-%d").to_string();
    if !rolls_year_dir.exists() {
        return 1;
    }
    let max = std::fs::read_dir(rolls_year_dir)
        .map(|rd| {
            rd.filter_map(|e| e.ok())
                .filter_map(|e| {
                    let name = e.file_name().to_string_lossy().to_string();
                    if name.starts_with(&prefix) {
                        // Name: YYYY-MM-DD_NN_...  rest after date prefix is _NN_...
                        let rest = &name[prefix.len()..];
                        // rest = "_01_HP5@1600"
                        rest.split('_').nth(1)?.parse::<u32>().ok()
                    } else {
                        None
                    }
                })
                .max()
                .unwrap_or(0)
        })
        .unwrap_or(0);
    max + 1
}

fn is_tiff(name: &str) -> bool {
    let lower = name.to_lowercase();
    lower.ends_with(".tif") || lower.ends_with(".tiff")
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    fn make_roll_dir(tmp: &TempDir) -> PathBuf {
        let roll_dir = tmp.path().join("2026-05-13_01_HP5+@1600");
        fs::create_dir_all(roll_dir.join("metadata")).unwrap();
        fs::create_dir_all(roll_dir.join("raw_scans")).unwrap();
        roll_dir
    }

    #[test]
    fn test_uid_format() {
        let date = NaiveDate::from_ymd_opt(2026, 5, 13).unwrap();
        assert_eq!(uid_from_date_and_number(date, 1), "2026-05-13_01");
        assert_eq!(uid_from_date_and_number(date, 9), "2026-05-13_09");
        assert_eq!(uid_from_date_and_number(date, 12), "2026-05-13_12");
    }

    #[test]
    fn test_roll_dir_name_strips_spaces() {
        assert_eq!(
            roll_dir_name("2026-05-13_01", "HP5+", 1600),
            "2026-05-13_01_HP5+@1600"
        );
        assert_eq!(
            roll_dir_name("2026-05-13_01", "Portra 400", 400),
            "2026-05-13_01_Portra400@400"
        );
        assert_eq!(
            roll_dir_name("2026-05-13_01", "Delta 400", 800),
            "2026-05-13_01_Delta400@800"
        );
    }

    #[test]
    fn test_metadata_round_trip() {
        let tmp = TempDir::new().unwrap();
        let roll_dir = make_roll_dir(&tmp);

        let meta = RollMetadata {
            uid: "2026-05-13_01".to_string(),
            film: "HP5+".to_string(),
            rated_iso: 1600,
            camera: "Leica M3".to_string(),
            lens: "50 Summicron".to_string(),
            developer: "HC-110B".to_string(),
            loaded_date: String::new(),
            unloaded_date: String::new(),
            developed_date: "2026-05-13".to_string(),
            shot_date_range: String::new(),
            notes: "Test notes".to_string(),
            tags: vec!["juniper".to_string(), "nyc".to_string()],
            frames: vec![FrameNote {
                id: "f01".to_string(),
                notes: "great light".to_string(),
            }],
        };

        meta.save(&roll_dir).unwrap();
        let loaded = RollMetadata::load(&roll_dir).unwrap();

        assert_eq!(loaded.uid, "2026-05-13_01");
        assert_eq!(loaded.film, "HP5+");
        assert_eq!(loaded.rated_iso, 1600);
        assert_eq!(loaded.tags, vec!["juniper", "nyc"]);
        assert_eq!(loaded.frames.len(), 1);
        assert_eq!(loaded.frames[0].id, "f01");
        assert_eq!(loaded.frames[0].notes, "great light");
    }

    #[test]
    fn test_raw_scans_count_empty_dir() {
        let tmp = TempDir::new().unwrap();
        let roll_dir = make_roll_dir(&tmp);
        assert_eq!(raw_scans_count(&roll_dir), 0);
    }

    #[test]
    fn test_raw_scans_count_only_tiffs() {
        let tmp = TempDir::new().unwrap();
        let roll_dir = make_roll_dir(&tmp);
        let raw = roll_dir.join("raw_scans");

        fs::write(raw.join("2026-05-13_01_f01.tif"), b"").unwrap();
        fs::write(raw.join("2026-05-13_01_f02.tif"), b"").unwrap();
        fs::write(raw.join("2026-05-13_01_f03.tiff"), b"").unwrap();
        fs::write(raw.join("README.txt"), b"").unwrap();

        assert_eq!(raw_scans_count(&roll_dir), 3);
    }

    #[test]
    fn test_next_frame_number_empty() {
        let tmp = TempDir::new().unwrap();
        let roll_dir = make_roll_dir(&tmp);
        assert_eq!(next_frame_number(&roll_dir, "2026-05-13_01"), 1);
    }

    #[test]
    fn test_next_frame_number_appends_correctly() {
        let tmp = TempDir::new().unwrap();
        let roll_dir = make_roll_dir(&tmp);
        let raw = roll_dir.join("raw_scans");

        fs::write(raw.join("2026-05-13_01_f01.tif"), b"").unwrap();
        fs::write(raw.join("2026-05-13_01_f02.tif"), b"").unwrap();

        assert_eq!(next_frame_number(&roll_dir, "2026-05-13_01"), 3);
    }

    #[test]
    fn test_next_frame_number_ignores_other_uids() {
        let tmp = TempDir::new().unwrap();
        let roll_dir = make_roll_dir(&tmp);
        let raw = roll_dir.join("raw_scans");

        fs::write(raw.join("2026-05-13_01_f01.tif"), b"").unwrap();
        // A file from a different UID should not affect the count
        fs::write(raw.join("2026-05-13_02_f05.tif"), b"").unwrap();

        assert_eq!(next_frame_number(&roll_dir, "2026-05-13_01"), 2);
    }

    #[test]
    fn test_get_frame_numbers_sorted_with_gaps() {
        let tmp = TempDir::new().unwrap();
        let roll_dir = make_roll_dir(&tmp);
        let raw = roll_dir.join("raw_scans");

        fs::write(raw.join("2026-05-13_01_f03.tif"), b"").unwrap();
        fs::write(raw.join("2026-05-13_01_f01.tif"), b"").unwrap();
        fs::write(raw.join("2026-05-13_01_f05.tif"), b"").unwrap();

        assert_eq!(
            get_frame_numbers(&roll_dir, "2026-05-13_01"),
            vec![1, 3, 5]
        );
    }

    #[test]
    fn test_next_roll_number_empty_dir() {
        let tmp = TempDir::new().unwrap();
        let date = NaiveDate::from_ymd_opt(2026, 5, 13).unwrap();
        assert_eq!(next_roll_number(tmp.path(), &date), 1);
    }

    #[test]
    fn test_next_roll_number_increments() {
        let tmp = TempDir::new().unwrap();
        fs::create_dir(tmp.path().join("2026-05-13_01_HP5+@1600")).unwrap();
        fs::create_dir(tmp.path().join("2026-05-13_02_TriX@800")).unwrap();

        let date = NaiveDate::from_ymd_opt(2026, 5, 13).unwrap();
        assert_eq!(next_roll_number(tmp.path(), &date), 3);
    }

    #[test]
    fn test_next_roll_number_ignores_different_date() {
        let tmp = TempDir::new().unwrap();
        fs::create_dir(tmp.path().join("2026-05-12_01_HP5+@1600")).unwrap();

        let date = NaiveDate::from_ymd_opt(2026, 5, 13).unwrap();
        assert_eq!(next_roll_number(tmp.path(), &date), 1);
    }
}
