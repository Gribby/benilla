//! Fork-only, not 1.12.1: the feed half of the ARPG view's dodge roll and health flask
//! (`crate::player::arpg::actions`): the health orb's flask and roll line (`ArpgHud_Status` in
//! `arpg_hud.lua`), the combo pips and the minions beside it.

use bevy::prelude::*;

use benilla_protocol::messages::ArpgStatus;
use benilla_ui::script::UiScript;

use crate::player::arpg::actions::ArpgActionState;
use crate::player::arpg::champions::ArpgMinionsHud;

/// Fork-only, not 1.12.1: the combo points over the power orb (`ArpgHud_Combo` in
/// `arpg_hud.lua`). An ARPG character's points are its own, carried from enemy to enemy (cmangos
/// `Arpg/ArpgDruid.h`), so the pips show the count whatever is targeted; stock
/// `GetComboPoints` counts only the points banked on the current target.
pub(crate) fn push_combo(
    script: Option<NonSendMut<UiScript>>,
    me: Query<&crate::net::ObjectStore, With<crate::net::SelfPlayer>>,
    mut pushed: Local<crate::ui_script::VmMemo<Option<u8>>>,
) {
    let (Some(script), Ok(store)) = (script, me.single()) else {
        return;
    };
    let points = store.0.player_combo_points().unwrap_or(0).min(5);
    let last = pushed.get(&script);
    if *last == Some(points) {
        return;
    }
    match script.eval::<bool>(&combo_chunk(points)) {
        Ok(true) => *last = Some(points),
        Ok(false) => {}
        Err(e) => {
            warn!("arpg: the orb refused the combo points: {e}");
            *last = Some(points);
        }
    }
}

/// Fork-only, not 1.12.1: the demon's health and the horde's count beside the health orb
/// (`ArpgHud_Minions` in `arpg_hud.lua`), when they change.
pub(crate) fn push_minions(
    script: Option<NonSendMut<UiScript>>,
    hud: Res<ArpgMinionsHud>,
    mut pushed: Local<
        crate::ui_script::VmMemo<Option<benilla_protocol::messages::arpg::ArpgMinions>>,
    >,
) {
    let Some(script) = script else {
        return;
    };
    let last = pushed.get(&script);
    if last.as_ref() == Some(&hud.0) {
        return;
    }
    match script.eval::<bool>(&minions_chunk(&hud.0)) {
        Ok(true) => *last = Some(hud.0.clone()),
        Ok(false) => {}
        Err(e) => {
            warn!("arpg: the HUD refused the minions: {e}");
            *last = Some(hud.0.clone());
        }
    }
}

fn minions_chunk(m: &benilla_protocol::messages::arpg::ArpgMinions) -> String {
    // The name goes over as a Lua string: only its letters, spaces and apostrophes.
    let name: String = m
        .demon_name
        .chars()
        .filter(|c| c.is_alphanumeric() || *c == ' ' || *c == '\'')
        .collect();
    format!(
        "if not ArpgHud_Minions then return false end\nArpgHud_Minions({}, {}, {}, {}, \"{}\")\nreturn true",
        u8::from(m.demon),
        m.demon_health,
        m.horde,
        m.horde_cap,
        name
    )
}

fn combo_chunk(points: u8) -> String {
    format!("if not ArpgHud_Combo then return false end\nArpgHud_Combo({points})\nreturn true")
}

/// The Lua chunk that tells the health orb about the flask and the roll.
fn status_chunk(s: &ArpgStatus) -> String {
    format!(
        "if not ArpgHud_Status then return false end\nArpgHud_Status({}, {}, {}, {}, {})\nreturn true",
        s.flask_charges, s.flask_max, s.flask_progress, s.dodge_ready_ms, s.dodge_cooldown_ms
    )
}

/// Tell the orb each new status.
pub(crate) fn push_status(
    script: Option<NonSendMut<UiScript>>,
    state: Res<ArpgActionState>,
    mut pushed: Local<Option<(u64, u64)>>,
) {
    let (Some(script), Some(status)) = (script, state.status()) else {
        return;
    };
    let mark = (script.session(), state.generation());
    if *pushed == Some(mark) {
        return;
    }
    match script.eval::<bool>(&status_chunk(&status)) {
        Ok(true) => *pushed = Some(mark),
        Ok(false) => {}
        Err(e) => {
            warn!("arpg: the orb refused the status: {e}");
            *pushed = Some(mark);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_status_chunk_calls_the_orb_with_every_field() {
        let chunk = status_chunk(&ArpgStatus {
            flask_charges: 2,
            flask_max: 3,
            flask_progress: 40,
            dodge_ready_ms: 900,
            dodge_cooldown_ms: 2500,
        });
        assert!(chunk.contains("ArpgHud_Status(2, 3, 40, 900, 2500)"));
        let script = UiScript::new().unwrap();
        script
            .run("function ArpgHud_Status(c, m, p, r, d) GOT = c + m + p + r + d end")
            .unwrap();
        assert!(script.eval::<bool>(&chunk).unwrap());
        assert_eq!(
            script.eval::<u32>("return GOT").unwrap(),
            2 + 3 + 40 + 900 + 2500
        );
        assert!(!UiScript::new().unwrap().eval::<bool>(&chunk).unwrap());
    }
}
