use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize)]
pub struct ScanDefaults {
    pub scanner: String,
    pub scan_software: String,
    pub dpi: u32,
    pub bit_depth: u8,
    pub infrared_cleaning: bool,
    pub samples: u8,
}

impl Default for ScanDefaults {
    fn default() -> Self {
        Self {
            scanner: "Coolscan 5000".into(),
            scan_software: "VueScan".into(),
            dpi: 4000,
            bit_depth: 16,
            infrared_cleaning: true,
            samples: 1,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub archive_root: PathBuf,
    pub editor: Option<String>,
    pub film_stocks: Vec<String>,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub default_photographer: String,
    #[serde(default)]
    pub scan_defaults: ScanDefaults,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            archive_root: dirs::picture_dir()
                .unwrap_or_else(|| PathBuf::from("~/Pictures"))
                .join("Film"),
            editor: std::env::var("EDITOR").ok(),
            film_stocks: default_film_stocks(),
            default_photographer: String::new(),
            scan_defaults: ScanDefaults::default(),
        }
    }
}

pub fn default_film_stocks() -> Vec<String> {
    vec![
        "HP5+".into(),
        "Delta 400".into(),
        "Delta 3200".into(),
        "FP4+".into(),
        "Pan F".into(),
        "SFX 200".into(),
        "Tri-X 400".into(),
        "T-Max 100".into(),
        "T-Max 400".into(),
        "Acros 100 II".into(),
        "Rollei RPX 400".into(),
        "Portra 160".into(),
        "Portra 400".into(),
        "Ektar 100".into(),
        "Gold 200".into(),
        "Ultramax 400".into(),
        "ColorPlus 200".into(),
        "Superia X-TRA 400".into(),
        "ProImage 100".into(),
        "Velvia 50".into(),
        "Velvia 100".into(),
        "Provia 100F".into(),
        "CineStill 800T".into(),
        "CineStill 400D".into(),
        "CineStill 50D".into(),
    ]
}

pub fn config_path() -> Result<PathBuf> {
    let home = dirs::home_dir().context("Could not find home directory")?;
    Ok(home.join(".config").join("halide").join("config.toml"))
}

pub fn load() -> Result<Config> {
    let path = config_path()?;
    if !path.exists() {
        anyhow::bail!("Config not found. Run `halide init` first.");
    }
    let content = std::fs::read_to_string(&path)
        .with_context(|| format!("Could not read config at {}", path.display()))?;
    toml::from_str(&content).context("Could not parse config")
}

pub fn save(config: &Config) -> Result<()> {
    let path = config_path()?;
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let content = toml::to_string_pretty(config)?;
    std::fs::write(&path, content)?;
    Ok(())
}

pub fn expand_tilde(s: &str) -> PathBuf {
    if let Some(rest) = s.strip_prefix("~/")
        && let Some(home) = dirs::home_dir()
    {
        return home.join(rest);
    }
    PathBuf::from(s)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_expand_tilde_absolute_path_unchanged() {
        let result = expand_tilde("/Users/roark/Pictures/Film");
        assert_eq!(result, PathBuf::from("/Users/roark/Pictures/Film"));
    }

    #[test]
    fn test_expand_tilde_expands_home() {
        let home = dirs::home_dir().unwrap();
        assert_eq!(expand_tilde("~/Pictures"), home.join("Pictures"));
        assert_eq!(expand_tilde("~/a/b/c"), home.join("a/b/c"));
    }

    #[test]
    fn test_expand_tilde_bare_tilde_unchanged() {
        // "~" alone has no trailing slash so strip_prefix("~/") won't match
        assert_eq!(expand_tilde("~"), PathBuf::from("~"));
    }

    #[test]
    fn test_default_film_stocks_contains_expected() {
        let stocks = default_film_stocks();
        assert!(!stocks.is_empty());
        assert!(stocks.iter().any(|s| s == "HP5+"));
        assert!(stocks.iter().any(|s| s == "Portra 400"));
        assert!(stocks.iter().any(|s| s == "Tri-X 400"));
    }
}
