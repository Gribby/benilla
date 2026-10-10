//! Fork-only, not 1.12.1: the ARPG view's weight of a blow. A crit or a heavy hit (a sixth of the
//! victim's health) by the player stops both models' animations for a few frames (hit-stop); any
//! hit pushes the struck model back along the blow for a moment (stagger); a kill by a heavy blow
//! flings the corpse a couple of yards on a short arc; and a kill leaves the creature's blood on
//! the ground, the splat textures 1.12 ships and never draws, fading over half a minute.
//!
//! All of it is drawing only: the server's positions never move. The push and the fling offset
//! the rig's root bones in [`benilla_world::rig_anim::PosePost`], the window the strafe twist
//! uses (`crate::creature_anim::twist`), so held items, emitters and picking follow the model.
//! The blows are the player's floating combat text, as the hit flash's are (`super::fx`); the
//! kills are `super::juice`'s.

use std::collections::{HashMap, VecDeque};

use bevy::prelude::*;

use benilla_world::decal::{DecalFrame, WorldDecal};
use benilla_world::particles::buffer::{begin_effect_frame, EffectVertex};
use benilla_world::rig_anim::{AnimParked, PosePost, RigPose};
use benilla_world::view::WorldCamera;

use crate::aura_visual::AnimRateFreeze;
use crate::combat_text::CombatTextSpawn;
use crate::net::{NetEntity, ObjectStore, SelfPlayer};

use super::juice::{text_damage, ArpgKill};

/// The combat text category of a crit.
const CATEGORY_CRIT: u8 = 2;
/// A blow this share of the victim's maximum health or more is heavy.
const HEAVY_SHARE: f32 = 1.0 / 6.0;
/// Hit-stop: how long the models hold, in seconds, for a crit and for another heavy blow.
const STOP_CRIT: f64 = 0.075;
const STOP_HEAVY: f64 = 0.05;
/// Stagger: the push in yards (a crit's, an ordinary hit's), and how long it takes to settle.
const PUSH_CRIT: f32 = 0.45;
const PUSH_HIT: f32 = 0.2;
const PUSH_SECS: f32 = 0.16;
/// The fling: how far, how high its arc, and how long it flies.
const FLING_REACH: f32 = 2.4;
const FLING_HEIGHT: f32 = 0.7;
const FLING_SECS: f32 = 0.35;
/// The splats: their size in yards (a unit of scale 1), how long they lie, and the last share of
/// that over which they fade; at most this many at once.
const SPLAT_SIZE: f32 = 2.2;
const SPLAT_LIFE: f32 = 30.0;
const SPLAT_FADE: f32 = 8.0;
const SPLAT_CAP: usize = 96;
const SPLAT_ALPHA: f32 = 0.85;
/// The splat projector's slab, above and below the corpse's feet.
const SPLAT_SLAB: f32 = 2.0;

/// The rigs held still, until when.
#[derive(Resource, Default)]
struct HitStops(HashMap<Entity, f64>);

/// A model being pushed or flung, and the root-bone bookkeeping that keeps the offset from
/// building up on bones the clip does not key every frame.
struct Recoil {
    /// The blow's direction on the ground, in the world.
    dir: Vec3,
    born: f64,
    kind: RecoilKind,
    /// Per root bone: its index, the animated translation the offset was laid on, and the
    /// translation written.
    bones: Vec<(usize, Vec3, Vec3)>,
}

#[derive(Clone, Copy, PartialEq, Debug)]
enum RecoilKind {
    /// A push of this many yards, settling.
    Stagger(f32),
    /// A corpse flung, held where it lands while it is dead.
    Fling,
}

#[derive(Resource, Default)]
struct Recoils(HashMap<Entity, Recoil>);

/// One splat on the ground.
struct Splat {
    verts: Vec<EffectVertex>,
    born: f32,
    texture: AssetId<Image>,
    anchor: Vec3,
}

#[derive(Resource, Default)]
struct Splats(VecDeque<Splat>);

/// The entity the splat draws name as their owner.
#[derive(Resource)]
struct SplatLane(Entity);

/// Splat textures by path, loaded on first use.
#[derive(Resource, Default)]
struct SplatTextures(HashMap<String, Handle<Image>>);

pub(super) fn plugin(app: &mut App) {
    app.init_resource::<HitStops>()
        .init_resource::<Recoils>()
        .init_resource::<Splats>()
        .init_resource::<SplatTextures>()
        .add_systems(Startup, spawn_lane)
        .add_systems(
            Update,
            (
                (feel_hits, feel_kills).chain(),
                hold_hit_stops.after(crate::aura_visual::apply_aura_anim_rate),
            ),
        )
        .add_systems(PostUpdate, apply_recoil.in_set(PosePost))
        .add_systems(PostUpdate, draw_splats.after(begin_effect_frame));
}

fn spawn_lane(mut commands: Commands) {
    let lane = commands.spawn(Name::new("arpg-splat-lane")).id();
    commands.insert_resource(SplatLane(lane));
}

/// How long a blow holds the models, or `None` for a light one.
fn hit_stop_for(damage: u32, max_health: u32, crit: bool) -> Option<f64> {
    if crit {
        return Some(STOP_CRIT);
    }
    (max_health > 0 && damage as f32 >= max_health as f32 * HEAVY_SHARE).then_some(STOP_HEAVY)
}

/// The flat direction from `from` to `to`, or the given fallback.
fn flat_dir(from: Vec3, to: Vec3) -> Vec3 {
    Vec3::new(to.x - from.x, 0.0, to.z - from.z).normalize_or(Vec3::NEG_Z)
}

/// Each blow the player landed: a push, and for a heavy one a hit-stop on both models.
fn feel_hits(
    mut spawns: MessageReader<CombatTextSpawn>,
    me: Query<(Entity, &Transform), With<SelfPlayer>>,
    units: Query<(&Transform, &ObjectStore)>,
    time: Res<Time>,
    mut stops: ResMut<HitStops>,
    mut recoils: ResMut<Recoils>,
) {
    let now = time.elapsed_secs_f64();
    let Ok((me, my_tf)) = me.single() else {
        for _ in spawns.read() {}
        return;
    };
    for spawn in spawns.read() {
        if spawn.anchor == me || !super::fx::is_damage_number(&spawn.text) {
            continue;
        }
        let Ok((tf, store)) = units.get(spawn.anchor) else {
            continue;
        };
        let crit = spawn.category == CATEGORY_CRIT;
        let damage = text_damage(&spawn.text);
        let max = store.0.unit_max_health().unwrap_or(0);
        if let Some(hold) = hit_stop_for(damage, max, crit) {
            for unit in [spawn.anchor, me] {
                let until = stops.0.entry(unit).or_insert(0.0);
                *until = until.max(now + hold);
            }
        }
        // A corpse already flung keeps its flight.
        if recoils
            .0
            .get(&spawn.anchor)
            .is_some_and(|r| r.kind == RecoilKind::Fling)
        {
            continue;
        }
        let push = if crit { PUSH_CRIT } else { PUSH_HIT };
        let bones = recoils
            .0
            .remove(&spawn.anchor)
            .map(|r| r.bones)
            .unwrap_or_default();
        recoils.0.insert(
            spawn.anchor,
            Recoil {
                dir: flat_dir(my_tf.translation, tf.translation),
                born: now,
                kind: RecoilKind::Stagger(push),
                bones,
            },
        );
    }
}

/// Each kill: the corpse's splat, and a heavy blow's fling.
#[allow(clippy::too_many_arguments)]
fn feel_kills(
    mut kills: MessageReader<ArpgKill>,
    me: Query<&Transform, With<SelfPlayer>>,
    nets: Query<&NetEntity>,
    time: Res<Time>,
    mut recoils: ResMut<Recoils>,
    mut splats: ResMut<Splats>,
    mut textures: ResMut<SplatTextures>,
    blood: Option<Res<crate::creature_anim::BloodTables>>,
    creatures: Option<Res<crate::entities::Creatures>>,
    asset_server: Res<AssetServer>,
    decals: WorldDecal,
) {
    let now = time.elapsed_secs_f64();
    let my_pos = me.single().ok().map(|t| t.translation);
    for kill in kills.read() {
        let dir = my_pos.map_or(Vec3::NEG_Z, |p| flat_dir(p, kill.at));
        if kill.overkill {
            let bones = recoils
                .0
                .remove(&kill.unit)
                .map(|r| r.bones)
                .unwrap_or_default();
            recoils.0.insert(
                kill.unit,
                Recoil {
                    dir,
                    born: now,
                    kind: RecoilKind::Fling,
                    bones,
                },
            );
        }
        // The blood: the creature's own splat texture, turned at random, under where it fell.
        let (Some(blood), Some(creatures)) = (blood.as_deref(), creatures.as_deref()) else {
            continue;
        };
        let Some(net) = nets.get(kill.unit).ok() else {
            continue;
        };
        let Some(display) = net.display_id else {
            continue;
        };
        let paths = crate::creature_anim::kill_splats(blood, creatures, display);
        if paths.is_empty() {
            continue;
        }
        let seed = kill.unit.to_bits();
        let path = &paths[(seed as usize) % paths.len()];
        let texture = textures
            .0
            .entry(path.clone())
            .or_insert_with(|| asset_server.load::<Image>(splat_url(path)))
            .id();
        let centre = kill.at
            + if kill.overkill {
                dir * FLING_REACH * 0.6
            } else {
                Vec3::ZERO
            };
        let yaw = (seed % 6283) as f32 / 1000.0;
        let half = SPLAT_SIZE * 0.5 * net.scale.clamp(0.5, 3.0);
        let frame = DecalFrame {
            center: centre,
            sin: yaw.sin(),
            cos: yaw.cos(),
            min_x: -half,
            max_x: half,
            min_z: -half,
            max_z: half,
            min_y: -SPLAT_SLAB,
            max_y: SPLAT_SLAB,
        };
        let mut verts = Vec::new();
        if !decals.project(
            &mut verts,
            &frame,
            |_| SPLAT_ALPHA,
            |x, z| frame.rect_uv(x, z),
        ) {
            continue;
        }
        if splats.0.len() >= SPLAT_CAP {
            splats.0.pop_front();
        }
        splats.0.push_back(Splat {
            verts,
            born: now as f32,
            texture,
            anchor: centre,
        });
    }
}

/// A `UnitBlood` splat path as the asset server's URL: forward slashes, the `.blp` extension.
fn splat_url(path: &str) -> String {
    let path = path.replace('\\', "/");
    let stem = path
        .strip_suffix(".blp")
        .or_else(|| path.strip_suffix(".BLP"))
        .unwrap_or(&path);
    format!("mpq://{stem}.blp")
}

/// Hold every clip of a rig whose hit-stop runs, re-asserted each frame (a clip armed this frame
/// would run); let it go when it ends, unless an aura has frozen the rig meanwhile.
fn hold_hit_stops(
    time: Res<Time>,
    mut stops: ResMut<HitStops>,
    mut rigs: Query<(&mut AnimationPlayer, Has<AnimRateFreeze>)>,
) {
    let now = time.elapsed_secs_f64();
    stops.0.retain(|entity, until| {
        let Ok((mut player, frozen)) = rigs.get_mut(*entity) else {
            return false;
        };
        let live = now < *until;
        if live {
            for (_, anim) in player.playing_animations_mut() {
                anim.pause();
            }
        } else if !frozen {
            for (_, anim) in player.playing_animations_mut() {
                anim.resume();
            }
        }
        live
    });
}

/// The push at `age` seconds: out quickly, back slowly, gone at [`PUSH_SECS`].
fn stagger_at(age: f32, push: f32) -> f32 {
    if !(0.0..PUSH_SECS).contains(&age) {
        return 0.0;
    }
    let t = age / PUSH_SECS;
    // A fast rise to the peak at a fifth of the way, then an ease back.
    if t < 0.2 {
        push * (t / 0.2)
    } else {
        push * (1.0 - (t - 0.2) / 0.8).powi(2)
    }
}

/// The fling's offset at `age` seconds along `dir`: out and over, then held where it lands.
fn fling_at(age: f32, dir: Vec3) -> Vec3 {
    let t = (age / FLING_SECS).clamp(0.0, 1.0);
    let out = 1.0 - (1.0 - t).powi(2);
    dir * FLING_REACH * out + Vec3::Y * (FLING_HEIGHT * 4.0 * t * (1.0 - t))
}

/// Lay each push or fling on its rig's root bones, in model space; a settled push restores the
/// bones and goes, a fling holds while the unit is dead.
fn apply_recoil(
    time: Res<Time>,
    mut recoils: ResMut<Recoils>,
    mut rigs: Query<&mut RigPose, Without<AnimParked>>,
    globals: Query<&GlobalTransform>,
    stores: Query<&ObjectStore>,
) {
    let now = time.elapsed_secs_f64();
    recoils.0.retain(|entity, recoil| {
        let Ok(mut rig) = rigs.get_mut(*entity) else {
            // Parked or gone: a parked rig's bones re-seat on wake (`cur != out`).
            return globals.contains(*entity);
        };
        let age = (now - recoil.born) as f32;
        let alive = stores
            .get(*entity)
            .is_ok_and(|s| s.0.unit_health().unwrap_or(1) > 0);
        let (world, done) = match recoil.kind {
            RecoilKind::Stagger(push) => {
                let d = stagger_at(age, push);
                (recoil.dir * d, d == 0.0 && age >= PUSH_SECS)
            }
            // A fling ends when the unit lives again (a respawn reusing the entity).
            RecoilKind::Fling => (fling_at(age, recoil.dir), alive),
        };
        let world = if done { Vec3::ZERO } else { world };
        // Into the model's space: the rig root's inverse, which carries the unit's facing and
        // scale (and a slope's tilt or a seat's frame).
        let Ok(root) = globals.get(rig.joints_root) else {
            return !done;
        };
        let local = root.affine().inverse().transform_vector3(world);
        if recoil.bones.is_empty() {
            recoil.bones = rig
                .parents
                .iter()
                .enumerate()
                .filter(|(_, p)| **p < 0)
                .filter_map(|(i, _)| rig.locals.get(i).map(|t| (i, t.translation, t.translation)))
                .collect();
        }
        let mut dirty = false;
        for (bone, base, last) in recoil.bones.iter_mut() {
            let Some(cur) = rig.locals.get(*bone).map(|t| t.translation) else {
                continue;
            };
            // A bone still holding what was written was not re-keyed: its base stands.
            if cur != *last {
                *base = cur;
            }
            let out = *base + local;
            *last = out;
            if out != cur {
                if let Some(t) = rig.locals.get_mut(*bone) {
                    t.translation = out;
                    dirty = true;
                }
            }
        }
        if dirty {
            rig.pose_dirty = true;
        }
        !done
    });
}

/// The fade of a splat `age` seconds old: whole, then out over its last [`SPLAT_FADE`] seconds.
fn splat_fade(age: f32) -> f32 {
    if age >= SPLAT_LIFE {
        return 0.0;
    }
    ((SPLAT_LIFE - age) / SPLAT_FADE).min(1.0)
}

/// Retire the spent splats and draw the rest, faded in vertex alpha.
fn draw_splats(
    time: Res<Time>,
    cam: Query<Entity, With<WorldCamera>>,
    lane: Option<Res<SplatLane>>,
    mut draw: benilla_world::particles::buffer::WorldEffectDraw,
    mut splats: ResMut<Splats>,
) {
    let now = time.elapsed_secs();
    while splats.0.front().is_some_and(|s| now - s.born >= SPLAT_LIFE) {
        splats.0.pop_front();
    }
    if splats.0.is_empty() {
        return;
    }
    let (Ok(cam), Some(lane)) = (cam.single(), lane) else {
        return;
    };
    for splat in &splats.0 {
        let alpha = splat_fade(now - splat.born);
        let mut batch = draw
            .batch(cam, splat.texture)
            .anchored(splat.anchor)
            .rung(
                benilla_world::sky_order::Rung::FOOTPRINT,
                benilla_world::sky_order::Rung::DECAL_RASTER,
            )
            .owner(lane.0);
        batch.extend(splat.verts.iter().map(|v| EffectVertex {
            color: [v.color[0], v.color[1], v.color[2], v.color[3] * alpha],
            ..*v
        }));
        batch.tris();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_crit_or_a_heavy_blow_holds_the_models() {
        assert_eq!(hit_stop_for(10, 1000, true), Some(STOP_CRIT));
        assert_eq!(hit_stop_for(200, 1000, false), Some(STOP_HEAVY));
        assert_eq!(hit_stop_for(100, 1000, false), None);
        assert_eq!(hit_stop_for(100, 0, false), None);
    }

    #[test]
    fn a_push_peaks_early_and_settles_to_nothing() {
        let peak = stagger_at(PUSH_SECS * 0.2, 0.4);
        assert!((peak - 0.4).abs() < 1e-5);
        assert!(stagger_at(PUSH_SECS * 0.6, 0.4) < peak);
        assert_eq!(stagger_at(PUSH_SECS, 0.4), 0.0);
        assert_eq!(stagger_at(-0.1, 0.4), 0.0);
    }

    #[test]
    fn a_fling_arcs_out_and_lands_where_it_holds() {
        let dir = Vec3::X;
        let mid = fling_at(FLING_SECS * 0.5, dir);
        assert!(mid.y > 0.5 * FLING_HEIGHT && mid.x > 0.0);
        let end = fling_at(FLING_SECS, dir);
        assert!((end - dir * FLING_REACH).length() < 1e-5);
        assert_eq!(fling_at(FLING_SECS * 3.0, dir), end);
    }

    #[test]
    fn a_splat_lies_whole_then_fades_out() {
        assert_eq!(splat_fade(0.0), 1.0);
        assert_eq!(splat_fade(SPLAT_LIFE - SPLAT_FADE), 1.0);
        assert!((splat_fade(SPLAT_LIFE - SPLAT_FADE / 2.0) - 0.5).abs() < 1e-5);
        assert_eq!(splat_fade(SPLAT_LIFE), 0.0);
        assert_eq!(
            splat_url("Textures\\Splats\\Blood01"),
            "mpq://Textures/Splats/Blood01.blp"
        );
    }
}
