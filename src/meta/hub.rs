//! The meta hub: hero select, shop, and feats. Keyboard only: WASD or arrows
//! move, Space or Enter confirms, W from the top row reaches the tab bar,
//! Esc goes back to the title.

use super::save::Save;
use super::{Buy, hero_unlocked, purchase, shop_entries};
use crate::content::{self, Content};
use crate::engine::DT;
use crate::game::Controls;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Tab {
    Heroes,
    Shop,
    Feats,
}

impl Tab {
    pub const ALL: [Tab; 3] = [Tab::Heroes, Tab::Shop, Tab::Feats];

    pub fn label(self) -> &'static str {
        match self {
            Tab::Heroes => "HEROES",
            Tab::Shop => "SHOP",
            Tab::Feats => "FEATS",
        }
    }
}

pub enum HubAction {
    None,
    Play(usize),
    Back,
    /// A purchase changed the save; carries the feats it completed.
    Bought(Vec<usize>),
}

/// Ignore confirm this long after opening, so a held Space does not start a run.
const GUARD: f32 = 0.25;
pub const FLASH: f32 = 0.4;

pub struct Hub {
    pub tab: Tab,
    pub on_tabs: bool,
    pub hero: usize,
    pub shop: usize,
    pub feat: usize,
    pub age: f32,
    /// Countdown for the "not enough bones" flash.
    pub deny: f32,
    /// Countdown for the purchase flash.
    pub bought: f32,
}

/// Grid cell of each shop entry: upgrades first, items start on a new row.
pub fn shop_layout(entries: &[Buy], cols: usize) -> Vec<(usize, usize)> {
    let cols = cols.max(1);
    let mut out = Vec::with_capacity(entries.len());
    let (mut row, mut col) = (0, 0);
    for (i, b) in entries.iter().enumerate() {
        let new_section = i > 0 && std::mem::discriminant(b) != std::mem::discriminant(&entries[i - 1]);
        if col == cols || (new_section && col > 0) {
            row += 1;
            col = 0;
        }
        out.push((row, col));
        col += 1;
    }
    out
}

impl Hub {
    pub fn new(hero: usize) -> Self {
        Hub { tab: Tab::Heroes, on_tabs: false, hero, shop: 0, feat: 0, age: 0.0, deny: 0.0, bought: 0.0 }
    }

    /// `cols` is the shop grid width the screen will draw.
    pub fn update(&mut self, c: &Controls, save: &mut Save, cols: usize) -> HubAction {
        let content = content::get();
        self.age += DT;
        self.deny = (self.deny - DT).max(0.0);
        self.bought = (self.bought - DT).max(0.0);
        if c.pause {
            return HubAction::Back;
        }
        if self.on_tabs {
            let i = Tab::ALL.iter().position(|&t| t == self.tab).unwrap_or(0);
            if c.left {
                self.tab = Tab::ALL[(i + Tab::ALL.len() - 1) % Tab::ALL.len()];
            } else if c.right {
                self.tab = Tab::ALL[(i + 1) % Tab::ALL.len()];
            } else if c.down || c.confirm {
                self.on_tabs = false;
            }
            return HubAction::None;
        }
        let confirm = c.confirm && self.age >= GUARD;
        match self.tab {
            Tab::Heroes => {
                let n = content.characters.len();
                if c.left {
                    self.hero = (self.hero + n - 1) % n;
                } else if c.right {
                    self.hero = (self.hero + 1) % n;
                } else if c.up {
                    self.on_tabs = true;
                } else if confirm {
                    if hero_unlocked(save, content, self.hero) {
                        return HubAction::Play(self.hero);
                    }
                    return self.buy(save, content, Buy::Hero(self.hero));
                }
            }
            Tab::Shop => {
                let entries = shop_entries(content);
                if entries.is_empty() {
                    self.on_tabs |= c.up;
                    return HubAction::None;
                }
                self.shop = self.shop.min(entries.len() - 1);
                let cells = shop_layout(&entries, cols);
                let (row, col) = cells[self.shop];
                if c.left {
                    self.shop = self.shop.saturating_sub(1);
                } else if c.right {
                    self.shop = (self.shop + 1).min(entries.len() - 1);
                } else if c.up && row == 0 {
                    self.on_tabs = true;
                } else if c.up || c.down {
                    let target = if c.up { row - 1 } else { row + 1 };
                    // Nearest column in the target row.
                    if let Some(i) = (0..cells.len())
                        .filter(|&i| cells[i].0 == target)
                        .min_by_key(|&i| cells[i].1.abs_diff(col))
                    {
                        self.shop = i;
                    }
                } else if confirm {
                    return self.buy(save, content, entries[self.shop]);
                }
            }
            Tab::Feats => {
                let n = content.meta.feats.len();
                if c.up && self.feat == 0 {
                    self.on_tabs = true;
                } else if c.up {
                    self.feat -= 1;
                } else if c.down {
                    self.feat = (self.feat + 1).min(n.saturating_sub(1));
                }
            }
        }
        HubAction::None
    }

    fn buy(&mut self, save: &mut Save, content: &Content, b: Buy) -> HubAction {
        match purchase(save, content, b) {
            Some(feats) => {
                self.bought = FLASH;
                // Unlocking a hero should not start a run on the same press.
                self.age = 0.0;
                HubAction::Bought(feats)
            }
            None => {
                self.deny = FLASH;
                HubAction::None
            }
        }
    }
}
