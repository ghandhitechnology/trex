//! Versioned save file (RON) in the platform data dir.

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

pub const VERSION: u32 = 1;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct Save {
    pub version: u32,
    /// Meta currency.
    #[serde(default)]
    pub bones: u32,
    /// Best survival time in seconds, per character id.
    #[serde(default)]
    pub best: BTreeMap<String, f32>,
    /// Ids of purchased unlocks (items, characters).
    #[serde(default)]
    pub unlocked: BTreeSet<String>,
    #[serde(default)]
    pub runs: u32,
    /// Last picked character id.
    #[serde(default)]
    pub character: String,
    /// Set when the file came from a newer build; we then never overwrite it.
    #[serde(skip)]
    pub read_only: bool,
}

impl Default for Save {
    fn default() -> Self {
        Save {
            version: VERSION,
            bones: 0,
            best: BTreeMap::new(),
            unlocked: BTreeSet::new(),
            runs: 0,
            character: String::new(),
            read_only: false,
        }
    }
}

/// `$TREX_SAVE` if set, else `<data dir>/trex/save.ron`.
pub fn default_path() -> Option<PathBuf> {
    if let Some(p) = std::env::var_os("TREX_SAVE") {
        return Some(PathBuf::from(p));
    }
    dirs::data_dir().map(|d| d.join("trex").join("save.ron"))
}

/// Parse save text, upgrading older versions.
pub fn parse(text: &str) -> Result<Save, String> {
    let mut save: Save = ron::from_str(text).map_err(|e| e.to_string())?;
    if save.version > VERSION {
        save.read_only = true;
    }
    // Older versions migrate here as the format evolves.
    save.version = save.version.max(VERSION);
    Ok(save)
}

/// Missing file → fresh save. Unreadable file → moved aside to `.bad`, fresh save.
pub fn load(path: &Path) -> Save {
    match fs::read_to_string(path) {
        Ok(text) => parse(&text).unwrap_or_else(|_| {
            let _ = fs::rename(path, path.with_extension("ron.bad"));
            Save::default()
        }),
        Err(_) => Save::default(),
    }
}

/// Atomic write: temp file then rename.
pub fn store(path: &Path, save: &Save) -> io::Result<()> {
    if save.read_only {
        return Ok(());
    }
    if let Some(dir) = path.parent() {
        fs::create_dir_all(dir)?;
    }
    let text =
        ron::ser::to_string_pretty(save, ron::ser::PrettyConfig::default()).map_err(io::Error::other)?;
    let tmp = path.with_extension("ron.tmp");
    fs::write(&tmp, text)?;
    fs::rename(tmp, path)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrip() {
        let mut s = Save { bones: 42, ..Save::default() };
        s.best.insert("rex".into(), 123.5);
        s.unlocked.insert("drill_bits".into());
        let text = ron::ser::to_string_pretty(&s, ron::ser::PrettyConfig::default()).unwrap();
        assert_eq!(parse(&text).unwrap(), s);
    }

    #[test]
    fn missing_fields_default_and_unknown_fields_are_ignored() {
        let s = parse("(version: 1, bones: 7, future_field: 3)").unwrap();
        assert_eq!(s.bones, 7);
        assert!(s.best.is_empty());
        assert!(!s.read_only);
    }

    #[test]
    fn newer_version_is_read_only() {
        let s = parse("(version: 99, bones: 1)").unwrap();
        assert!(s.read_only);
    }

    #[test]
    fn garbage_is_an_error() {
        assert!(parse("not a save").is_err());
    }
}
