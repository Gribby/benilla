//! Fork-only, not 1.12.1: the ARPG server's living world and crafting (cmangos
//! `Arpg/ArpgEvents.h` and `Arpg/ArpgCraft.h`).
//!
//! A zone event (`SMSG_ARPG_EVENT`) is shown two ways: a tall orange column of light over its spot,
//! so it can be found across the zone, and the HUD addon's tracker (`ArpgEvent_Update`), which this
//! feeds twice a second with the objective, its progress, the time left, and how far and which way
//! the spot lies from the player. The bounties (`SMSG_ARPG_BOUNTIES`) go to the same tracker as
//! `ArpgBounties_Update`, and the crafting purse (`SMSG_ARPG_PURSE`) to the Forge window as
//! `ArpgPurse_Update`.
//!
//! The Forge answers through the session-only setting `arpgCraft`, "op:bag:slot:affix:nonce"
//! ([`parse_craft`]), turned into `ClientCommand::ArpgCraft`; a bounty is abandoned through the
//! tree's action setting (kind 13).

use benilla_protocol::messages::arpg::EVENT_LIVE;
use benilla_protocol::messages::{ArpgBounty, ArpgEvent, ArpgPurse};
use benilla_protocol::{SessionEvent, SessionEventKind};
use benilla_ui::script::UiScript;
use benilla_world::view::WorldCamera;

use crate::net::{ClientCommand, NetCommands, NetHandlerApp, SelfPlayer};

use super::tree::lua_str;
use super::*;

/// The Forge's action setting (session-owned, never saved).
pub(crate) const CVAR_CRAFT: &str = "arpgCraft";

/// How often the tracker hears the distance and the clock, in seconds.
const TRACK_EVERY: f64 = 0.5;
/// The column over a live event's spot: height and half width in yards, and its colour.
const BEAM_HEIGHT: f32 = 70.0;
const BEAM_WIDTH: f32 = 2.4;
const BEAM_RGBA: [f32; 4] = [1.0, 0.5, 0.05, 0.55];

/// What the server last sent, and what the addon has been given.
#[derive(Resource, Default)]
pub(crate) struct ArpgWorldState {
    /// The zone's event and when it arrived (the clock counts down from then).
    event: Option<(ArpgEvent, f64)>,
    /// The arrival still to be stamped with the time (the handler has no clock).
    fresh: bool,
    event_generation: u64,
    event_pushed: Option<(u64, u64)>,
    event_pushed_at: f64,
    bounties: Option<Vec<ArpgBounty>>,
    bounties_generation: u64,
    bounties_pushed: Option<(u64, u64)>,
    purse: Option<ArpgPurse>,
    purse_generation: u64,
    purse_pushed: Option<(u64, u64)>,
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
    app.add_systems(Update, (push_world, run_beam))
        .add_observer(on_craft);
}

/// The compass word for a flat WoW-space offset (`dx` north, `dy` west).
fn compass(dx: f32, dy: f32) -> &'static str {
    const WORDS: [&str; 8] = ["N", "NW", "W", "SW", "S", "SE", "E", "NE"];
    // Counter-clockwise from north, as WoW's own angles turn.
    let angle = dy.atan2(dx).rem_euclid(std::f32::consts::TAU);
    let sector = ((angle / std::f32::consts::FRAC_PI_4) + 0.5).floor() as usize % 8;
    WORDS[sector]
}

/// The Lua chunk that hands the tracker the event, or clears it (`None`); it returns whether the
/// addon took it.
fn event_chunk(event: Option<(&ArpgEvent, u32, f32, &str)>) -> String {
    let body = match event {
        None => "nil".to_string(),
        Some((e, secs, dist, dir)) => format!(
            "{{state={},kind={},title={},objective={},have={},need={},secs={},dist={},dir={}}}",
            e.state,
            e.kind,
            lua_str(&e.title),
            lua_str(&e.objective),
            e.have,
            e.need,
            secs,
            dist.round() as i64,
            lua_str(dir),
        ),
    };
    format!("if not ArpgEvent_Update then return false end\nArpgEvent_Update({body})\nreturn true")
}

fn bounties_chunk(bounties: &[ArpgBounty]) -> String {
    let rows: Vec<String> = bounties
        .iter()
        .map(|b| {
            format!(
                "{{slot={},kind={},have={},need={},title={},reward={}}}",
                b.slot,
                b.kind,
                b.have,
                b.need,
                lua_str(&b.title),
                lua_str(&b.reward)
            )
        })
        .collect();
    format!(
        "if not ArpgBounties_Update then return false end\nArpgBounties_Update({{{}}})\nreturn true",
        rows.join(",")
    )
}

fn purse_chunk(purse: &ArpgPurse) -> String {
    let orbs: Vec<String> = purse
        .orbs
        .iter()
        .map(|o| {
            format!(
                "{{id={},held={},name={},text={}}}",
                o.id,
                o.held,
                lua_str(&o.name),
                lua_str(&o.text)
            )
        })
        .collect();
    let shards: Vec<String> = purse
        .shards
        .iter()
        .map(|s| {
            format!(
                "{{affix={},held={},name={}}}",
                s.affix,
                s.held,
                lua_str(&s.name)
            )
        })
        .collect();
    format!(
        "if not ArpgPurse_Update then return false end\n\
         ArpgPurse_Update({{orbs={{{}}},shards={{{}}}}})\nreturn true",
        orbs.join(","),
        shards.join(",")
    )
}

/// Run `chunk`, and say whether it should count as delivered: taken, or refused for good.
fn deliver(script: &UiScript, chunk: &str, what: &str) -> bool {
    match script.eval::<bool>(chunk) {
        Ok(taken) => taken,
        Err(e) => {
            warn!("arpg: the HUD refused the {what}: {e}");
            true
        }
    }
}

/// Hand the addon the event (and its clock and bearing, twice a second), the bounties and the
/// purse, once per change and VM.
fn push_world(
    script: Option<NonSendMut<UiScript>>,
    mut state: ResMut<ArpgWorldState>,
    time: Res<Time>,
    me: Query<&Transform, With<SelfPlayer>>,
    mut entered: MessageReader<crate::net::EnteredWorldMessage>,
) {
    let now = time.elapsed_secs_f64();
    // A new world entry (another character, a relog): the old event is not this one's; the
    // server's hello sends the zone's anew.
    if entered.read().count() > 0 && state.event.is_some() {
        state.event = None;
        state.event_generation += 1;
    }
    if state.fresh {
        state.fresh = false;
        if let Some((_, at)) = state.event.as_mut() {
            *at = now;
        }
    }
    let Some(script) = script else {
        return;
    };
    let vm = script.session();

    let mark = (vm, state.event_generation);
    let live = state
        .event
        .as_ref()
        .is_some_and(|(e, _)| e.state == EVENT_LIVE);
    if state.event_pushed != Some(mark) || (live && now - state.event_pushed_at >= TRACK_EVERY) {
        let feet = me
            .single()
            .ok()
            .map(|t| benilla_assets::coords::bevy_to_wow(t.translation));
        let chunk = match state.event.as_ref() {
            None => event_chunk(None),
            Some((e, at)) => {
                let gone = ((now - at) * 1000.0).max(0.0) as u32;
                let secs = e.ms_left.saturating_sub(gone).div_ceil(1000);
                let (dist, dir) = feet.map_or((0.0, ""), |p| {
                    let (dx, dy) = (e.pos[0] - p[0], e.pos[1] - p[1]);
                    let d = (dx * dx + dy * dy).sqrt();
                    (
                        d,
                        if d <= e.radius {
                            "here"
                        } else {
                            compass(dx, dy)
                        },
                    )
                });
                event_chunk(Some((e, secs, dist, dir)))
            }
        };
        if deliver(&script, &chunk, "zone event") {
            state.event_pushed = Some(mark);
            state.event_pushed_at = now;
        }
    }

    let mark = (vm, state.bounties_generation);
    if state.bounties_pushed != Some(mark) {
        if let Some(b) = state.bounties.as_ref() {
            if deliver(&script, &bounties_chunk(b), "bounties") {
                state.bounties_pushed = Some(mark);
            }
        }
    }

    let mark = (vm, state.purse_generation);
    if state.purse_pushed != Some(mark) {
        if let Some(p) = state.purse.as_ref() {
            if deliver(&script, &purse_chunk(p), "purse") {
                state.purse_pushed = Some(mark);
            }
        }
    }
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

    #[test]
    fn the_chunks_hand_the_addon_its_tables() {
        let script = UiScript::new().expect("a UI VM");
        script
            .run(
                "function ArpgEvent_Update(t) EV = t end \
                 function ArpgBounties_Update(t) BO = t end \
                 function ArpgPurse_Update(t) PU = t end",
            )
            .expect("the stubs load");
        let e = ArpgEvent {
            state: EVENT_LIVE,
            id: 3,
            kind: 1,
            title: "Purge near \"Goldshire\"".into(),
            objective: "Slay 12".into(),
            have: 4,
            need: 12,
            ..Default::default()
        };
        assert!(script
            .eval::<bool>(&event_chunk(Some((&e, 95, 120.4, "NE"))))
            .unwrap());
        assert_eq!(
            script.eval::<String>("return EV.title").unwrap(),
            "Purge near \"Goldshire\""
        );
        assert_eq!(script.eval::<i64>("return EV.dist").unwrap(), 120);
        assert_eq!(script.eval::<String>("return EV.dir").unwrap(), "NE");
        assert!(script.eval::<bool>(&event_chunk(None)).unwrap());
        assert!(script.eval::<bool>("return EV == nil").unwrap());

        let b = vec![ArpgBounty {
            slot: 1,
            kind: 2,
            have: 3,
            need: 20,
            title: "Westfall: slay 20".into(),
            reward: "gold".into(),
        }];
        assert!(script.eval::<bool>(&bounties_chunk(&b)).unwrap());
        assert_eq!(script.eval::<i64>("return BO[1].need").unwrap(), 20);

        let p = ArpgPurse {
            orbs: vec![benilla_protocol::messages::ArpgOrb {
                id: 4,
                held: 2,
                name: "Fel Orb".into(),
                text: "Rerolls".into(),
            }],
            shards: vec![benilla_protocol::messages::ArpgShard {
                affix: 7,
                held: 1,
                name: "Shard of Holy damage".into(),
            }],
        };
        assert!(script.eval::<bool>(&purse_chunk(&p)).unwrap());
        assert_eq!(
            script.eval::<String>("return PU.orbs[1].name").unwrap(),
            "Fel Orb"
        );
        assert_eq!(script.eval::<i64>("return PU.shards[1].affix").unwrap(), 7);
    }
}
