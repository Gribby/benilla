//! Fork-only, not 1.12.1: the feed half of the ARPG server's living world and crafting
//! (`crate::player::arpg::world`): the HUD addon's tracker (`ArpgEvent_Update`, fed twice a
//! second with the objective, its progress, the time left, and how far and which way the spot
//! lies from the player), the bounties (`ArpgBounties_Update`) and the Forge's purse
//! (`ArpgPurse_Update`).

use bevy::prelude::*;

use benilla_protocol::messages::arpg::EVENT_LIVE;
use benilla_protocol::messages::{ArpgBounty, ArpgEvent, ArpgPurse};
use benilla_ui::script::UiScript;

use crate::net::SelfPlayer;
use crate::player::arpg::world::{compass, ArpgWorldState};

use super::tree::lua_str;

/// How often the tracker hears the distance and the clock, in seconds.
const TRACK_EVERY: f64 = 0.5;

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
pub(crate) fn push_world(
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

#[cfg(test)]
mod tests {
    use super::*;

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
