//! Fork-only, not 1.12.1: the ARPG view's gibs. A kill by a heavy blow (a crit, or a quarter of
//! the health at once: `super::juice`'s overkill) bursts the corpse: its body collapses away
//! (`super::impact`) as a spray of chunks flies out of it, tumbling, bouncing on the ground and
//! settling, then shrinking away. A physical blow tears a creature that bleeds into flesh and a
//! little bone, one that does not (an elemental, a skeleton, a construct) into grey stone and
//! bone. A spell's kill comes apart in its element: fire in burning embers that cool to char,
//! frost in ice shards that skid, nature in sizzling green, shadow in dark motes that sink slowly
//! and fade, arcane and holy in glowing shards that float up. A champion or a rare comes apart in
//! more pieces. Beside it, `super::strikes` sprays the creature's own blood
//! and `super::impact` lays its blood on the ground.
//!
//! Drawing only, local to this client, as the rest of the feel is.

use bevy::prelude::*;

use crate::net::NetEntity;

use super::juice::ArpgKill;

/// Chunks for a kill of a unit of scale 1, and more for a champion's or a rare's; at most this
/// many at once in the world.
const CHUNKS: f32 = 11.0;
const CHUNKS_PER_TIER: u32 = 6;
const CHUNKS_LIVE_MAX: usize = 160;
/// A chunk's edge, in yards at scale 1, from smallest to largest.
const SIZE_MIN: f32 = 0.1;
const SIZE_MAX: f32 = 0.28;
/// Where the chunks start, over the feet, at scale 1.
const BURST_HEIGHT: f32 = 0.9;
/// Their launch: outward and upward speeds in yards a second, and their spin in turns a second.
const OUT_MIN: f32 = 2.5;
const OUT_MAX: f32 = 7.0;
const UP_MIN: f32 = 2.5;
const UP_MAX: f32 = 7.5;
const SPIN_MAX: f32 = 3.0;
/// Gravity (at an element's weight of 1), and what a bounce keeps of the fall and of the slide.
const GRAVITY: f32 = 22.0;
const BOUNCE_KEEP: f32 = 0.32;
const SLIDE_KEEP: f32 = 0.55;
/// What of a lying chunk's slide and tumble survives a second on the ground.
const REST_KEEP_PER_SEC: f32 = 0.002;
/// A chunk lies this long (at an element's), then shrinks away over the next stretch.
const LIE_SECS: f32 = 2.4;
const SHRINK_SECS: f32 = 0.7;

/// The flesh and the stone, and the bone among either.
const FLESH: [[f32; 3]; 3] = [[0.42, 0.04, 0.035], [0.58, 0.09, 0.07], [0.3, 0.025, 0.025]];
const STONE: [[f32; 3]; 3] = [[0.5, 0.48, 0.44], [0.36, 0.34, 0.31], [0.62, 0.6, 0.55]];
const BONE: [f32; 3] = [0.82, 0.77, 0.66];
/// One chunk in this many is bone.
const BONE_EVERY: u32 = 5;

/// The elements' chunks: fire's embers and the char they cool to, frost's ice, nature's green,
/// shadow's dark motes, arcane's and holy's light.
const EMBER: [[f32; 3]; 3] = [[1.0, 0.42, 0.06], [1.0, 0.68, 0.18], [0.95, 0.22, 0.03]];
const CHAR: [[f32; 3]; 3] = [[0.07, 0.05, 0.04], [0.16, 0.09, 0.05], [0.3, 0.07, 0.02]];
const ICE: [[f32; 3]; 3] = [[0.72, 0.88, 1.0], [0.5, 0.72, 0.98], [0.88, 0.96, 1.0]];
const VENOM: [[f32; 3]; 3] = [[0.38, 0.8, 0.18], [0.2, 0.52, 0.1], [0.62, 0.95, 0.38]];
const GLOOM: [[f32; 3]; 3] = [[0.32, 0.12, 0.45], [0.14, 0.05, 0.2], [0.5, 0.22, 0.66]];
const ARCANE: [[f32; 3]; 3] = [[0.9, 0.5, 1.0], [0.62, 0.38, 1.0], [1.0, 0.78, 1.0]];
const RADIANT: [[f32; 3]; 3] = [[1.0, 0.9, 0.5], [1.0, 0.78, 0.3], [1.0, 1.0, 0.8]];
/// Fire's embers burn this long, between the two, before they are char.
const EMBER_SECS: (f32, f32) = (0.5, 1.6);

/// How a kill's chunks look and move, by the killing blow's element.
#[derive(Clone, Copy, Debug, PartialEq)]
struct Look {
    /// The palette's key in [`GibArt`], and its colours.
    key: u8,
    palette: &'static [[f32; 3]],
    /// The palette's brightness: past 1 it glows under the bloom.
    glow: f32,
    /// What a chunk cools to (fire's char), and after how long.
    cooled: Option<&'static [[f32; 3]]>,
    /// Bone among the chunks.
    bone: bool,
    /// Gravity's share: under 1 drifts, below 0 floats up.
    weight: f32,
    /// Lying time, chunk count and size, against a physical kill's.
    lie: f32,
    count: f32,
    size: f32,
    /// What a lying chunk keeps of its slide over a second (ice skids).
    rest_keep: f32,
}

/// The chunks of a kill by a blow of school `school` (Physical to Arcane, as the combat log's), on
/// a creature that `bleeds` or not.
fn look_of(school: u8, bleeds: bool) -> Look {
    let base = Look {
        key: 0,
        palette: &FLESH,
        glow: 1.0,
        cooled: None,
        bone: true,
        weight: 1.0,
        lie: LIE_SECS,
        count: 1.0,
        size: 1.0,
        rest_keep: REST_KEEP_PER_SEC,
    };
    let magic = |key, palette, glow| Look {
        key,
        palette,
        glow,
        bone: false,
        ..base
    };
    match school {
        1 => Look {
            weight: -0.12,
            lie: 1.1,
            ..magic(1, &RADIANT[..], 2.6)
        },
        2 => Look {
            cooled: Some(&CHAR[..]),
            bone: true,
            ..magic(2, &EMBER[..], 3.2)
        },
        3 => magic(3, &VENOM[..], 1.6),
        4 => Look {
            count: 1.5,
            size: 0.8,
            rest_keep: 0.25,
            ..magic(4, &ICE[..], 1.3)
        },
        5 => Look {
            weight: 0.3,
            lie: 1.3,
            ..magic(5, &GLOOM[..], 1.5)
        },
        6 => Look {
            weight: -0.15,
            lie: 1.0,
            ..magic(6, &ARCANE[..], 2.6)
        },
        _ if bleeds => base,
        _ => Look {
            key: 7,
            palette: &STONE,
            ..base
        },
    }
}

/// One chunk in flight or at rest.
#[derive(Component)]
struct Gib {
    vel: Vec3,
    spin: Vec3,
    age: f32,
    /// Its element's gravity share, lying time and resting slide ([`Look`]).
    weight: f32,
    lie: f32,
    rest_keep: f32,
    /// What it cools to, and when (fire's embers to char).
    cool: Option<(f32, Handle<StandardMaterial>)>,
    /// The ground under it (the corpse's feet), and its half-size, so it lies on it.
    ground: f32,
    half: f32,
    size: f32,
}

/// The chunk mesh and the palettes' materials, each made once: by [`Look::key`], and the cooled
/// palettes past [`COOLED_KEY`].
#[derive(Resource, Default)]
struct GibArt {
    mesh: Option<Handle<Mesh>>,
    palettes: std::collections::HashMap<u8, Vec<Handle<StandardMaterial>>>,
    bone: Option<Handle<StandardMaterial>>,
}

/// Added to a look's key for its cooled palette's.
const COOLED_KEY: u8 = 100;

impl GibArt {
    fn palette(
        &mut self,
        materials: &mut Assets<StandardMaterial>,
        key: u8,
        colours: &[[f32; 3]],
        glow: f32,
    ) -> Vec<Handle<StandardMaterial>> {
        self.palettes
            .entry(key)
            .or_insert_with(|| colours.iter().map(|&c| unlit(materials, c, glow)).collect())
            .clone()
    }
}

pub(super) fn plugin(app: &mut App) {
    app.init_resource::<GibArt>()
        .add_systems(Update, (burst_corpses, fly_gibs).chain());
}

/// A small, fast random stream, seeded per kill: the chunks' scatter needs no more.
struct Scatter(u64);

impl Scatter {
    fn next(&mut self) -> f32 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        (self.0 >> 40) as f32 / (1u64 << 24) as f32
    }
    fn range(&mut self, lo: f32, hi: f32) -> f32 {
        lo + (hi - lo) * self.next()
    }
}

/// How many chunks a kill of a unit of `scale` and champion `tier` bursts into.
fn chunk_count(scale: f32, tier: u8) -> u32 {
    (CHUNKS * scale.clamp(0.7, 2.5)).round() as u32 + u32::from(tier.min(2)) * CHUNKS_PER_TIER
}

/// An unlit chunk material; a `glow` past 1 lifts it past white, for the bloom to catch.
fn unlit(
    materials: &mut Assets<StandardMaterial>,
    rgb: [f32; 3],
    glow: f32,
) -> Handle<StandardMaterial> {
    let base = Color::srgb(rgb[0], rgb[1], rgb[2]).to_linear();
    materials.add(StandardMaterial {
        base_color: Color::LinearRgba(LinearRgba::rgb(
            base.red * glow,
            base.green * glow,
            base.blue * glow,
        )),
        unlit: true,
        ..default()
    })
}

/// Each heavy kill's chunks.
#[allow(clippy::too_many_arguments)]
fn burst_corpses(
    mut commands: Commands,
    mut kills: MessageReader<ArpgKill>,
    nets: Query<&NetEntity>,
    blood: Option<Res<crate::creature_anim::BloodTables>>,
    creatures: Option<Res<crate::entities::Creatures>>,
    live: Query<(), With<Gib>>,
    mut art: ResMut<GibArt>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let mut room = CHUNKS_LIVE_MAX.saturating_sub(live.iter().count());
    for kill in kills.read() {
        if !kill.overkill || room == 0 {
            continue;
        }
        let net = nets.get(kill.unit).ok();
        let scale = net.map_or(1.0, |n| n.scale).clamp(0.5, 3.0);
        // A creature with blood to spill comes apart in flesh; one without, in stone and bone.
        let bleeds = match (
            net.and_then(|n| n.display_id),
            blood.as_deref(),
            creatures.as_deref(),
        ) {
            (Some(display), Some(blood), Some(creatures)) => {
                !crate::creature_anim::kill_splats(blood, creatures, display).is_empty()
            }
            _ => true,
        };
        if art.mesh.is_none() {
            art.mesh = Some(meshes.add(Cuboid::new(1.0, 1.0, 1.0)));
            art.bone = Some(unlit(&mut materials, BONE, 1.0));
        }
        let (Some(mesh), Some(bone)) = (art.mesh.clone(), art.bone.clone()) else {
            continue;
        };
        let look = look_of(kill.school, bleeds);
        let palette = art.palette(&mut materials, look.key, look.palette, look.glow);
        let cooled = look
            .cooled
            .map(|c| art.palette(&mut materials, look.key + COOLED_KEY, c, 1.0));
        let count =
            ((chunk_count(scale, kill.tier) as f32 * look.count).round() as usize).min(room);
        room -= count;
        let mut rng = Scatter(
            kill.unit.to_bits() ^ 0x9E37_79B9_7F4A_7C15 ^ (kill.at.x.to_bits() as u64) << 20,
        );
        let origin = kill.at + Vec3::Y * BURST_HEIGHT * scale;
        for i in 0..count as u32 {
            let size = rng.range(SIZE_MIN, SIZE_MAX) * scale.sqrt() * look.size;
            let angle = rng.range(0.0, std::f32::consts::TAU);
            let out = rng.range(OUT_MIN, OUT_MAX);
            let vel = Vec3::new(
                angle.cos() * out,
                rng.range(UP_MIN, UP_MAX),
                angle.sin() * out,
            );
            let spin = Vec3::new(
                rng.range(-1.0, 1.0),
                rng.range(-1.0, 1.0),
                rng.range(-1.0, 1.0),
            ) * SPIN_MAX
                * std::f32::consts::TAU;
            let shade = i as usize % palette.len();
            let is_bone = look.bone && i % BONE_EVERY == BONE_EVERY - 1;
            let material = if is_bone && look.cooled.is_none() {
                bone.clone()
            } else {
                palette[shade].clone()
            };
            let cool = cooled
                .as_ref()
                .map(|c| (rng.range(EMBER_SECS.0, EMBER_SECS.1), c[shade].clone()));
            // Not cubes: each chunk stretched its own way.
            let shape = Vec3::new(
                rng.range(0.6, 1.4),
                rng.range(0.5, 1.1),
                rng.range(0.6, 1.4),
            ) * size;
            commands.spawn((
                Mesh3d(mesh.clone()),
                MeshMaterial3d(material),
                Transform::from_translation(origin + vel.normalize_or_zero() * 0.2 * scale)
                    .with_rotation(Quat::from_scaled_axis(spin * 0.1))
                    .with_scale(shape),
                Gib {
                    vel,
                    spin,
                    age: 0.0,
                    weight: look.weight,
                    lie: look.lie,
                    rest_keep: look.rest_keep,
                    cool,
                    ground: kill.at.y,
                    half: shape.y * 0.5,
                    size: 1.0,
                },
                Name::new("arpg-gib"),
            ));
        }
    }
}

/// The scale a chunk `age` seconds old keeps: whole while it lies `lie` seconds, then shrinking
/// to nothing.
fn gib_left(age: f32, lie: f32) -> f32 {
    if age <= lie {
        return 1.0;
    }
    (1.0 - (age - lie) / SHRINK_SECS).max(0.0)
}

/// Fly, bounce, settle and shrink each chunk.
fn fly_gibs(
    mut commands: Commands,
    time: Res<Time>,
    mut gibs: Query<(
        Entity,
        &mut Gib,
        &mut Transform,
        &mut MeshMaterial3d<StandardMaterial>,
    )>,
) {
    let dt = time.delta_secs().min(0.05);
    for (entity, mut gib, mut tf, mut material) in &mut gibs {
        gib.age += dt;
        let left = gib_left(gib.age, gib.lie);
        if left <= 0.0 {
            commands.entity(entity).despawn();
            continue;
        }
        if gib.cool.as_ref().is_some_and(|(at, _)| gib.age >= *at) {
            if let Some((_, cooled)) = gib.cool.take() {
                material.0 = cooled;
            }
        }
        gib.vel.y -= GRAVITY * gib.weight * dt;
        tf.translation += gib.vel * dt;
        let rest = gib.ground + gib.half;
        if tf.translation.y < rest {
            tf.translation.y = rest;
            if gib.vel.y < 0.0 {
                gib.vel.y = -gib.vel.y * BOUNCE_KEEP;
                if gib.vel.y < 0.6 {
                    gib.vel.y = 0.0;
                }
                // Each landing scrubs the slide and the tumble.
                gib.vel.x *= SLIDE_KEEP;
                gib.vel.z *= SLIDE_KEEP;
                gib.spin *= SLIDE_KEEP;
            }
            // Lying, friction by the second, so it stops alike at any frame rate.
            let rub = gib.rest_keep.powf(dt);
            gib.vel.x *= rub;
            gib.vel.z *= rub;
            gib.spin *= rub;
        }
        let spin = gib.spin * dt;
        tf.rotate(Quat::from_scaled_axis(spin));
        if left < gib.size {
            tf.scale *= left / gib.size;
            gib.size = left;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_bigger_or_tougher_foe_comes_apart_in_more_pieces() {
        assert_eq!(chunk_count(1.0, 0), 11);
        assert!(chunk_count(2.0, 0) > chunk_count(1.0, 0));
        assert_eq!(chunk_count(1.0, 2), 11 + 2 * CHUNKS_PER_TIER);
    }

    #[test]
    fn a_chunk_lies_whole_then_shrinks_away() {
        assert_eq!(gib_left(0.0, LIE_SECS), 1.0);
        assert_eq!(gib_left(LIE_SECS, LIE_SECS), 1.0);
        assert!(gib_left(LIE_SECS + SHRINK_SECS * 0.5, LIE_SECS) > 0.4);
        assert_eq!(gib_left(LIE_SECS + SHRINK_SECS, LIE_SECS), 0.0);
    }

    #[test]
    fn each_element_comes_apart_its_own_way() {
        // Physical: flesh on a bleeder, stone on the rest, both with bone.
        assert_eq!(look_of(0, true).palette, &FLESH[..]);
        assert_eq!(look_of(0, false).palette, &STONE[..]);
        assert!(look_of(0, true).bone);
        // Fire burns, glowing, and cools to char.
        let fire = look_of(2, true);
        assert!(fire.glow > 1.0 && fire.cooled == Some(&CHAR[..]));
        // Frost shatters finer and skids.
        let frost = look_of(4, false);
        assert!(frost.count > 1.0 && frost.rest_keep > REST_KEEP_PER_SEC);
        // Arcane and holy float up; shadow drifts down slowly.
        assert!(look_of(6, true).weight < 0.0 && look_of(1, true).weight < 0.0);
        assert!((0.0..1.0).contains(&look_of(5, true).weight));
        // Every element's palette key is its own.
        let keys: std::collections::HashSet<u8> = (0..7)
            .map(|s| look_of(s, true).key)
            .chain([look_of(0, false).key])
            .collect();
        assert_eq!(keys.len(), 8);
    }
}
