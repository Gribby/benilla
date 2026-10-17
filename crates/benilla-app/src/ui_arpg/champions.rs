//! Fork-only, not 1.12.1: the feed half of the ARPG server's champions and rares
//! (`crate::player::arpg::champions`): the hover bar's champion line (`ArpgHud_Champion` in
//! `arpg_hud.lua`) and states line (`ArpgHud_States`) for the enemy under the cursor.

use bevy::prelude::*;

use benilla_protocol::messages::ArpgChampion;
use benilla_ui::script::UiScript;

use crate::player::arpg::champions::{states_text, ArpgChampions, ArpgEnemyStates};
use crate::target::arpg_soft::ArpgEnemyHover;

use super::tree::lua_str;

/// Tell the hover bar the states of the enemy under the cursor, when they change.
pub(crate) fn push_hover_states(
    script: Option<NonSendMut<UiScript>>,
    hover: Option<Res<ArpgEnemyHover>>,
    states: Res<ArpgEnemyStates>,
    mut pushed: Local<Option<(u64, String)>>,
) {
    let Some(script) = script else {
        return;
    };
    let text = hover
        .and_then(|h| h.0.map(|(_, guid)| guid))
        .and_then(|guid| states.get(guid))
        .map_or_else(String::new, |(mask, daze)| states_text(mask, daze));
    let mark = (script.session(), text);
    if pushed.as_ref() == Some(&mark) {
        return;
    }
    let chunk = format!(
        "if not ArpgHud_States then return false end\nArpgHud_States({})\nreturn true",
        lua_str(&mark.1)
    );
    match script.eval::<bool>(&chunk) {
        Ok(true) => *pushed = Some(mark),
        Ok(false) => {}
        Err(e) => {
            warn!("arpg: the hover bar refused the states: {e}");
            *pushed = Some(mark);
        }
    }
}

/// The Lua chunk that tells the hover bar about `champion` (`None`: an ordinary enemy, or none).
fn hover_chunk(champion: Option<&ArpgChampion>) -> String {
    let call = match champion {
        None => "ArpgHud_Champion(0)".to_string(),
        Some(c) => format!(
            "ArpgHud_Champion({}, {}, {})",
            c.tier,
            lua_str(&c.name),
            lua_str(&c.affixes.join(", "))
        ),
    };
    format!("if not ArpgHud_Champion then return false end\n{call}\nreturn true")
}

/// Tell the hover bar whether the enemy under the cursor is a champion, when that changes.
pub(crate) fn push_hover(
    script: Option<NonSendMut<UiScript>>,
    hover: Option<Res<ArpgEnemyHover>>,
    champions: Res<ArpgChampions>,
    mut pushed: Local<Option<(u64, Option<u64>, u64)>>,
) {
    let Some(script) = script else {
        return;
    };
    let champion = hover
        .and_then(|h| h.0.map(|(_, guid)| guid))
        .and_then(|guid| champions.get(guid));
    let mark = (
        script.session(),
        champion.map(|c| c.guid),
        champions.generation(),
    );
    if *pushed == Some(mark) {
        return;
    }
    match script.eval::<bool>(&hover_chunk(champion)) {
        Ok(true) => *pushed = Some(mark),
        Ok(false) => {}
        Err(e) => {
            warn!("arpg: the hover bar refused a champion: {e}");
            *pushed = Some(mark);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_hover_chunk_names_the_champion_and_its_affixes() {
        let rare = ArpgChampion {
            guid: 7,
            tier: 2,
            name: "Gorefang the Swift".into(),
            affixes: vec!["Extra Fast".into(), "Vampiric".into()],
        };
        let script = UiScript::new().unwrap();
        script
            .run("function ArpgHud_Champion(t, n, a) GOT = { t = t, n = n, a = a } end")
            .unwrap();
        assert!(script.eval::<bool>(&hover_chunk(Some(&rare))).unwrap());
        assert_eq!(script.eval::<u32>("return GOT.t").unwrap(), 2);
        assert_eq!(
            script.eval::<String>("return GOT.n").unwrap(),
            "Gorefang the Swift"
        );
        assert_eq!(
            script.eval::<String>("return GOT.a").unwrap(),
            "Extra Fast, Vampiric"
        );
        assert!(script.eval::<bool>(&hover_chunk(None)).unwrap());
        assert_eq!(script.eval::<u32>("return GOT.t").unwrap(), 0);
        assert!(!UiScript::new()
            .unwrap()
            .eval::<bool>(&hover_chunk(None))
            .unwrap());
    }
}
