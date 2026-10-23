//! Fork-only, not 1.12.1: a player's summoned object (an ARPG town portal's two ends, cmangos
//! `Arpg/ArpgActions.h`) grows in from nothing when it first shows, over [`GROW_SECS`] with a
//! slight overshoot, instead of popping into being.

use benilla_protocol::EntityKind;
use bevy::prelude::*;

use crate::net::{NetEntity, ObjectStore};

/// How long an object takes to grow in.
const GROW_SECS: f32 = 0.8;
/// The overshoot of the ease (`easeOutBack`'s constant): a touch past full size, then settling.
const OVERSHOOT: f32 = 1.4;

/// An object growing in: time so far. It grows to the server's scale, read each frame, since the
/// visuals' attach sets the entity's scale only once the model loads.
#[derive(Component)]
struct GrowIn {
    elapsed: f32,
}

pub(super) fn plugin(app: &mut App) {
    // After the visuals' attach, so its scale write never shows a frame at full size.
    app.add_systems(
        Update,
        (arm_grow_in, grow_in)
            .chain()
            .after(crate::entities::EntityVisualsSet),
    );
}

/// `easeOutBack` over 0..1: 0 at 0, 1 at 1, a little past 1 on the way.
fn ease_out_back(t: f32) -> f32 {
    let t = t.clamp(0.0, 1.0) - 1.0;
    1.0 + (OVERSHOOT + 1.0) * t * t * t + OVERSHOOT * t * t
}

/// A game object a player made, seen for the first time: it grows in.
fn arm_grow_in(
    mut commands: Commands,
    mut fresh: Query<(Entity, &NetEntity, &ObjectStore, &mut Transform), Added<NetEntity>>,
) {
    for (entity, net, store, mut tf) in &mut fresh {
        if net.kind != EntityKind::GameObject || store.0.gameobject_created_by().is_none() {
            continue;
        }
        tf.scale = Vec3::splat(net.scale * 0.01);
        commands.entity(entity).insert(GrowIn { elapsed: 0.0 });
    }
}

fn grow_in(
    mut commands: Commands,
    time: Res<Time>,
    mut growing: Query<(Entity, &NetEntity, &mut GrowIn, &mut Transform)>,
) {
    for (entity, net, mut grow, mut tf) in &mut growing {
        grow.elapsed += time.delta_secs();
        let t = grow.elapsed / GROW_SECS;
        let target = Vec3::splat(net.scale);
        if t >= 1.0 {
            tf.scale = target;
            commands.entity(entity).remove::<GrowIn>();
        } else {
            tf.scale = target * ease_out_back(t).max(0.01);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_grow_starts_small_overshoots_and_settles_at_full_size() {
        assert!(ease_out_back(0.0).abs() < 1e-6);
        assert!((ease_out_back(1.0) - 1.0).abs() < 1e-6);
        let peak = (1..100)
            .map(|i| ease_out_back(i as f32 / 100.0))
            .fold(0.0f32, f32::max);
        assert!(peak > 1.02 && peak < 1.2, "a slight overshoot: {peak}");
    }
}
