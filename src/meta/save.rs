//! Versioned save file (RON) in the platform data dir.
//!
//! Compatibility rules:
//! - Every field has a default, so older files load and newer fields fill in.
//! - Unknown fields are ignored, and ids of content that no longer exists are
//!   kept untouched, so a file survives a trip through an older or newer build.
//! - The save holds only plain values (numbers, strings, maps), never enums, so
//!   a newer build can add variants without breaking older readers.
//! - A file from a newer version is loaded read-only and never overwritten.

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

/// 1: bones, best times, unlocks. 2: upgrades, feats, records.
pub const VERSION: u32 = 2;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct Save {
    pub version: u32,
    /// Meta currency.
    #[serde(default)]
    pub bones: u32,
    /// Best survival time in seconds, per character id.
    #[serde(default)]
    pub best: BTreeMap<String, f32>,
    /// Ids of purchased or earned unlocks (items, characters).
    #[serde(default)]
    pub unlocked: BTreeSet<String>,
    #[serde(default)]
    pub runs: u32,
    /// Last picked character id.
    #[serde(default)]
    pub character: String,
    /// Upgrade id to level bought.
    #[serde(default)]
    pub upgrades: BTreeMap<String, u32>,
    /// Ids of completed feats.
    #[serde(default)]
    pub feats: BTreeSet<String>,
    #[serde(default)]
    pub records: Records,
    /// Set when the file came from a newer build; we then never overwrite it.
    #[serde(skip)]
    pub read_only: bool,
}

/// Lifetime records that feats read.
#[derive(Serialize, Deserialize, Debug, Clone, Default, PartialEq)]
#[serde(default)]
pub struct Records {
    /// Most kills in one run.
    pub kills: u32,
    /// Highest level in one run.
    pub level: u32,
    /// Most stacks of one item in one run.
    pub stacks: u32,
    /// Most different items in one run.
    pub items: u32,
    pub total_kills: u64,
    /// Bones earned over all runs (spending does not lower it).
    pub total_bones: u64,
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
            upgrades: BTreeMap::new(),
            feats: BTreeSet::new(),
            records: Records::default(),
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
        return Ok(save);
    }
    if save.version < 2 {
        // v1 kept no lifetime totals; the unspent balance is the best lower bound.
        save.records.total_bones = u64::from(save.bones);
    }
    save.version = VERSION;
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
        s.upgrades.insert("sharp_teeth".into(), 3);
        s.feats.insert("hatchling".into());
        s.records.total_kills = 9001;
        let text = ron::ser::to_string_pretty(&s, ron::ser::PrettyConfig::default()).unwrap();
        assert_eq!(parse(&text).unwrap(), s);
    }

    #[test]
    fn missing_fields_default_and_unknown_fields_are_ignored() {
        let s = parse("(version: 2, bones: 7, future_field: 3, records: (kills: 5, later: [1]))").unwrap();
        assert_eq!(s.bones, 7);
        assert_eq!(s.records.kills, 5);
        assert!(s.best.is_empty());
        assert!(!s.read_only);
    }

    #[test]
    fn v1_migrates() {
        let v1 = r#"(version: 1, bones: 80, best: {"rex": 200.0}, unlocked: ["trike"], runs: 4, character: "rex")"#;
        let s = parse(v1).unwrap();
        assert_eq!(s.version, VERSION);
        assert_eq!(s.records.total_bones, 80);
        assert_eq!(s.best["rex"], 200.0);
        assert!(s.unlocked.contains("trike") && s.upgrades.is_empty() && s.feats.is_empty());
    }

    #[test]
    fn newer_version_is_read_only() {
        let s = parse("(version: 99, bones: 1)").unwrap();
        assert!(s.read_only);
        assert_eq!(s.version, 99);
    }

    #[test]
    fn garbage_is_an_error() {
        assert!(parse("not a save").is_err());
    }
}
