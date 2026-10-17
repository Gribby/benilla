//! Fork-only, not 1.12.1: the ARPG view's feeds, which hand the HUD addon (`arpg_hud.lua`) what
//! `crate::player::arpg` holds. One file per source module there; the systems are registered by
//! that module's plugin, where they always were.

pub(crate) mod actions;
pub(crate) mod champions;
pub(crate) mod feel;
#[cfg(test)]
mod hud_tests;
pub(crate) mod sheet;
pub(crate) mod tree;
pub(crate) mod world;
