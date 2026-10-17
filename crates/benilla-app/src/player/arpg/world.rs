//! Fork-only, not 1.12.1: the ARPG server's living world and crafting (cmangos
//! `Arpg/ArpgEvents.h` and `Arpg/ArpgCraft.h`).
//!
//! A zone event (`SMSG_ARPG_EVENT`) is shown two ways: a tall orange column of light over its spot,
//! so it can be found across the zone, and the HUD addon's tracker (`ArpgEvent_Update`), which this
//! feeds twice a second with the objective, its progress, the time left, and how far and which way
//! the spot lies from the player. The bounties (`SMSG_ARPG_BOUNTIES`) go to the same tracker as
//! `ArpgBounties_Update`, and the crafting purse (`SMSG_ARPG_PURSE`) to the Forge window as
//! `ArpgPurse_Update` (the feed is `crate::ui_arpg::world`).
//!
//! The Forge answers through the session-only setting `arpgCraft`, "op:bag:slot:affix:nonce"
//! ([`parse_craft`]), turned into `ClientCommand::ArpgCraft`; a bounty is abandoned through the
//! tree's action setting (kind 13).

use benilla_protocol::messages::arpg::EVENT_LIVE;
use benilla_protocol::messages::{ArpgBounty, ArpgEvent, ArpgPurse};
use benilla_protocol::{SessionEvent, SessionEventKind};
use benilla_world::view::WorldCamera;

use crate::net::{ClientCommand, NetCommands, NetHandlerApp};

use super::*;

/// The Forge's action setting (session-owned, never saved).
pub(crate) const CVAR_CRAFT: &str = "arpgCraft";

/// The column over a live event's spot: height and half width in yards, and its colour.
const BEAM_HEIGHT: f32 = 70.0;
const BEAM_WIDTH: f32 = 2.4;
const BEAM_RGBA: [f32; 4] = [1.0, 0.5, 0.05, 0.55];

/// What the server last sent, and what the addon has been given.
#[derive(Resource, Default)]
pub(crate) struct ArpgWorldState {
    /// The zone's event and when it arrived (the clock counts down from then).
    pub(crate) event: Option<(ArpgEvent, f64)>,
    /// The arrival still to be stamped with the time (the handler has no clock).
    pub(crate) fresh: bool,
    pub(crate) event_generation: u64,
    pub(crate) event_pushed: Option<(u64, u64)>,
    pub(crate) event_pushed_at: f64,
    pub(crate) bounties: Option<Vec<ArpgBounty>>,
    pub(crate) bounties_generation: u64,
    pub(crate) bounties_pushed: Option<(u64, u64)>,
    pub(crate) purse: Option<ArpgPurse>,
    pub(crate) purse_generation: u64,
    pub(crate) purse_pushed: Option<(u64, u64)>,
}

/// Registered whatever the view, so every session event kind has its owner.
pub(super) fn register_net(app: &mut App) {
    app.init_resource::<ArpgWorldState>()
        .net_handler(SessionEventKind::ArpgEvent, on_event)
        .net_handler(SessionEventKind::ArpgBounties, on_bounties)
        .net_handler(SessionEventKind::ArpgPurse, on_purse);
}

fn on_event(In(ev): In<SessionEvent>, mut state: ResMut<ArpgWorldState>) {
    if let SessionEvent::ArpgEvent { event } = ev {
        state.event =
            (event.state != benilla_protocol::messages::arpg::EVENT_GONE).then_some((event, 0.0));
        state.fresh = true;
        state.event_generation += 1;
    }
}

fn on_bounties(In(ev): In<SessionEvent>, mut state: ResMut<ArpgWorldState>) {
    if let SessionEvent::ArpgBounties { bounties } = ev {
        state.bounties = Some(bounties);
        state.bounties_generation += 1;
    }
}

fn on_purse(In(ev): In<SessionEvent>, mut state: ResMut<ArpgWorldState>) {
    if let SessionEvent::ArpgPurse { purse } = ev {
        state.purse = Some(purse);
        state.purse_generation += 1;
    }
}

pub(super) fn plugin(app: &mut App) {
    app.add_systems(Update, (crate::ui_arpg::world::push_world, run_beam))
        .add_observer(on_craft);
}

/// The compass word for a flat WoW-space offset (`dx` north, `dy` west).
pub(crate) fn compass(dx: f32, dy: f32) -> &'static str {
    const WORDS: [&str; 8] = ["N", "NW", "W", "SW", "S", "SE", "E", "NE"];
    // Counter-clockwise from north, as WoW's own angles turn.
    let angle = dy.atan2(dx).rem_euclid(std::f32::consts::TAU);
    let sector = ((angle / std::f32::consts::FRAC_PI_4) + 0.5).floor() as usize % 8;
    WORDS[sector]
}

/// The column over the live event's spot, and the event id it stands for.
#[derive(Component)]
struct EventBeam(u32);

/// Raise the column over a live event, take it away when the event ends, and turn it to the
/// camera about its upright.
fn run_beam(
    mut commands: Commands,
    state: Res<ArpgWorldState>,
    mut beams: Query<(Entity, &EventBeam, &mut Transform)>,
    camera: Query<&GlobalTransform, With<WorldCamera>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut art: Local<Option<(Handle<Mesh>, Handle<StandardMaterial>)>>,
) {
    let want = state
        .event
        .as_ref()
        .filter(|(e, _)| e.state == EVENT_LIVE && e.pos.iter().all(|v| v.is_finite()))
        .map(|(e, _)| e);
    let mut have = false;
    for (entity, beam, _) in &beams {
        if want.is_some_and(|e| e.id == beam.0) {
            have = true;
        } else {
            commands.entity(entity).despawn();
        }
    }
    if let (Some(e), false) = (want, have) {
        let (mesh, material) = art
            .get_or_insert_with(|| {
                let [r, g, b, a] = BEAM_RGBA;
                (
                    meshes.add(super::loot::beam_mesh()),
                    materials.add(StandardMaterial {
                        base_color: Color::srgba(r, g, b, a),
                        unlit: true,
                        alpha_mode: AlphaMode::Add,
                        cull_mode: None,
                        ..default()
                    }),
                )
            })
            .clone();
        let foot = benilla_assets::coords::wow_to_bevy(e.pos);
        commands.spawn((
            EventBeam(e.id),
            Mesh3d(mesh),
            MeshMaterial3d(material),
            Transform::from_translation(foot + Vec3::Y * BEAM_HEIGHT * 0.5).with_scale(Vec3::new(
                BEAM_WIDTH,
                BEAM_HEIGHT,
                1.0,
            )),
        ));
    }
    let Some(eye) = camera.single().ok().map(|c| c.translation()) else {
        return;
    };
    for (_, _, mut tf) in &mut beams {
        let d = eye - tf.translation;
        if d.x * d.x + d.z * d.z > 1e-4 {
            tf.rotation = Quat::from_rotation_y(d.x.atan2(d.z));
        }
    }
}

/// A Forge request: the craft op, the item's bag and slot, and a shard's affix.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Craft {
    op: u8,
    bag: u8,
    slot: u8,
    affix: u16,
}

/// "op:bag:slot:affix:nonce"; the nonce only moves the setting, so a repeat still sends.
fn parse_craft(value: &str) -> Option<Craft> {
    let mut parts = value.trim().split(':');
    let op = parts.next()?.parse().ok()?;
    let bag = parts.next()?.parse().ok()?;
    let slot = parts.next()?.parse().ok()?;
    let affix = parts.next()?.parse().ok()?;
    parts.next()?;
    ((1..=9).contains(&op) && slot >= 1).then_some(Craft {
        op,
        bag,
        slot,
        affix,
    })
}

/// The Forge's setting moved: send the craft.
fn on_craft(ev: On<crate::cvars::CvarChanged>, net: Res<NetCommands>) {
    if !ev.is(CVAR_CRAFT) {
        return;
    }
    if let Some(c) = parse_craft(&ev.new) {
        let _ = net.0.send(ClientCommand::ArpgCraft {
            op: c.op,
            bag: c.bag,
            slot: c.slot,
            affix: c.affix,
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_compass_names_the_eight_ways() {
        assert_eq!(compass(10.0, 0.0), "N");
        assert_eq!(compass(0.0, -10.0), "E");
        assert_eq!(compass(-10.0, 0.0), "S");
        assert_eq!(compass(0.0, 10.0), "W");
        assert_eq!(compass(10.0, -10.0), "NE");
        assert_eq!(compass(-10.0, 10.0), "SW");
    }

    #[test]
    fn crafts_parse_and_junk_does_not() {
        assert_eq!(
            parse_craft("9:255:16:7:3"),
            Some(Craft {
                op: 9,
                bag: 255,
                slot: 16,
                affix: 7
            })
        );
        assert_eq!(
            parse_craft("4:0:3:0:1"),
            Some(Craft {
                op: 4,
                bag: 0,
                slot: 3,
                affix: 0
            })
        );
        assert_eq!(parse_craft("0"), None);
        assert_eq!(parse_craft("12:0:1:0:1"), None);
        assert_eq!(parse_craft("4:0:0:0:1"), None);
        assert_eq!(parse_craft("4:0:3:0"), None);
    }
}
