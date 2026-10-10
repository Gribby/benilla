//! Fork-only, not 1.12.1: the ARPG server's telegraphed attacks (cmangos `Arpg/ArpgThreats.h`).
//! An elite, a champion, a rare or a boss winding up a heavy attack sends where it will land
//! (`SMSG_ARPG_TELEGRAPH`): a ring round itself, a cone in front of it, or a circle at a player's
//! feet. The ground shows it at once, a dim area with a bright edge and a fill that grows from
//! the centre (from the apex, for a cone) as the wind-up runs, so the fill reaching the edge is
//! the moment it lands. Then it flashes and fades; a wind-up the server breaks off (a stun on the
//! caster, its death) greys out instead. Standing inside when it lands kicks the camera. The
//! server decides the damage; walking out or rolling avoids it.
//!
//! A mark lies over the ground, not flat at the caster's height: it is a polar grid of rings and
//! spokes, each point set down on the ground under it (a ray to the world's floors and terrain)
//! when the mark arrives, so it follows a hillside or a stair.

use std::collections::HashMap;
use std::f32::consts::TAU;

use bevy::asset::RenderAssetUsages;
use bevy::mesh::{Indices, PrimitiveTopology};

use benilla_protocol::messages::arpg::{
    TELEGRAPH_BROKEN, TELEGRAPH_CONE, TELEGRAPH_LINE, TELEGRAPH_WIND_UP,
};
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
const GRADE_RGB: [[f32; 3]; 6] = [
    [1.0, 0.55, 0.12],
    [1.0, 0.45, 0.10],
    [1.0, 0.25, 0.08],
    [1.0, 0.18, 0.08],
    [0.85, 0.05, 0.12],
    // An ordinary creature's move: a lighter amber.
    [1.0, 0.78, 0.25],
];
const BROKEN_RGB: [f32; 3] = [0.55, 0.55, 0.55];
/// The draped grid: rings from the centre out, and spokes round a full circle (a cone keeps its
/// share of them).
const RINGS: usize = 8;
const CIRCLE_SPOKES: usize = 40;
/// A line's columns across its width.
const LINE_COLUMNS: usize = 5;
/// The ground under a mark point is looked for from this far above the ground found one ring in,
/// this far down, in yards: walking out along each spoke, a mark climbs a slope or a stair and
/// stays under a low ceiling. A rise steeper than the first try is looked for again from higher.
const PROBE_UP: f32 = 1.5;
const PROBE_UP_STEEP: f32 = 4.0;
const PROBE_DOWN: f32 = 14.0;
/// Ground found this far under the server's centre is not the caster's (a lake bed under a
/// swimmer): the centre keeps the server's height.
const CENTRE_DROP: f32 = 2.0;
/// The fill is rebuilt when its reach moves by this share of the radius.
const FILL_STEP: f32 = 0.01;

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

/// A mark laid over the ground: the flat direction of each spoke and the ground's height at each
/// ring of each spoke, `heights[ring * spokes + spoke]`.
#[derive(Clone, Debug)]
struct Drape {
    radius: f32,
    /// A circle's or a cone's: each spoke's flat direction out from the centre. A line's: each
    /// column's flat offset across the line from its axis.
    spokes: Vec<Vec3>,
    /// A circle's spokes go all the way round and join; a cone's and a line's stop at the edges.
    closed: bool,
    heights: Vec<f32>,
    /// A line's flat direction from its start: its rings run along it.
    line: Option<Vec3>,
}

impl Drape {
    /// The spokes of a circle (`half_angle` 0) or of a cone opening along `facing`.
    fn spokes(facing: Vec3, half_angle: f32) -> (Vec<Vec3>, bool) {
        if half_angle <= 0.0 {
            let spokes = (0..CIRCLE_SPOKES)
                .map(|i| Quat::from_rotation_y(TAU * i as f32 / CIRCLE_SPOKES as f32) * facing)
                .collect();
            return (spokes, true);
        }
        let n = ((2.0 * half_angle / TAU * CIRCLE_SPOKES as f32).ceil() as usize).max(4) + 1;
        let spokes = (0..n)
            .map(|i| {
                let t = -half_angle + 2.0 * half_angle * i as f32 / (n - 1) as f32;
                Quat::from_rotation_y(t) * facing
            })
            .collect();
        (spokes, false)
    }

    /// A mark of `radius`, each point set down by `ground(point, from)`: the height of the ground
    /// under `point` looking down from height `from`, if any.
    fn new(
        centre: Vec3,
        radius: f32,
        facing: Vec3,
        half_angle: f32,
        mut ground: impl FnMut(Vec3, f32) -> Option<f32>,
    ) -> Self {
        let (spokes, closed) = Self::spokes(facing, half_angle);
        let n = spokes.len();
        let centre_y = ground(centre, centre.y + PROBE_UP)
            .filter(|y| *y >= centre.y - CENTRE_DROP)
            .unwrap_or(centre.y);
        let mut heights = vec![centre_y; (RINGS + 1) * n];
        for (s, dir) in spokes.iter().enumerate() {
            let mut prev = centre_y;
            for ring in 1..=RINGS {
                let p = centre + *dir * radius * ring as f32 / RINGS as f32;
                let y = ground(p, prev + PROBE_UP)
                    .or_else(|| ground(p, prev + PROBE_UP_STEEP))
                    .unwrap_or(prev);
                heights[ring * n + s] = y;
                prev = y;
            }
        }
        Self {
            radius,
            spokes,
            closed,
            heights,
            line: None,
        }
    }

    /// A line of `length` from `start` along `dir`, `half_width` either side, set down as
    /// [`Self::new`] sets a circle: each column walked out from its start.
    fn line(
        start: Vec3,
        length: f32,
        dir: Vec3,
        half_width: f32,
        mut ground: impl FnMut(Vec3, f32) -> Option<f32>,
    ) -> Self {
        let across = Vec3::new(-dir.z, 0.0, dir.x);
        let spokes: Vec<Vec3> = (0..LINE_COLUMNS)
            .map(|i| across * half_width * (2.0 * i as f32 / (LINE_COLUMNS - 1) as f32 - 1.0))
            .collect();
        let n = spokes.len();
        let start_y = ground(start, start.y + PROBE_UP)
            .filter(|y| *y >= start.y - CENTRE_DROP)
            .unwrap_or(start.y);
        let mut heights = vec![start_y; (RINGS + 1) * n];
        for (s, offset) in spokes.iter().enumerate() {
            let mut prev = ground(start + *offset, start_y + PROBE_UP).unwrap_or(start_y);
            heights[s] = prev;
            for ring in 1..=RINGS {
                let p = start + *offset + dir * length * ring as f32 / RINGS as f32;
                let y = ground(p, prev + PROBE_UP)
                    .or_else(|| ground(p, prev + PROBE_UP_STEEP))
                    .unwrap_or(prev);
                heights[ring * n + s] = y;
                prev = y;
            }
        }
        Self {
            radius: length,
            spokes,
            closed: false,
            heights,
            line: Some(dir),
        }
    }

    /// The flat offset from the centre of the point `frac` of the way out along spoke `spoke`.
    fn flat(&self, frac: f32, spoke: usize) -> Vec3 {
        match self.line {
            Some(dir) => dir * self.radius * frac + self.spokes[spoke],
            None => self.spokes[spoke] * self.radius * frac,
        }
    }

    /// The ground's height `frac` of the way out along spoke `spoke`, between its rings.
    fn height(&self, frac: f32, spoke: usize) -> f32 {
        let n = self.spokes.len();
        let x = frac.clamp(0.0, 1.0) * RINGS as f32;
        let i = (x.floor() as usize).min(RINGS - 1);
        let t = x - i as f32;
        let a = self.heights[i * n + spoke];
        let b = self.heights[(i + 1) * n + spoke];
        a + (b - a) * t
    }

    /// A point of the mark, relative to its centre on the ground plane, at its ground height.
    fn point(&self, frac: f32, spoke: usize, lift: f32) -> [f32; 3] {
        let flat = self.flat(frac, spoke);
        [flat.x, self.height(frac, spoke) + lift, flat.z]
    }

    /// The band of the mark from `f0` to `f1` of its radius, in `steps` rings, into `out`.
    fn band(&self, f0: f32, f1: f32, steps: usize, lift: f32, out: &mut MeshParts) {
        let n = self.spokes.len();
        let base = out.positions.len() as u32;
        for j in 0..=steps {
            let f = f0 + (f1 - f0) * j as f32 / steps as f32;
            for s in 0..n {
                out.positions.push(self.point(f, s, lift));
            }
        }
        let joins = if self.closed { n } else { n - 1 };
        for j in 0..steps as u32 {
            for s in 0..joins as u32 {
                let s1 = (s + 1) % n as u32;
                let (a, b) = (base + j * n as u32 + s, base + j * n as u32 + s1);
                let (c, d) = (a + n as u32, b + n as u32);
                out.indices.extend_from_slice(&[a, c, b, b, c, d]);
            }
        }
    }

    /// A cone's two straight edges, apex to rim, each `width` yards wide, into `out`.
    fn sides(&self, width: f32, lift: f32, out: &mut MeshParts) {
        if self.closed {
            return;
        }
        for spoke in [0, self.spokes.len() - 1] {
            let dir = self.line.unwrap_or(self.spokes[spoke]);
            let across = Vec3::new(-dir.z, 0.0, dir.x) * (width * 0.5);
            let base = out.positions.len() as u32;
            for j in 0..=RINGS {
                let f = j as f32 / RINGS as f32;
                let [x, y, z] = self.point(f, spoke, lift);
                out.positions.push([x - across.x, y, z - across.z]);
                out.positions.push([x + across.x, y, z + across.z]);
            }
            for j in 0..RINGS as u32 {
                let a = base + j * 2;
                out.indices
                    .extend_from_slice(&[a, a + 2, a + 1, a + 1, a + 2, a + 3]);
            }
        }
    }
}

/// A mesh being built.
#[derive(Default)]
struct MeshParts {
    positions: Vec<[f32; 3]>,
    indices: Vec<u32>,
}

impl MeshParts {
    fn into_mesh(self) -> Mesh {
        let n = self.positions.len();
        Mesh::new(
            PrimitiveTopology::TriangleList,
            RenderAssetUsages::default(),
        )
        .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, self.positions)
        .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, vec![[0.0, 1.0, 0.0]; n])
        .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, vec![[0.0, 0.0]; n])
        .with_inserted_indices(Indices::U32(self.indices))
    }
}

/// The area, the fill reaching `grow` of the way out, and the edge, as meshes.
fn area_mesh(drape: &Drape) -> Mesh {
    let mut out = MeshParts::default();
    drape.band(0.0, 1.0, RINGS, LIFT, &mut out);
    out.into_mesh()
}

fn fill_mesh(drape: &Drape, grow: f32) -> Mesh {
    let mut out = MeshParts::default();
    drape.band(0.0, grow.clamp(0.01, 1.0), RINGS, LIFT + 0.01, &mut out);
    out.into_mesh()
}

fn edge_mesh(drape: &Drape) -> Mesh {
    let mut out = MeshParts::default();
    drape.band(1.0 - EDGE, 1.0, 1, LIFT + 0.02, &mut out);
    drape.sides(drape.radius * EDGE, LIFT + 0.02, &mut out);
    // A line's start is an edge too.
    if drape.line.is_some() {
        drape.band(0.0, EDGE, 1, LIFT + 0.02, &mut out);
    }
    out.into_mesh()
}

/// A mark being drawn.
struct Live {
    /// Where it lands in the world: the centre or the apex, and the cone's facing.
    centre: Vec3,
    facing: Vec3,
    radius: f32,
    /// The cone's half angle, 0 for a circle; a line's half width.
    half_angle: f32,
    /// A line: from `centre` along `facing` for `radius`.
    line: bool,
    rgb: [f32; 3],
    caster: u64,
    born: f64,
    wind_up: f32,
    /// When the server broke it off.
    broken_at: Option<f64>,
    /// The landing was seen (the kick is given once).
    landed: bool,
    /// The caster was seen alive during the wind-up: only then does its death break the mark (a
    /// champion's death burst is its corpse's).
    caster_seen_alive: bool,
    parts: Vec<Entity>,
    /// The parts' own materials and meshes, freed with the mark whatever became of the parts.
    materials: Vec<Handle<StandardMaterial>>,
    meshes: Vec<Handle<Mesh>>,
    /// The ground under it, the fill's mesh, and the reach that mesh was built at.
    drape: Drape,
    fill: Handle<Mesh>,
    fill_at: f32,
}

#[derive(Resource, Default)]
struct LiveTelegraphs(HashMap<u32, Live>);

/// The WoW orientation `o` at WoW point `at`, as a flat Bevy direction.
fn bevy_facing(at: [f32; 3], o: f32) -> Vec3 {
    let a = benilla_assets::coords::wow_to_bevy(at);
    let b = benilla_assets::coords::wow_to_bevy([at[0] + o.cos(), at[1] + o.sin(), at[2]]);
    let d = b - a;
    Vec3::new(d.x, 0.0, d.z).normalize_or(Vec3::NEG_Z)
}

/// Whether `point` stands inside the mark (its flat footprint, the body's reach added).
fn inside(live: &Live, point: Vec3, reach: f32) -> bool {
    let d = Vec3::new(point.x - live.centre.x, 0.0, point.z - live.centre.z);
    if live.line {
        let along = d.dot(live.facing);
        let across = d.dot(Vec3::new(-live.facing.z, 0.0, live.facing.x));
        return (point.y - live.centre.y).abs() <= 6.0
            && along >= -reach
            && along <= live.radius + reach
            && across.abs() <= live.half_angle + reach;
    }
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

/// Start each new mark, set down on the ground under it, and break off the ones the server says.
#[allow(clippy::too_many_arguments)]
fn take_telegraphs(
    mut commands: Commands,
    time: Res<Time>,
    mut state: ResMut<ArpgTelegraphs>,
    mut live: ResMut<LiveTelegraphs>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    spatial: avian3d::prelude::SpatialQuery,
    occluders: Query<(), With<benilla_world::collision::PickOccluder>>,
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
        let line = t.shape == TELEGRAPH_LINE && t.half_angle > 0.0 && t.half_angle < 20.0;
        let half_angle = if cone {
            t.half_angle.min(std::f32::consts::PI)
        } else if line {
            t.half_angle
        } else {
            0.0
        };
        let centre = benilla_assets::coords::wow_to_bevy(t.pos);
        let facing = bevy_facing(t.pos, t.orientation);
        let rgb = GRADE_RGB[usize::from(t.grade).min(GRADE_RGB.len() - 1)];
        // Each grid point down on the floor or terrain under it, walking out from the centre.
        let ground = |p: Vec3, from_y: f32| {
            let from = Vec3::new(p.x, from_y, p.z);
            spatial
                .cast_ray_predicate(
                    from,
                    Dir3::NEG_Y,
                    PROBE_DOWN,
                    true,
                    &benilla_world::collision::WorldCollision::body_filter(),
                    &|e| occluders.contains(e),
                )
                .map(|hit| from.y - hit.distance)
        };
        let drape = if line {
            Drape::line(centre, t.radius, facing, half_angle, ground)
        } else {
            Drape::new(centre, t.radius, facing, half_angle, ground)
        };
        let area = meshes.add(area_mesh(&drape));
        let fill = meshes.add(fill_mesh(&drape, 0.01));
        let edge = meshes.add(edge_mesh(&drape));
        let at = Transform::from_translation(Vec3::new(centre.x, 0.0, centre.z));
        let mut parts = Vec::new();
        let mut mats = Vec::new();
        for (part, mesh, alpha) in [
            (Part::Area, area.clone(), AREA_ALPHA),
            (Part::Fill, fill.clone(), FILL_ALPHA),
            (Part::Edge, edge.clone(), EDGE_ALPHA),
        ] {
            let mat = material(&mut materials, rgb, alpha);
            mats.push(mat.clone());
            parts.push(
                commands
                    .spawn((
                        Mesh3d(mesh),
                        MeshMaterial3d(mat.clone()),
                        at,
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
                line,
                rgb,
                caster: t.caster,
                born: now,
                wind_up: (t.wind_up_ms as f32 / 1000.0).clamp(0.1, 10.0),
                broken_at: None,
                landed: false,
                caster_seen_alive: false,
                parts,
                materials: mats,
                meshes: vec![area, fill.clone(), edge],
                drape,
                fill,
                fill_at: 0.01,
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
    parts: Query<&TelegraphPart>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
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
            let health = index
                .0
                .get(&l.caster)
                .and_then(|e| stores.get(*e).ok())
                .and_then(|s| s.0.unit_health());
            match health {
                Some(0) if l.caster_seen_alive => l.broken_at = Some(now),
                Some(h) if h > 0 => l.caster_seen_alive = true,
                _ => {}
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
                if (grow - l.fill_at).abs() >= FILL_STEP {
                    if let Some(mesh) = meshes.get_mut(&l.fill) {
                        *mesh = fill_mesh(&l.drape, grow);
                    }
                    l.fill_at = grow;
                }
                for e in &l.parts {
                    let Ok(part) = parts.get(*e) else {
                        continue;
                    };
                    let alpha = match part.part {
                        Part::Area => alphas[0],
                        Part::Fill => alphas[1],
                        Part::Edge => alphas[2],
                    };
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
            for m in &l.meshes {
                meshes.remove(m);
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
        let drape = Drape::new(Vec3::ZERO, 10.0, Vec3::NEG_Z, half_angle, |_, _| Some(0.0));
        Live {
            centre: Vec3::ZERO,
            facing: Vec3::NEG_Z,
            radius: 10.0,
            half_angle,
            line: false,
            rgb: GRADE_RGB[0],
            caster: 1,
            born: 0.0,
            wind_up: 1.5,
            broken_at: None,
            landed: false,
            caster_seen_alive: true,
            parts: Vec::new(),
            materials: Vec::new(),
            meshes: Vec::new(),
            drape,
            fill: Handle::default(),
            fill_at: 0.0,
        }
    }

    #[test]
    fn a_mark_lies_on_the_slope_under_it() {
        // A hill rising one yard per yard toward +X, found only from above it.
        let hill = |p: Vec3, from: f32| (from >= p.x).then_some(p.x);
        let drape = Drape::new(Vec3::ZERO, 8.0, Vec3::NEG_Z, 0.0, hill);
        for s in 0..drape.spokes.len() {
            let [x, y, _] = drape.point(1.0, s, 0.0);
            assert!((y - x).abs() < 1e-3, "spoke {s}: {x} at {y}");
            // Half way out, between rings, still on the hill.
            let [x, y, _] = drape.point(0.55, s, 0.0);
            assert!((y - x).abs() < 1e-3);
        }
    }

    #[test]
    fn a_swimmers_mark_stays_at_the_surface_and_a_ceiling_is_not_the_floor() {
        // A lake bed 5 yards under the caster: the centre keeps the caster's height.
        let lake = Drape::new(Vec3::ZERO, 4.0, Vec3::NEG_Z, 0.0, |_, _| Some(-5.0));
        assert_eq!(lake.heights[0], 0.0);
        // A ceiling 3 yards up and the floor at 0: the probes start under the ceiling.
        let room = |_: Vec3, from: f32| Some(if from > 3.0 { 3.0 } else { 0.0 });
        let drape = Drape::new(Vec3::ZERO, 6.0, Vec3::NEG_Z, 0.0, room);
        assert!(drape.heights.iter().all(|h| *h == 0.0));
    }

    #[test]
    fn a_circle_joins_round_and_a_cone_keeps_its_edges() {
        let ring = Drape::new(Vec3::ZERO, 5.0, Vec3::NEG_Z, 0.0, |_, _| Some(0.0));
        assert!(ring.closed && ring.spokes.len() == CIRCLE_SPOKES);
        let cone = Drape::new(
            Vec3::ZERO,
            5.0,
            Vec3::NEG_Z,
            std::f32::consts::FRAC_PI_4,
            |_, _| Some(0.0),
        );
        assert!(!cone.closed);
        let first = cone.spokes[0];
        let last = *cone.spokes.last().unwrap();
        assert!((first.angle_between(Vec3::NEG_Z) - std::f32::consts::FRAC_PI_4).abs() < 1e-4);
        assert!((last.angle_between(Vec3::NEG_Z) - std::f32::consts::FRAC_PI_4).abs() < 1e-4);
        // The area: RINGS bands of quads between the spokes; the edge adds the two sides.
        let mut out = MeshParts::default();
        cone.band(0.0, 1.0, RINGS, 0.0, &mut out);
        assert_eq!(out.indices.len(), RINGS * (cone.spokes.len() - 1) * 6);
        let mut sides = MeshParts::default();
        cone.sides(0.3, 0.0, &mut sides);
        assert_eq!(sides.indices.len(), 2 * RINGS * 6);
    }

    #[test]
    fn a_line_runs_from_its_start_and_holds_what_is_on_it() {
        let drape = Drape::line(Vec3::ZERO, 8.0, Vec3::NEG_Z, 1.0, |_, _| Some(0.0));
        let [x, _, z] = drape.point(1.0, LINE_COLUMNS / 2, 0.0);
        assert!(x.abs() < 1e-5 && (z + 8.0).abs() < 1e-5);
        let [x, _, _] = drape.point(0.0, 0, 0.0);
        assert!((x.abs() - 1.0).abs() < 1e-5);
        let mut lane = live(0.0);
        lane.line = true;
        lane.radius = 8.0;
        lane.half_angle = 1.0;
        assert!(inside(&lane, Vec3::new(0.5, 0.0, -6.0), 0.4));
        assert!(!inside(&lane, Vec3::new(2.0, 0.0, -6.0), 0.4));
        assert!(!inside(&lane, Vec3::new(0.0, 0.0, 3.0), 0.4));
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
}
