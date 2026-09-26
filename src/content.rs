//! Game data embedded from `content/*.ron`. Parsed and cross-checked once at
//! startup; everything else refers to entries by index.

use std::sync::OnceLock;

use serde::de::DeserializeOwned;

use crate::enemies::EnemyDef;
use crate::enemies::director::WavesDef;
use crate::items::ItemDef;
use crate::items::synergy::SynergyDef;
use crate::meta::characters::CharacterDef;
use crate::render::sprite::{SpriteId, bank};

const ITEMS: &str = include_str!("../content/items.ron");
const ENEMIES: &str = include_str!("../content/enemies.ron");
const CHARACTERS: &str = include_str!("../content/characters.ron");
const WAVES: &str = include_str!("../content/waves.ron");
const SYNERGIES: &str = include_str!("../content/synergies.ron");

pub struct Content {
    pub items: Vec<ItemDef>,
    pub enemies: Vec<EnemyDef>,
    pub characters: Vec<CharacterDef>,
    pub waves: WavesDef,
    pub synergies: Vec<SynergyDef>,
}

static CONTENT: OnceLock<Content> = OnceLock::new();

/// The global content, loaded on first use. Invalid data is a startup panic
/// with the file and position.
pub fn get() -> &'static Content {
    CONTENT.get_or_init(|| Content::load().unwrap_or_else(|e| panic!("content: {e}")))
}

fn parse<T: DeserializeOwned>(file: &str, src: &str) -> Result<T, String> {
    ron::from_str(src).map_err(|e| format!("content/{file}: {e}"))
}

fn sprite(name: &str, owner: &str) -> Result<SpriteId, String> {
    bank().id(name).ok_or_else(|| format!("{owner}: unknown sprite `{name}`"))
}

fn unique<'a>(kind: &str, ids: impl Iterator<Item = &'a String>) -> Result<(), String> {
    let mut seen = std::collections::HashSet::new();
    for id in ids {
        if !seen.insert(id) {
            return Err(format!("duplicate {kind} id `{id}`"));
        }
    }
    Ok(())
}

impl Content {
    pub fn load() -> Result<Content, String> {
        let mut c = Content {
            items: parse("items.ron", ITEMS)?,
            enemies: parse("enemies.ron", ENEMIES)?,
            characters: parse("characters.ron", CHARACTERS)?,
            waves: parse("waves.ron", WAVES)?,
            synergies: parse("synergies.ron", SYNERGIES)?,
        };
        c.resolve()?;
        Ok(c)
    }

    fn resolve(&mut self) -> Result<(), String> {
        unique("item", self.items.iter().map(|i| &i.id))?;
        unique("enemy", self.enemies.iter().map(|e| &e.id))?;
        unique("character", self.characters.iter().map(|c| &c.id))?;
        if self.characters.is_empty() {
            return Err("no characters".into());
        }
        for it in &mut self.items {
            it.sprite_id = sprite(&it.sprite, &format!("item `{}`", it.id))?;
        }
        crate::items::resolve(&mut self.items, &mut self.synergies)?;
        crate::enemies::resolve(&mut self.enemies)?;
        for ch in &mut self.characters {
            let owner = format!("character `{}`", ch.id);
            ch.sprite_id = sprite(&ch.sprite, &owner)?;
            ch.weapon.shot_id = sprite(&ch.weapon.shot, &owner)?;
        }
        self.waves.resolve(&self.enemies)
    }

    pub fn character(&self, id: &str) -> Option<usize> {
        self.characters.iter().position(|c| c.id == id)
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn embedded_content_loads() {
        let c = super::Content::load().unwrap_or_else(|e| panic!("{e}"));
        assert!(!c.items.is_empty() && !c.enemies.is_empty());
    }
}
