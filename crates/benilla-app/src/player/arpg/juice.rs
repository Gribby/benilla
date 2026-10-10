//! Fork-only, not 1.12.1: the ARPG view's kill and hit feedback. A unit the player struck that
//! dies bursts: a ring of light swells out from it and fades, white for an ordinary kill and in
//! the champion's or rare's colour for theirs. The camera kicks a little at a crit and more at a
//! kill, most at a champion's or a rare's, and settles at once; the kick never moves the aim more
//! than a fraction of a yard.
//!
//! The struck units are the floating combat text's anchors, as the hit flash's are (`fx`): a
//! damage number spawns only for the player's own and the pet's hits.

use std::f32::consts::FRAC_PI_2;

use bevy::prelude::*;

use crate::combat_text::CombatTextSpawn;
use crate::net::{Guid, ObjectStore, SelfPlayer};
use crate::player::camera::FlyCam;

use super::champions::ArpgChampions;
use super::control;

/// How long after its last hit a unit's death still counts as the player's kill, in seconds.
const KILL_WINDOW_SECS: f64 = 6.0;
/// The combat text category of a crit.
const CATEGORY_CRIT: u8 = 2;

/// The burst: its life in seconds, its ring's radii at the start and the end, in yards, its peak
/// alpha, and its lift off the ground.
const BURST_SECS: f32 = 0.42;
const BURST_START: f32 = 0.35;
const BURST_END: f32 = 2.6;
const BURST_WIDTH: f32 = 0.32;
const BURST_ALPHA: f32 = 0.85;
const BURST_LIFT: f32 = 0.08;
/// A champion's and a rare's burst is this much bigger.
const BURST_CHAMPION_SCALE: f32 = 1.6;
const KILL_RGB: [f32; 3] = [1.0, 0.92, 0.75];
const CHAMPION_RGB: [f32; 3] = [0.35, 0.6, 1.0];
const RARE_RGB: [f32; 3] = [1.0, 0.85, 0.25];

/// The camera kick, as trauma (0 to 1): what a crit, a kill and a champion's kill add.
const SHAKE_CRIT: f32 = 0.4;
const SHAKE_KILL: f32 = 0.5;
const SHAKE_CHAMPION: f32 = 0.9;
/// Trauma lost per second, and the camera's offset at full trauma, in yards. The offset goes as
/// trauma squared, so a small kick stays small.
const SHAKE_DECAY: f32 = 2.6;
const SHAKE_MAX_YARDS: f32 = 0.3;
/// The shake's speed, in radians per second of its noise.
const SHAKE_SPEED: f32 = 47.0;

/// The units the player struck, with the time of the last hit, and the dead ones that burst
/// already: a melee number lands at the swing's impact, after the death, and splash and periodic
/// numbers can land on a corpse, none of which may burst it again.
#[derive(Resource, Default)]
struct StruckUnits {
    /// The unit, the last hit's time, its damage and whether it was a crit.
    live: Vec<(Entity, f64, u32, bool)>,
    burst: std::collections::HashSet<Entity>,
}

/// The camera kick's state.
#[derive(Resource, Default)]
pub(crate) struct CameraShake {
    trauma: f32,
    /// The offset written last frame, and the translation it produced, so a frame the seat does
    /// not run takes it back off.
    applied: Option<(Vec3, Vec3)>,
}

impl CameraShake {
    /// Add a kick, capped at full.
    pub(crate) fn kick(&mut self, amount: f32) {
        self.trauma = (self.trauma + amount).min(1.0);
    }
}

/// A burst ring, its start and its size.
#[derive(Component)]
struct Burst {
    born: f64,
    scale: f32,
    material: Handle<StandardMaterial>,
    rgb: [f32; 3],
}

/// A unit the player struck died: where, and whether the blow that did it was heavy (a crit, or
/// a quarter of its health at once), which flings the corpse (`super::impact`).
#[derive(Message, Clone, Copy, Debug)]
pub(crate) struct ArpgKill {
    pub(crate) unit: Entity,
    pub(crate) at: Vec3,
    pub(crate) overkill: bool,
}

/// The damage a combat text names: its leading number.
pub(super) fn text_damage(text: &str) -> u32 {
    let digits: String = text
        .trim_start()
        .chars()
        .take_while(|c| c.is_ascii_digit())
        .collect();
    digits.parse().unwrap_or(0)
}

pub(super) fn plugin(app: &mut App) {
    app.add_message::<ArpgKill>()
        .init_resource::<StruckUnits>()
        .init_resource::<CameraShake>()
        .add_systems(Update, (note_hits, spot_kills, grow_bursts).chain())
        .add_systems(
            Update,
            shake_camera
                .in_set(benilla_world::schedule::WorldStage::Input)
                .after(control)
                .after(crate::camera_shake::apply_camera_shake)
                .run_if(not(resource_exists::<crate::run_mode::CaptureMode>))
                .in_set(crate::char_select::InWorldGated),
        );
}

/// Note each unit the player struck, and kick the camera at a crit.
fn note_hits(
    mut spawns: MessageReader<CombatTextSpawn>,
    me: Query<(), With<SelfPlayer>>,
    time: Res<Time>,
    mut struck: ResMut<StruckUnits>,
    mut shake: ResMut<CameraShake>,
) {
    let now = time.elapsed_secs_f64();
    for spawn in spawns.read() {
        if me.contains(spawn.anchor)
            || !super::fx::is_damage_number(&spawn.text)
            || struck.burst.contains(&spawn.anchor)
        {
            continue;
        }
        if spawn.category == CATEGORY_CRIT {
            shake.kick(SHAKE_CRIT);
        }
        struck.live.retain(|(unit, ..)| *unit != spawn.anchor);
        struck.live.push((
            spawn.anchor,
            now,
            text_damage(&spawn.text),
            spawn.category == CATEGORY_CRIT,
        ));
    }
    struck
        .live
        .retain(|(_, at, ..)| now - at < KILL_WINDOW_SECS);
}

/// The burst's colour and size for a kill of a unit of champion tier `tier` (0 for none).
fn burst_look(tier: u8) -> ([f32; 3], f32, f32) {
    match tier {
        0 => (KILL_RGB, 1.0, SHAKE_KILL),
        1 => (CHAMPION_RGB, BURST_CHAMPION_SCALE, SHAKE_CHAMPION),
        _ => (RARE_RGB, BURST_CHAMPION_SCALE, SHAKE_CHAMPION),
    }
}

/// A struck unit that is now dead bursts, and kicks the camera.
#[allow(clippy::too_many_arguments)]
fn spot_kills(
    mut commands: Commands,
    time: Res<Time>,
    mut struck: ResMut<StruckUnits>,
    units: Query<(&Transform, &ObjectStore, Option<&Guid>)>,
    champions: Res<ArpgChampions>,
    mut shake: ResMut<CameraShake>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut mesh: Local<Option<Handle<Mesh>>>,
    mut kills: MessageWriter<ArpgKill>,
) {
    let now = time.elapsed_secs_f64();
    let mut killed = Vec::new();
    struck
        .live
        .retain(|(unit, _, damage, crit)| match units.get(*unit) {
            Ok((at, store, guid)) if store.0.unit_health() == Some(0) => {
                let tier = guid.and_then(|g| champions.tier_of(g.0)).unwrap_or(0);
                let max = store.0.unit_max_health().unwrap_or(0);
                let overkill = *crit || (max > 0 && damage.saturating_mul(4) >= max);
                kills.write(ArpgKill {
                    unit: *unit,
                    at: at.translation,
                    overkill,
                });
                killed.push((*unit, at.translation, tier));
                false
            }
            Ok(_) => true,
            Err(_) => false,
        });
    // A corpse that is gone, or a unit alive again (a respawn reusing the entity), may burst anew.
    struck.burst.retain(|unit| {
        units
            .get(*unit)
            .is_ok_and(|(_, store, _)| store.0.unit_health() == Some(0))
    });
    for (unit, at, tier) in killed {
        struck.burst.insert(unit);
        let (rgb, scale, kick) = burst_look(tier);
        shake.kick(kick);
        let mesh = mesh
            .get_or_insert_with(|| meshes.add(Annulus::new(1.0 - BURST_WIDTH, 1.0)))
            .clone();
        let material = materials.add(StandardMaterial {
            base_color: Color::srgba(rgb[0], rgb[1], rgb[2], BURST_ALPHA),
            unlit: true,
            alpha_mode: AlphaMode::Add,
            cull_mode: None,
            ..default()
        });
        commands.spawn((
            Mesh3d(mesh),
            MeshMaterial3d(material.clone()),
            Transform::from_translation(at + Vec3::Y * BURST_LIFT)
                .with_rotation(Quat::from_rotation_x(-FRAC_PI_2))
                .with_scale(Vec3::splat(BURST_START * scale)),
            Burst {
                born: now,
                scale,
                material,
                rgb,
            },
        ));
    }
}

/// A burst `age` seconds old: its radius share (0 to 1, easing out) and its alpha share.
fn burst_at(age: f32) -> Option<(f32, f32)> {
    if !(0.0..BURST_SECS).contains(&age) {
        return None;
    }
    let t = age / BURST_SECS;
    let grow = 1.0 - (1.0 - t).powi(3);
    Some((grow, (1.0 - t).powi(2)))
}

/// Swell and fade the bursts, and take the spent ones away.
fn grow_bursts(
    mut commands: Commands,
    time: Res<Time>,
    mut bursts: Query<(Entity, &Burst, &mut Transform)>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let now = time.elapsed_secs_f64();
    for (entity, burst, mut tf) in &mut bursts {
        let Some((grow, fade)) = burst_at((now - burst.born) as f32) else {
            materials.remove(&burst.material);
            commands.entity(entity).despawn();
            continue;
        };
        let radius = (BURST_START + (BURST_END - BURST_START) * grow) * burst.scale;
        tf.scale = Vec3::splat(radius);
        if let Some(mat) = materials.get_mut(&burst.material) {
            let [r, g, b] = burst.rgb;
            mat.base_color = Color::srgba(r, g, b, BURST_ALPHA * fade);
        }
    }
}

/// The shake's offset at trauma `trauma` and noise time `t`: three unrelated sines per axis, so it
/// never reads as a wobble, scaled by trauma squared.
fn shake_offset(trauma: f32, t: f32) -> Vec3 {
    let amp = SHAKE_MAX_YARDS * trauma * trauma;
    let n = |a: f32, b: f32, c: f32| {
        ((t * a).sin() + (t * b * 1.31).sin() * 0.6 + (t * c * 0.73).sin() * 0.4) / 2.0
    };
    Vec3::new(
        n(SHAKE_SPEED, 0.9 * SHAKE_SPEED, 1.7 * SHAKE_SPEED),
        n(1.13 * SHAKE_SPEED, 0.7 * SHAKE_SPEED, 1.3 * SHAKE_SPEED) * 0.6,
        n(0.87 * SHAKE_SPEED, 1.21 * SHAKE_SPEED, 0.6 * SHAKE_SPEED),
    ) * amp
}

/// Offset the camera by the shake, after the seat wrote it this frame.
fn shake_camera(
    time: Res<Time>,
    mut shake: ResMut<CameraShake>,
    mut cameras: Query<&mut Transform, (With<Camera>, With<FlyCam>)>,
) {
    let Ok(mut tf) = cameras.single_mut() else {
        return;
    };
    // The seat did not run (or wrote the same pose): take last frame's kick back off.
    if let Some((offset, written)) = shake.applied.take() {
        if tf.translation == written {
            tf.translation -= offset;
        }
    }
    if shake.trauma <= 0.0 {
        return;
    }
    let offset = shake_offset(shake.trauma, time.elapsed_secs());
    shake.trauma = (shake.trauma - SHAKE_DECAY * time.delta_secs()).max(0.0);
    tf.translation += offset;
    shake.applied = Some((offset, tf.translation));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_burst_swells_and_fades_then_ends() {
        let (g0, a0) = burst_at(0.0).unwrap();
        assert_eq!((g0, a0), (0.0, 1.0));
        let (g1, a1) = burst_at(BURST_SECS * 0.5).unwrap();
        assert!(g1 > 0.5 && a1 < 0.5);
        assert!(burst_at(BURST_SECS).is_none());
        assert!(burst_at(-0.01).is_none());
    }

    #[test]
    fn the_shake_stays_within_its_reach_and_a_small_kick_stays_small() {
        for i in 0..500 {
            let t = i as f32 * 0.013;
            assert!(shake_offset(1.0, t).length() <= SHAKE_MAX_YARDS * 1.8);
            assert!(shake_offset(SHAKE_CRIT, t).length() <= SHAKE_MAX_YARDS * 0.3);
        }
        assert_eq!(shake_offset(0.0, 1.0), Vec3::ZERO);
    }

    #[test]
    fn champions_burst_bigger_and_kick_harder() {
        let (_, plain, kick) = burst_look(0);
        let (rgb, champ, champ_kick) = burst_look(1);
        let (rare_rgb, _, _) = burst_look(2);
        assert!(champ > plain && champ_kick > kick);
        assert_ne!(rgb, rare_rgb);
        let mut shake = CameraShake::default();
        shake.kick(0.8);
        shake.kick(0.8);
        assert_eq!(shake.trauma, 1.0);
    }
}
