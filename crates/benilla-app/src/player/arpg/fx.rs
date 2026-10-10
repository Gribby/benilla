//! Fork-only, not 1.12.1: the ARPG view's hit flash. A unit the player (or the player's pet)
//! damages washes toward white for a moment, so a swing or a skillshot that lands reads at a
//! glance. The trigger is the floating combat text itself: a damage number spawns only for the
//! player's own and the pet's hits (`combat_text`'s source gates), so its anchor is the struck
//! unit. A miss word and the XP text do not flash.
//!
//! The flash rides the body tint table ([`benilla_world::instance_tint::with_flash`]): after the
//! aura tint publishes each rig's word, every rig under a flashing unit, worn gear included, gets
//! the flash folded into its word's spare alpha byte. The aura writer rewrites every rig each
//! frame, so a flash that ends needs no cleanup.

use bevy::prelude::*;

use benilla_world::instance_tint::{with_flash, InstanceTints};
use benilla_world::model_fade::{ParentModel, MAX_MODEL_CHAIN};
use benilla_world::rig_palette::RigSkin;

use crate::combat_text::CombatTextSpawn;
use crate::net::SelfPlayer;

/// How long a flash lasts, in seconds.
const FLASH_SECS: f32 = 0.16;
/// The white wash at the hit, for a normal hit and for a crit (category 2).
const FLASH_PEAK: f32 = 0.5;
const CRIT_PEAK: f32 = 0.8;
/// The combat text category of a crit.
const CATEGORY_CRIT: u8 = 2;

/// The live flashes: the struck unit, when it was struck, and the peak.
#[derive(Resource, Default)]
struct HitFlashes(Vec<(Entity, f64, f32)>);

pub(super) fn plugin(app: &mut App) {
    app.init_resource::<HitFlashes>().add_systems(
        Update,
        (collect_hits, apply_hit_flash)
            .chain()
            .after(crate::aura_visual::apply_aura_tint),
    );
}

/// The wash `age` seconds after a hit with this peak: a linear fall to nothing.
fn flash_at(age: f32, peak: f32) -> f32 {
    if !(0.0..FLASH_SECS).contains(&age) {
        return 0.0;
    }
    peak * (1.0 - age / FLASH_SECS)
}

/// Whether a combat text is a damage number: it starts with a digit, where a miss, a dodge or an
/// immune is a word.
pub(super) fn is_damage_number(text: &str) -> bool {
    text.trim_start().starts_with(|c: char| c.is_ascii_digit())
}

/// Start a flash for each damage number over another unit, and drop the spent ones.
fn collect_hits(
    mut spawns: MessageReader<CombatTextSpawn>,
    me: Query<(), With<SelfPlayer>>,
    time: Res<Time>,
    mut flashes: ResMut<HitFlashes>,
) {
    let now = time.elapsed_secs_f64();
    for spawn in spawns.read() {
        if me.contains(spawn.anchor) || !is_damage_number(&spawn.text) {
            continue;
        }
        let peak = if spawn.category == CATEGORY_CRIT {
            CRIT_PEAK
        } else {
            FLASH_PEAK
        };
        flashes.0.retain(|(unit, _, _)| *unit != spawn.anchor);
        flashes.0.push((spawn.anchor, now, peak));
    }
    flashes
        .0
        .retain(|(_, at, _)| ((now - at) as f32) < FLASH_SECS);
}

/// Fold the live flashes into the tint word of every rig under a flashing unit. A rig belongs to
/// a unit through its [`ParentModel`] chain (worn gear, a mount's rider) or the scene hierarchy.
fn apply_hit_flash(
    flashes: Res<HitFlashes>,
    time: Res<Time>,
    rigs: Query<(Entity, &RigSkin)>,
    links: Query<(Option<&ParentModel>, Option<&ChildOf>)>,
    mut tints: ResMut<InstanceTints>,
) {
    if flashes.0.is_empty() {
        return;
    }
    let now = time.elapsed_secs_f64();
    for (entity, rig) in &rigs {
        let Some(flash) = owner_flash(entity, &flashes.0, &links, now) else {
            continue;
        };
        let word = with_flash(tints.get(rig.slot), flash);
        tints.set(rig.slot, word);
    }
}

/// The flash of the first flashing unit at or above `entity`, walking the attachment chain and
/// then the scene parents, bounded as [`MAX_MODEL_CHAIN`] bounds a model walk.
fn owner_flash(
    entity: Entity,
    flashes: &[(Entity, f64, f32)],
    links: &Query<(Option<&ParentModel>, Option<&ChildOf>)>,
    now: f64,
) -> Option<f32> {
    let mut at = entity;
    for _ in 0..MAX_MODEL_CHAIN * 2 {
        if let Some((_, struck, peak)) = flashes.iter().find(|(unit, _, _)| *unit == at) {
            return Some(flash_at((now - struck) as f32, *peak)).filter(|f| *f > 0.0);
        }
        let (model, child) = links.get(at).ok()?;
        at = match (model, child) {
            (Some(model), _) => model.0,
            (None, Some(child)) => child.parent(),
            (None, None) => return None,
        };
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_flash_falls_from_its_peak_to_nothing() {
        assert_eq!(flash_at(0.0, 0.5), 0.5);
        assert!((flash_at(FLASH_SECS / 2.0, 0.5) - 0.25).abs() < 1e-6);
        assert_eq!(flash_at(FLASH_SECS, 0.5), 0.0);
        assert_eq!(flash_at(-0.1, 0.5), 0.0);
    }

    #[test]
    fn only_numbers_flash() {
        assert!(is_damage_number("123"));
        assert!(is_damage_number("45 (12 absorbed)"));
        assert!(!is_damage_number("Miss"));
        assert!(!is_damage_number("Dodge"));
        assert!(!is_damage_number(""));
    }
}
