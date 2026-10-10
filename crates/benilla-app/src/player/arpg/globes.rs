//! Fork-only, not 1.12.1: the ARPG server's health globes (cmangos `Arpg/ArpgActions.h`). A kill
//! sometimes drops one where the creature fell (`SMSG_ARPG_GLOBE`): a red orb that bobs and
//! glows over the ground. Walking onto it takes it (`ClientCommand::ArpgGlobe`), and the server
//! heals the taker a fifth of their health; it fades after a minute either way.

use std::collections::HashMap;

use benilla_protocol::messages::arpg::{GLOBE_DROPPED, GLOBE_GONE};
use benilla_protocol::messages::ArpgGlobe;
use benilla_protocol::{SessionEvent, SessionEventKind};

use crate::net::{NetHandlerApp, SelfPlayer};

use super::*;

/// How near the player's feet take a globe, in yards, and how often a take is asked again.
const TAKE_REACH: f32 = 1.6;
const TAKE_RETRY: f64 = 0.5;
/// How long one lies there (the server's, a little short so a stale one never lingers).
const LIFE_SECS: f64 = 58.0;
/// Its look: radius, height over the ground, bob, and the halo's size.
const ORB_RADIUS: f32 = 0.32;
const ORB_LIFT: f32 = 0.7;
const ORB_BOB: f32 = 0.12;
const HALO_RADIUS: f32 = 0.75;
const ORB_RGB: [f32; 3] = [1.0, 0.08, 0.06];

/// The globes the server sent, waiting for the view.
#[derive(Resource, Default)]
pub(crate) struct ArpgGlobes {
    incoming: Vec<ArpgGlobe>,
}

/// Registered whatever the view, so every session event kind has its owner.
pub(super) fn register_net(app: &mut App) {
    app.init_resource::<ArpgGlobes>()
        .net_handler(SessionEventKind::ArpgGlobe, on_globe);
}

fn on_globe(In(ev): In<SessionEvent>, mut state: ResMut<ArpgGlobes>) {
    if let SessionEvent::ArpgGlobe { globe } = ev {
        state.incoming.push(globe);
    }
}

pub(super) fn plugin(app: &mut App) {
    app.init_resource::<LiveGlobes>()
        .add_systems(Update, (take_globes, run_globes).chain());
}

/// One globe on the ground.
struct Live {
    at: Vec3,
    born: f64,
    asked_at: Option<f64>,
    orb: Entity,
    halo: Entity,
}

#[derive(Resource, Default)]
struct LiveGlobes(HashMap<u32, Live>);

/// The meshes and materials, made once.
type GlobeArt = (
    Handle<Mesh>,
    Handle<Mesh>,
    Handle<StandardMaterial>,
    Handle<StandardMaterial>,
);

/// Lay out the new globes and take away the gone ones.
fn take_globes(
    mut commands: Commands,
    time: Res<Time>,
    mut state: ResMut<ArpgGlobes>,
    mut live: ResMut<LiveGlobes>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut art: Local<Option<GlobeArt>>,
    mut entered: MessageReader<crate::net::EnteredWorldMessage>,
    mut ports: MessageReader<crate::net::WorldportMessage>,
) {
    let now = time.elapsed_secs_f64();
    // A new map (a portal, a hearth, another character): the old map's globes are gone.
    let moved = entered.read().count() + ports.read().count() > 0;
    if moved {
        for (_, l) in live.0.drain() {
            despawn(&mut commands, &l);
        }
        state.incoming.clear();
    }
    for g in std::mem::take(&mut state.incoming) {
        if g.kind == GLOBE_GONE {
            if let Some(l) = live.0.remove(&g.id) {
                despawn(&mut commands, &l);
            }
            continue;
        }
        if g.kind != GLOBE_DROPPED
            || live.0.contains_key(&g.id)
            || !g.pos.iter().all(|v| v.is_finite())
        {
            continue;
        }
        let (orb_mesh, halo_mesh, orb_mat, halo_mat) = art
            .get_or_insert_with(|| {
                let [r, gr, b] = ORB_RGB;
                (
                    meshes.add(Sphere::new(ORB_RADIUS)),
                    meshes.add(Circle::new(HALO_RADIUS)),
                    materials.add(StandardMaterial {
                        base_color: Color::srgba(r, gr, b, 0.9),
                        emissive: LinearRgba::rgb(r * 3.0, gr * 3.0, b * 3.0),
                        unlit: true,
                        alpha_mode: AlphaMode::Blend,
                        ..default()
                    }),
                    materials.add(StandardMaterial {
                        base_color: Color::srgba(r, gr, b, 0.35),
                        unlit: true,
                        alpha_mode: AlphaMode::Add,
                        cull_mode: None,
                        ..default()
                    }),
                )
            })
            .clone();
        let at = benilla_assets::coords::wow_to_bevy(g.pos);
        let orb = commands
            .spawn((
                Mesh3d(orb_mesh),
                MeshMaterial3d(orb_mat),
                Transform::from_translation(at + Vec3::Y * ORB_LIFT),
            ))
            .id();
        let halo = commands
            .spawn((
                Mesh3d(halo_mesh),
                MeshMaterial3d(halo_mat),
                Transform::from_translation(at + Vec3::Y * 0.08)
                    .with_rotation(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2)),
            ))
            .id();
        live.0.insert(
            g.id,
            Live {
                at,
                born: now,
                asked_at: None,
                orb,
                halo,
            },
        );
    }
}

fn despawn(commands: &mut Commands, l: &Live) {
    for e in [l.orb, l.halo] {
        if let Ok(mut ec) = commands.get_entity(e) {
            ec.despawn();
        }
    }
}

/// The orb's height over its ground `age` seconds in: a slow bob.
fn bob_at(age: f32) -> f32 {
    ORB_LIFT + ORB_BOB * (age * 2.4).sin()
}

/// Bob the globes, take the one the player walks onto, and let the old ones fade away.
fn run_globes(
    mut commands: Commands,
    time: Res<Time>,
    mut live: ResMut<LiveGlobes>,
    mut transforms: Query<&mut Transform, Without<SelfPlayer>>,
    me: Query<&Transform, With<SelfPlayer>>,
    net: Res<NetCommands>,
) {
    let now = time.elapsed_secs_f64();
    let feet = me.single().ok().map(|t| t.translation);
    let mut spent = Vec::new();
    for (id, l) in live.0.iter_mut() {
        let age = now - l.born;
        if age >= LIFE_SECS {
            spent.push(*id);
            continue;
        }
        if let Ok(mut tf) = transforms.get_mut(l.orb) {
            tf.translation = l.at + Vec3::Y * bob_at(age as f32);
        }
        let near = feet.is_some_and(|p| {
            Vec2::new(p.x - l.at.x, p.z - l.at.z).length() <= TAKE_REACH
                && (p.y - l.at.y).abs() < 3.0
        });
        if near && l.asked_at.is_none_or(|at| now - at >= TAKE_RETRY) {
            l.asked_at = Some(now);
            let _ = net.0.send(ClientCommand::ArpgGlobe { id: *id });
        }
    }
    for id in spent {
        if let Some(l) = live.0.remove(&id) {
            despawn(&mut commands, &l);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_globe_bobs_over_its_ground() {
        for i in 0..100 {
            let h = bob_at(i as f32 * 0.07);
            assert!((ORB_LIFT - ORB_BOB - 1e-4..=ORB_LIFT + ORB_BOB + 1e-4).contains(&h));
        }
    }
}
