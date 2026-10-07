//! Fork-only, not 1.12.1: the ARPG view's soft target (`WOW_ARPG`, [`crate::player`]'s `arpg`).
//! The enemy under the cursor becomes the selection, so every spell, ability and swing goes at
//! what the player points at through the stock cast and attack paths, with no tab-targeting. A
//! cursor on open ground near an enemy snaps to it (the magnet), and moving off every enemy keeps
//! the last one. [`plugin`] is called only while the view is on, so stock benilla never runs it.
//!
//! `WOW_ARPG_MAGNET` (yards, default 3, 0 for none) sizes the snap; `WOW_ARPG_SOFT_TARGET=0` keeps
//! the magnet and the hover but leaves the selection to clicks.

use bevy::prelude::*;

use crate::net::{Guid, NetEntity, ObjectStore, SelfPlayer};

use super::{Hovered, HoveredObject, PickOcclusion, ReactionInputs, SelectCommit, TargetUpdate};

/// The facing the ARPG view turns an idle character to: the selection's feet, while it is a live
/// enemy. Written by [`soft_target`], read by the player's controller a frame later.
#[derive(Resource, Default, Clone, Copy)]
pub(crate) struct ArpgFacing(pub(crate) Option<Vec3>);

/// The magnet's reach and whether the hover selects, read once from the environment.
#[derive(Resource, Clone, Copy)]
struct SoftTargetConfig {
    magnet: f32,
    selects: bool,
}

impl SoftTargetConfig {
    fn from_env() -> Self {
        let magnet = std::env::var("WOW_ARPG_MAGNET")
            .ok()
            .and_then(|v| v.trim().parse::<f32>().ok())
            .filter(|v| v.is_finite())
            .unwrap_or(3.0)
            .clamp(0.0, 15.0);
        let selects = !std::env::var("WOW_ARPG_SOFT_TARGET")
            .is_ok_and(|v| matches!(v.trim(), "0" | "off"));
        Self { magnet, selects }
    }
}

/// Add the soft target between the pick and the cursor, so this frame's cursor, clicks and casts
/// all read the snapped hover and the new selection.
pub(crate) fn plugin(app: &mut App) {
    app.insert_resource(SoftTargetConfig::from_env())
        .init_resource::<ArpgFacing>()
        .add_systems(
            Update,
            soft_target
                .in_set(TargetUpdate)
                .after(super::hover::update_hovered_object)
                .before(super::cursor_mode::classify_cursor),
        );
}

/// The candidate the magnet snaps to: `current` while it is still inside `reach` of `point` on
/// the ground, so a cursor between two enemies does not flip between them, else the nearest.
fn magnet_pick<T: PartialEq>(
    point: Vec3,
    reach: f32,
    current: Option<T>,
    candidates: impl Iterator<Item = (T, Vec3)>,
) -> Option<T> {
    let mut nearest: Option<(T, f32)> = None;
    for (item, at) in candidates {
        let d = Vec2::new(at.x - point.x, at.z - point.z).length();
        if d > reach {
            continue;
        }
        if current.as_ref() == Some(&item) {
            return Some(item);
        }
        if nearest.as_ref().is_none_or(|(_, best)| d < *best) {
            nearest = Some((item, d));
        }
    }
    nearest.map(|(item, _)| item)
}

/// Snap the hover to the enemy nearest the cursor when it lies on open ground, select the hovered
/// enemy, and publish the facing. A cursor over a UI panel neither snaps nor selects, as the pick
/// itself yields to the UI. While swinging, only an enemy truly under the cursor (not a snap)
/// switches the target, since a switch stops and restarts the swing.
#[allow(clippy::type_complexity)]
fn soft_target(
    config: Res<SoftTargetConfig>,
    // UI chrome under the pointer, nameplates excepted: a plate hover is an enemy hover.
    over_panel: Res<crate::ui_script::PointerOverUiPanel>,
    mut hovered: ResMut<Hovered>,
    object: Res<HoveredObject>,
    occlusion: Res<PickOcclusion>,
    spell_targeting: Option<Res<crate::spell::targeting::SpellTargeting>>,
    units: Query<
        (Entity, &Guid, &Transform, Option<&ObjectStore>),
        (With<NetEntity>, Without<SelfPlayer>),
    >,
    me: Query<Option<&ObjectStore>, With<SelfPlayer>>,
    reaction: ReactionInputs,
    mut select: SelectCommit,
    mut facing: ResMut<ArpgFacing>,
) {
    let self_store = me.single().ok().flatten();
    let live_enemy = |store: Option<&ObjectStore>| {
        store.is_some_and(ObjectStore::is_unit)
            && super::scan::attack_target_valid(
                store,
                reaction.factions.as_deref(),
                &reaction.reputations,
                self_store,
            )
    };
    // The magnet: only on open ground in the world, never over the UI, a GameObject, a corpse or
    // a refused pick, and never while a spell is choosing its target, whose word filters the hover.
    let targeting = spell_targeting.as_ref().is_some_and(|t| t.active());
    let open_ground = hovered.any().is_none() && !hovered.refused && object.target.is_none();
    let mut snapped = false;
    if config.magnet > 0.0 && open_ground && !targeting && !over_panel.0 {
        if let Some(point) = occlusion.point {
            let candidates = units
                .iter()
                .filter(|(_, _, _, store)| live_enemy(*store))
                .map(|(e, g, t, _)| ((e, g.0), t.translation));
            let current = select.selection.target.zip(select.selection.guid);
            if let Some((entity, guid)) = magnet_pick(point, config.magnet, current, candidates) {
                hovered.target = Some(entity);
                hovered.guid = Some(guid);
                hovered.distance = occlusion.distance;
                snapped = true;
            }
        }
    }
    // The soft select: a live enemy under the cursor becomes the target; anything else leaves it.
    let may_switch = !snapped || !select.engaged();
    if config.selects && !targeting && !over_panel.0 && may_switch {
        if let Some((entity, guid)) = hovered.mouseover(&object) {
            let enemy = units
                .get(entity)
                .is_ok_and(|(_, _, _, store)| live_enemy(store));
            if enemy && select.selection.guid != Some(guid) {
                select.commit(entity, guid);
            }
        }
    }
    // Face the selection while it is a live enemy.
    let target = select.selection.target.and_then(|e| units.get(e).ok());
    facing.0 = target
        .filter(|(_, _, _, store)| live_enemy(*store))
        .map(|(_, _, t, _)| t.translation);
}

#[cfg(test)]
mod tests {
    use super::*;

    const CANDIDATES: [(i32, Vec3); 3] = [
        (1, Vec3::new(12.0, 0.0, 10.0)),
        (2, Vec3::new(10.0, 40.0, 11.0)),
        (3, Vec3::new(30.0, 0.0, 30.0)),
    ];
    const POINT: Vec3 = Vec3::new(10.0, 5.0, 10.0);

    #[test]
    fn the_magnet_takes_the_nearest_inside_its_reach() {
        // Height is ignored: the unit 35 yd above is 1 yd away on the ground.
        assert_eq!(magnet_pick(POINT, 3.0, None, CANDIDATES.into_iter()), Some(2));
        assert_eq!(magnet_pick(POINT, 0.5, None, CANDIDATES.into_iter()), None);
    }

    #[test]
    fn the_magnet_keeps_the_current_target_while_it_is_in_reach() {
        assert_eq!(magnet_pick(POINT, 3.0, Some(1), CANDIDATES.into_iter()), Some(1));
        // Out of reach, the current target is not kept.
        assert_eq!(magnet_pick(POINT, 3.0, Some(3), CANDIDATES.into_iter()), Some(2));
    }
}
