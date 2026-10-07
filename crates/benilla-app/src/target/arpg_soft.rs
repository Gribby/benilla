//! Fork-only, not 1.12.1: the ARPG view's hover (`WOW_ARPG`, [`crate::player`]'s `arpg`). There
//! is no selection in the ARPG view: swings and casts go to an ARPG server with an aim point, and
//! the server picks who they hit. This publishes the live enemy under the cursor ([`ArpgEnemyHover`])
//! for the swing input and the hover health bar, and snaps a cursor on open ground near an enemy
//! to it (the magnet), so the highlight, the cursor and the bar find it without pixel aim.
//! [`plugin`] is called only while the view is on, so stock benilla never runs it.
//!
//! `WOW_ARPG_MAGNET` (yards, default 3, 0 for none) sizes the snap.

use bevy::prelude::*;

use crate::net::{Guid, NetEntity, ObjectStore, SelfPlayer};

use super::{Hovered, HoveredObject, PickOcclusion, ReactionInputs, TargetUpdate};

/// The live enemy under the cursor this frame, snapped or not, and its guid; `None` over anything
/// else.
#[derive(Resource, Default, Clone, Copy)]
pub(crate) struct ArpgEnemyHover(pub(crate) Option<(Entity, u64)>);

/// The magnet's reach, read once from the environment.
#[derive(Resource, Clone, Copy)]
struct SoftTargetConfig {
    magnet: f32,
}

impl SoftTargetConfig {
    fn from_env() -> Self {
        let magnet = std::env::var("WOW_ARPG_MAGNET")
            .ok()
            .and_then(|v| v.trim().parse::<f32>().ok())
            .filter(|v| v.is_finite())
            .unwrap_or(3.0)
            .clamp(0.0, 15.0);
        Self { magnet }
    }
}

/// Add the hover between the pick and the cursor, so this frame's cursor and highlight read the
/// snapped hover.
pub(crate) fn plugin(app: &mut App) {
    app.insert_resource(SoftTargetConfig::from_env())
        .init_resource::<ArpgEnemyHover>()
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

/// Snap the hover to the enemy nearest the cursor when it lies on open ground, and publish the
/// live enemy under the cursor. A cursor over a UI panel neither snaps nor hovers, as the pick
/// itself yields to the UI. The last enemy hovered holds the snap while it stays in reach, so a
/// cursor between two enemies does not flip between them.
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
    mut enemy_hover: ResMut<ArpgEnemyHover>,
    // The last enemy hovered, which holds the magnet.
    mut last: Local<Option<(Entity, u64)>>,
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
    if config.magnet > 0.0 && open_ground && !targeting && !over_panel.0 {
        if let Some(point) = occlusion.point {
            let candidates = units
                .iter()
                .filter(|(_, _, _, store)| live_enemy(*store))
                .map(|(e, g, t, _)| ((e, g.0), t.translation));
            if let Some((entity, guid)) = magnet_pick(point, config.magnet, *last, candidates) {
                hovered.target = Some(entity);
                hovered.guid = Some(guid);
                hovered.distance = occlusion.distance;
            }
        }
    }
    let enemy = hovered
        .mouseover(&object)
        .filter(|_| !over_panel.0)
        .filter(|(entity, _)| {
            units
                .get(*entity)
                .is_ok_and(|(_, _, _, store)| live_enemy(store))
        });
    if enemy.is_some() {
        *last = enemy;
    }
    enemy_hover.0 = enemy;
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
        assert_eq!(
            magnet_pick(POINT, 3.0, None, CANDIDATES.into_iter()),
            Some(2)
        );
        assert_eq!(magnet_pick(POINT, 0.5, None, CANDIDATES.into_iter()), None);
    }

    #[test]
    fn the_magnet_keeps_the_current_target_while_it_is_in_reach() {
        assert_eq!(
            magnet_pick(POINT, 3.0, Some(1), CANDIDATES.into_iter()),
            Some(1)
        );
        // Out of reach, the current target is not kept.
        assert_eq!(
            magnet_pick(POINT, 3.0, Some(3), CANDIDATES.into_iter()),
            Some(2)
        );
    }
}
