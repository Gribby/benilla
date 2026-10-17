//! Fork-only, not 1.12.1: the feed half of the ARPG view's feel (`crate::player::arpg::feel`):
//! the kill streak on the HUD (`ArpgHud_Streak` in `arpg_hud.lua`).

use bevy::prelude::*;

use benilla_ui::script::UiScript;

use crate::player::arpg::feel::Streak;

/// Tell the HUD of the streak.
pub(crate) fn push_streak(script: Option<NonSendMut<UiScript>>, mut streak: ResMut<Streak>) {
    let Some(script) = script else {
        return;
    };
    if !streak.dirty() {
        return;
    }
    let (count, word) = streak.take_news();
    let word = word.map_or("nil".to_string(), |w| format!("{w:?}"));
    let chunk = format!("if ArpgHud_Streak then ArpgHud_Streak({count}, {word}) end\n");
    if let Err(e) = script.eval::<()>(&chunk) {
        warn!("arpg: the HUD refused the kill streak: {e}");
    }
}
