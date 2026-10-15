//! Fork-only, not 1.12.1: the ARPG view's gibs. A kill by a heavy blow (a crit, or a quarter of
//! the health at once: `super::juice`'s overkill) bursts the corpse: its body collapses away
//! (`super::impact`) as a spray of chunks flies out of it, tumbling, bouncing on the ground and
//! settling, then shrinking away. A creature that bleeds comes apart in flesh and a little bone; one
//! that does not (an elemental, a skeleton, a construct) in grey stone and bone. A champion or a
//! rare comes apart in more pieces. Beside it, `super::strikes` sprays the creature's own blood
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
/// Gravity, and what a bounce keeps of the fall and of the slide.
const GRAVITY: f32 = 22.0;
const BOUNCE_KEEP: f32 = 0.32;
const SLIDE_KEEP: f32 = 0.55;
/// What of a lying chunk's slide and tumble survives a second on the ground.
const REST_KEEP_PER_SEC: f32 = 0.002;
/// A chunk lies this long, then shrinks away over the next stretch.
const LIE_SECS: f32 = 2.4;
const SHRINK_SECS: f32 = 0.7;

/// The flesh and the stone, and the bone among either.
const FLESH: [[f32; 3]; 3] = [[0.42, 0.04, 0.035], [0.58, 0.09, 0.07], [0.3, 0.025, 0.025]];
const STONE: [[f32; 3]; 3] = [[0.5, 0.48, 0.44], [0.36, 0.34, 0.31], [0.62, 0.6, 0.55]];
const BONE: [f32; 3] = [0.82, 0.77, 0.66];
/// One chunk in this many is bone.
const BONE_EVERY: u32 = 5;

/// One chunk in flight or at rest.
#[derive(Component)]
struct Gib {
    vel: Vec3,
    spin: Vec3,
    age: f32,
    /// The ground under it (the corpse's feet), and its half-size, so it lies on it.
    ground: f32,
    half: f32,
    size: f32,
}

/// The chunk mesh and the palette's materials, made once.
#[derive(Resource, Default)]
struct GibArt {
    mesh: Option<Handle<Mesh>>,
    flesh: Vec<Handle<StandardMaterial>>,
    stone: Vec<Handle<StandardMaterial>>,
    bone: Option<Handle<StandardMaterial>>,
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

fn unlit(materials: &mut Assets<StandardMaterial>, rgb: [f32; 3]) -> Handle<StandardMaterial> {
    materials.add(StandardMaterial {
        base_color: Color::srgb(rgb[0], rgb[1], rgb[2]),
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
            art.flesh = FLESH.iter().map(|&c| unlit(&mut materials, c)).collect();
            art.stone = STONE.iter().map(|&c| unlit(&mut materials, c)).collect();
            art.bone = Some(unlit(&mut materials, BONE));
        }
        let (Some(mesh), Some(bone)) = (art.mesh.clone(), art.bone.clone()) else {
            continue;
        };
        let palette = if bleeds { &art.flesh } else { &art.stone };
        let count = (chunk_count(scale, kill.tier) as usize).min(room);
        room -= count;
        let mut rng = Scatter(
            kill.unit.to_bits() ^ 0x9E37_79B9_7F4A_7C15 ^ (kill.at.x.to_bits() as u64) << 20,
        );
        let origin = kill.at + Vec3::Y * BURST_HEIGHT * scale;
        for i in 0..count as u32 {
            let size = rng.range(SIZE_MIN, SIZE_MAX) * scale.sqrt();
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
            let material = if i % BONE_EVERY == BONE_EVERY - 1 {
                bone.clone()
            } else {
                palette[i as usize % palette.len()].clone()
            };
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
                    ground: kill.at.y,
                    half: shape.y * 0.5,
                    size: 1.0,
                },
                Name::new("arpg-gib"),
            ));
        }
    }
}

/// The scale a chunk `age` seconds old keeps: whole while it lies, then shrinking to nothing.
fn gib_left(age: f32) -> f32 {
    if age <= LIE_SECS {
        return 1.0;
    }
    (1.0 - (age - LIE_SECS) / SHRINK_SECS).max(0.0)
}

/// Fly, bounce, settle and shrink each chunk.
fn fly_gibs(
    mut commands: Commands,
    time: Res<Time>,
    mut gibs: Query<(Entity, &mut Gib, &mut Transform)>,
) {
    let dt = time.delta_secs().min(0.05);
    for (entity, mut gib, mut tf) in &mut gibs {
        gib.age += dt;
        let left = gib_left(gib.age);
        if left <= 0.0 {
            commands.entity(entity).despawn();
            continue;
        }
        gib.vel.y -= GRAVITY * dt;
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
            let rub = REST_KEEP_PER_SEC.powf(dt);
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
        assert_eq!(gib_left(0.0), 1.0);
        assert_eq!(gib_left(LIE_SECS), 1.0);
        assert!(gib_left(LIE_SECS + SHRINK_SECS * 0.5) > 0.4);
        assert_eq!(gib_left(LIE_SECS + SHRINK_SECS), 0.0);
    }
}
