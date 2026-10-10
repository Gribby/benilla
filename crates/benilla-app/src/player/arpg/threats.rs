//! Fork-only, not 1.12.1: the ARPG server's telegraphed attacks (cmangos `Arpg/ArpgThreats.h`).
//! An elite, a champion, a rare or a boss winding up a heavy attack sends where it will land
//! (`SMSG_ARPG_TELEGRAPH`): a ring round itself, a cone in front of it, or a circle at a player's
//! feet. The ground shows it at once, a dim area with a bright edge and a fill that grows from
//! the centre (from the apex, for a cone) as the wind-up runs, so the fill reaching the edge is
//! the moment it lands. Then it flashes and fades; a wind-up the server breaks off (a stun on the
//! caster, its death) greys out instead. Standing inside when it lands kicks the camera. The
//! server decides the damage; walking out or rolling avoids it.

use std::collections::HashMap;
use std::f32::consts::FRAC_PI_2;

use benilla_protocol::messages::arpg::{TELEGRAPH_BROKEN, TELEGRAPH_CONE, TELEGRAPH_WIND_UP};
use benilla_protocol::messages::ArpgTelegraph;
use benilla_protocol::{SessionEvent, SessionEventKind};

use crate::net::{GuidIndex, NetHandlerApp, ObjectStore, SelfPlayer};

use super::juice::CameraShake;
use super::*;

/// How long the landing flash and the broken fade last, in seconds.
const FLASH_SECS: f32 = 0.3;
const BROKEN_SECS: f32 = 0.25;
/// The marks' lift off the ground, in yards, each part a hair above the last.
const LIFT: f32 = 0.10;
/// The edge's width, as a share of the radius.
const EDGE: f32 = 0.06;
/// The camera kick for standing in one that lands.
const SHAKE_STRUCK: f32 = 0.75;
/// The alphas: the area, the growing fill, the edge, and the landing flash.
const AREA_ALPHA: f32 = 0.16;
const FILL_ALPHA: f32 = 0.34;
const EDGE_ALPHA: f32 = 0.7;
const FLASH_ALPHA: f32 = 0.75;
/// The colours, by grade: an elite's and a champion's orange, a rare's and a boss's red, a raid
/// boss's deep crimson. A broken one goes grey.
const GRADE_RGB: [[f32; 3]; 5] = [
    [1.0, 0.55, 0.12],
    [1.0, 0.45, 0.10],
    [1.0, 0.25, 0.08],
    [1.0, 0.18, 0.08],
    [0.85, 0.05, 0.12],
];
const BROKEN_RGB: [f32; 3] = [0.55, 0.55, 0.55];

/// The telegraphs the server sent, waiting for the view to draw them.
#[derive(Resource, Default)]
pub(crate) struct ArpgTelegraphs {
    incoming: Vec<ArpgTelegraph>,
}

/// Registered whatever the view, so every session event kind has its owner.
pub(super) fn register_net(app: &mut App) {
    app.init_resource::<ArpgTelegraphs>()
        .net_handler(SessionEventKind::ArpgTelegraph, on_telegraph);
}

fn on_telegraph(In(ev): In<SessionEvent>, mut state: ResMut<ArpgTelegraphs>) {
    if let SessionEvent::ArpgTelegraph { telegraph } = ev {
        state.incoming.push(telegraph);
    }
}

/// The view's half: the marks on the ground.
pub(super) fn plugin(app: &mut App) {
    app.init_resource::<LiveTelegraphs>()
        .add_systems(Update, (take_telegraphs, run_telegraphs).chain());
}

/// Which part of a mark an entity is.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Part {
    Area,
    Fill,
    Edge,
}

/// One part of a mark on the ground.
#[derive(Component)]
struct TelegraphPart {
    part: Part,
    material: Handle<StandardMaterial>,
}

/// A mark being drawn.
struct Live {
    /// Where it lands in the world: the centre or the apex, and the cone's facing.
    centre: Vec3,
    facing: Vec3,
    radius: f32,
    /// The cone's half angle, 0 for a circle.
    half_angle: f32,
    rgb: [f32; 3],
    caster: u64,
    born: f64,
    wind_up: f32,
    /// When the server broke it off.
    broken_at: Option<f64>,
    /// The landing was seen (the kick is given once).
    landed: bool,
    parts: Vec<Entity>,
    /// The parts' own materials, freed with the mark whatever became of the parts.
    materials: Vec<Handle<StandardMaterial>>,
}

#[derive(Resource, Default)]
struct LiveTelegraphs(HashMap<u32, Live>);

/// The meshes, made once: a unit circle and edge ring, and cone sectors by half angle (in
/// thousandths of a radian).
#[derive(Default)]
struct Meshes {
    circle: Option<Handle<Mesh>>,
    edge: Option<Handle<Mesh>>,
    sectors: HashMap<u32, Handle<Mesh>>,
    sector_edges: HashMap<u32, Handle<Mesh>>,
}

/// The rotation that lays a 2D primitive (drawn in XY, a sector opening along +Y) on the ground,
/// opening along `facing`.
fn ground_rotation(facing: Vec3) -> Quat {
    let lay = Quat::from_rotation_x(-FRAC_PI_2); // +Y to -Z, the normal up
    let flat = Vec3::new(facing.x, 0.0, facing.z).normalize_or(Vec3::NEG_Z);
    Quat::from_rotation_arc(Vec3::NEG_Z, flat) * lay
}

/// The WoW orientation `o` at WoW point `at`, as a flat Bevy direction.
fn bevy_facing(at: [f32; 3], o: f32) -> Vec3 {
    let a = benilla_assets::coords::wow_to_bevy(at);
    let b = benilla_assets::coords::wow_to_bevy([at[0] + o.cos(), at[1] + o.sin(), at[2]]);
    let d = b - a;
    Vec3::new(d.x, 0.0, d.z).normalize_or(Vec3::NEG_Z)
}

/// A cone's outline at radius 1, drawn in XY and opening along +Y as [`CircularSector`] does: a
/// band along the arc and one down each side to the apex, each [`EDGE`] wide.
fn cone_edge_mesh(half_angle: f32) -> Mesh {
    use bevy::asset::RenderAssetUsages;
    use bevy::mesh::{Indices, PrimitiveTopology};
    let segments = ((half_angle * 24.0) as u32).clamp(4, 64);
    let mut positions: Vec<[f32; 3]> = Vec::new();
    let mut indices: Vec<u32> = Vec::new();
    let mut quad = |a: Vec2, b: Vec2, c: Vec2, d: Vec2| {
        let base = positions.len() as u32;
        for p in [a, b, c, d] {
            positions.push([p.x, p.y, 0.0]);
        }
        indices.extend_from_slice(&[base, base + 1, base + 2, base, base + 2, base + 3]);
    };
    // The direction at angle `t` off +Y.
    let at = |t: f32, r: f32| Vec2::new(-t.sin(), t.cos()) * r;
    for i in 0..segments {
        let t0 = -half_angle + 2.0 * half_angle * i as f32 / segments as f32;
        let t1 = -half_angle + 2.0 * half_angle * (i + 1) as f32 / segments as f32;
        quad(
            at(t0, 1.0 - EDGE),
            at(t0, 1.0),
            at(t1, 1.0),
            at(t1, 1.0 - EDGE),
        );
    }
    for side in [-half_angle, half_angle] {
        let tip = at(side, 1.0);
        let across = Vec2::new(tip.y, -tip.x).normalize() * (EDGE * 0.5);
        quad(-across, across, tip + across, tip - across);
    }
    let n = positions.len();
    Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::default(),
    )
    .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, positions)
    .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, vec![[0.0, 0.0, 1.0]; n])
    .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, vec![[0.0, 0.0]; n])
    .with_inserted_indices(Indices::U32(indices))
}

/// Whether `point` stands inside the mark (its flat footprint, the body's reach added).
fn inside(live: &Live, point: Vec3, reach: f32) -> bool {
    let d = Vec3::new(point.x - live.centre.x, 0.0, point.z - live.centre.z);
    let dist = d.length();
    if dist > live.radius + reach || (point.y - live.centre.y).abs() > 6.0 {
        return false;
    }
    if live.half_angle <= 0.0 || dist < 0.5 {
        return true;
    }
    let angle = live.facing.angle_between(d / dist);
    angle <= live.half_angle + (reach / dist).atan()
}

fn material(
    materials: &mut Assets<StandardMaterial>,
    rgb: [f32; 3],
    alpha: f32,
) -> Handle<StandardMaterial> {
    materials.add(StandardMaterial {
        base_color: Color::srgba(rgb[0], rgb[1], rgb[2], alpha),
        unlit: true,
        alpha_mode: AlphaMode::Blend,
        cull_mode: None,
        ..default()
    })
}

/// Start each new mark, and break off the ones the server says.
#[allow(clippy::too_many_arguments)]
fn take_telegraphs(
    mut commands: Commands,
    time: Res<Time>,
    mut state: ResMut<ArpgTelegraphs>,
    mut live: ResMut<LiveTelegraphs>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut art: Local<Meshes>,
) {
    let now = time.elapsed_secs_f64();
    for t in std::mem::take(&mut state.incoming) {
        if t.kind == TELEGRAPH_BROKEN {
            if let Some(l) = live.0.get_mut(&t.serial) {
                l.broken_at.get_or_insert(now);
            }
            continue;
        }
        if t.kind != TELEGRAPH_WIND_UP
            || live.0.contains_key(&t.serial)
            || !(t.radius.is_finite() && t.radius > 0.0 && t.radius < 100.0)
            || !t.pos.iter().all(|v| v.is_finite())
        {
            continue;
        }
        let cone = t.shape == TELEGRAPH_CONE && t.half_angle > 0.0;
        let half_angle = if cone {
            t.half_angle.min(std::f32::consts::PI)
        } else {
            0.0
        };
        let centre = benilla_assets::coords::wow_to_bevy(t.pos);
        let facing = bevy_facing(t.pos, t.orientation);
        let rgb = GRADE_RGB[usize::from(t.grade).min(GRADE_RGB.len() - 1)];
        let (fill_mesh, edge_mesh) = if cone {
            let key = (half_angle * 1000.0) as u32;
            let fill = art
                .sectors
                .entry(key)
                .or_insert_with(|| meshes.add(CircularSector::new(1.0, half_angle)))
                .clone();
            let edge = art
                .sector_edges
                .entry(key)
                .or_insert_with(|| meshes.add(cone_edge_mesh(half_angle)))
                .clone();
            (fill, edge)
        } else {
            let fill = art
                .circle
                .get_or_insert_with(|| meshes.add(Circle::new(1.0)))
                .clone();
            let edge = art
                .edge
                .get_or_insert_with(|| meshes.add(Annulus::new(1.0 - EDGE, 1.0)))
                .clone();
            (fill, edge)
        };
        let rotation = ground_rotation(facing);
        let mut parts = Vec::new();
        let mut mats = Vec::new();
        for (part, mesh, alpha, lift) in [
            (Part::Area, fill_mesh.clone(), AREA_ALPHA, LIFT),
            (Part::Fill, fill_mesh.clone(), FILL_ALPHA, LIFT + 0.01),
            (Part::Edge, edge_mesh, EDGE_ALPHA, LIFT + 0.02),
        ] {
            let mat = material(&mut materials, rgb, alpha);
            mats.push(mat.clone());
            let scale = match part {
                Part::Fill => 0.01,
                _ => t.radius,
            };
            parts.push(
                commands
                    .spawn((
                        Mesh3d(mesh),
                        MeshMaterial3d(mat.clone()),
                        Transform::from_translation(centre + Vec3::Y * lift)
                            .with_rotation(rotation)
                            .with_scale(Vec3::splat(scale)),
                        TelegraphPart {
                            part,
                            material: mat,
                        },
                    ))
                    .id(),
            );
        }
        live.0.insert(
            t.serial,
            Live {
                centre,
                facing,
                radius: t.radius,
                half_angle,
                rgb,
                caster: t.caster,
                born: now,
                wind_up: (t.wind_up_ms as f32 / 1000.0).clamp(0.1, 10.0),
                broken_at: None,
                landed: false,
                parts,
                materials: mats,
            },
        );
    }
}

/// What a mark's parts look like `age` seconds in: the fill's scale share and each part's alpha,
/// or `None` once it is spent.
fn mark_at(age: f32, wind_up: f32, broken: Option<f32>) -> Option<(f32, [f32; 3], bool)> {
    if let Some(since) = broken {
        if since >= BROKEN_SECS {
            return None;
        }
        let fade = 1.0 - since / BROKEN_SECS;
        let grow = (age / wind_up).clamp(0.0, 1.0);
        return Some((
            grow,
            [AREA_ALPHA * fade, FILL_ALPHA * fade, EDGE_ALPHA * fade],
            true,
        ));
    }
    if age < wind_up {
        let grow = (age / wind_up).clamp(0.01, 1.0);
        // The edge pulses quicker as it nears.
        let pulse = 0.75 + 0.25 * (age * (6.0 + 10.0 * grow)).sin();
        return Some((grow, [AREA_ALPHA, FILL_ALPHA, EDGE_ALPHA * pulse], false));
    }
    let since = age - wind_up;
    if since >= FLASH_SECS {
        return None;
    }
    let fade = 1.0 - since / FLASH_SECS;
    Some((
        1.0,
        [FLASH_ALPHA * fade, FLASH_ALPHA * fade, FLASH_ALPHA * fade],
        false,
    ))
}

/// Grow, flash and fade the marks; a dead caster's mark breaks off; standing in one that lands
/// kicks the camera.
#[allow(clippy::too_many_arguments)]
fn run_telegraphs(
    mut commands: Commands,
    time: Res<Time>,
    mut live: ResMut<LiveTelegraphs>,
    mut parts: Query<(&TelegraphPart, &mut Transform)>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    index: Res<GuidIndex>,
    stores: Query<&ObjectStore>,
    me: Query<&GlobalTransform, With<SelfPlayer>>,
    mut shake: ResMut<CameraShake>,
) {
    let now = time.elapsed_secs_f64();
    let my_pos = me.single().ok().map(|t| t.translation());
    let mut spent = Vec::new();
    for (serial, l) in live.0.iter_mut() {
        let age = (now - l.born) as f32;
        // A caster that died mid wind-up never lands it.
        if l.broken_at.is_none() && age < l.wind_up {
            let dead = index
                .0
                .get(&l.caster)
                .and_then(|e| stores.get(*e).ok())
                .is_some_and(|s| s.0.unit_health() == Some(0));
            if dead {
                l.broken_at = Some(now);
            }
        }
        let broken = l.broken_at.map(|at| (now - at) as f32);
        if broken.is_none() && age >= l.wind_up && !l.landed {
            l.landed = true;
            if my_pos.is_some_and(|p| inside(l, p, 0.4)) {
                shake.kick(SHAKE_STRUCK);
            }
        }
        match mark_at(age, l.wind_up, broken) {
            None => spent.push(*serial),
            Some((grow, alphas, grey)) => {
                let rgb = if grey { BROKEN_RGB } else { l.rgb };
                for e in &l.parts {
                    let Ok((part, mut tf)) = parts.get_mut(*e) else {
                        continue;
                    };
                    let alpha = match part.part {
                        Part::Area => alphas[0],
                        Part::Fill => alphas[1],
                        Part::Edge => alphas[2],
                    };
                    if part.part == Part::Fill {
                        tf.scale = Vec3::splat((l.radius * grow).max(0.01));
                    }
                    if let Some(m) = materials.get_mut(&part.material) {
                        m.base_color = Color::srgba(rgb[0], rgb[1], rgb[2], alpha);
                    }
                }
            }
        }
    }
    for serial in spent {
        if let Some(l) = live.0.remove(&serial) {
            for m in &l.materials {
                materials.remove(m);
            }
            for e in l.parts {
                if let Ok(mut ec) = commands.get_entity(e) {
                    ec.despawn();
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn live(half_angle: f32) -> Live {
        Live {
            centre: Vec3::ZERO,
            facing: Vec3::NEG_Z,
            radius: 10.0,
            half_angle,
            rgb: GRADE_RGB[0],
            caster: 1,
            born: 0.0,
            wind_up: 1.5,
            broken_at: None,
            landed: false,
            parts: Vec::new(),
            materials: Vec::new(),
        }
    }

    #[test]
    fn the_cone_outline_has_its_arc_and_both_sides() {
        let mesh = cone_edge_mesh(std::f32::consts::FRAC_PI_4);
        let quads = mesh.count_vertices() / 4;
        assert_eq!(
            quads,
            ((std::f32::consts::FRAC_PI_4 * 24.0) as usize).clamp(4, 64) + 2
        );
    }

    #[test]
    fn a_ring_holds_what_is_within_its_radius() {
        let ring = live(0.0);
        assert!(inside(&ring, Vec3::new(6.0, 0.0, 6.0), 0.4));
        assert!(!inside(&ring, Vec3::new(9.0, 0.0, 9.0), 0.4));
        assert!(!inside(&ring, Vec3::new(1.0, 9.0, 1.0), 0.4));
    }

    #[test]
    fn a_cone_holds_only_what_is_in_front() {
        let cone = live(std::f32::consts::FRAC_PI_4);
        assert!(inside(&cone, Vec3::new(0.0, 0.0, -8.0), 0.4));
        assert!(inside(&cone, Vec3::new(3.0, 0.0, -6.0), 0.4));
        assert!(!inside(&cone, Vec3::new(0.0, 0.0, 8.0), 0.4));
        assert!(!inside(&cone, Vec3::new(8.0, 0.0, -1.0), 0.4));
    }

    #[test]
    fn a_mark_grows_lands_flashes_and_ends() {
        let (g, a, grey) = mark_at(0.75, 1.5, None).unwrap();
        assert!((g - 0.5).abs() < 1e-6 && !grey && a[1] == FILL_ALPHA);
        let (g, a, _) = mark_at(1.5, 1.5, None).unwrap();
        assert_eq!(g, 1.0);
        assert_eq!(a[0], FLASH_ALPHA);
        assert!(mark_at(1.5 + FLASH_SECS + 0.01, 1.5, None).is_none());
        let (_, _, grey) = mark_at(0.5, 1.5, Some(0.1)).unwrap();
        assert!(grey);
        assert!(mark_at(0.5, 1.5, Some(BROKEN_SECS)).is_none());
    }

    #[test]
    fn a_cone_lies_flat_and_opens_along_its_facing() {
        let facing = Vec3::new(1.0, 0.0, 0.0);
        let r = ground_rotation(facing);
        // The sector opens along local +Y; laid down, that points along the facing.
        assert!((r * Vec3::Y - facing).length() < 1e-5);
        // Its face looks up.
        assert!((r * Vec3::Z - Vec3::Y).length() < 1e-5);
    }
}
