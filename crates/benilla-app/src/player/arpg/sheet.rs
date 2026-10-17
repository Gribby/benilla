//! Fork-only, not 1.12.1: the ARPG server's procs and character sheet (cmangos
//! `Arpg/ArpgCombat.h` and `Arpg/ArpgCharacter.h`).
//!
//! A proc (`SMSG_ARPG_PROC`) is one line the server words: a skill node, rune, keystone, unique
//! or globe that fired on its own, its name already in the proc gold. The line goes to the combat
//! log as a `COMBAT_MISC_INFO` line, and a heal or mana proc floats its amount over the player,
//! green or blue (the stock client floats no heals, so a leech would otherwise never show).
//!
//! The sheet (`SMSG_ARPG_SHEET`) says what each attribute does and what adds to each skill, at the
//! character's own numbers; it is handed to the HUD addon as `ArpgSheet_Update`, which puts the
//! attribute lines on the paper doll's stat tooltips, the skill lines in the Skills tab and the
//! summary on the passive web's Totals button (`crate::ui_arpg::sheet`).

use benilla_protocol::messages::arpg::{PROC_HEAL, PROC_MANA};
use benilla_protocol::messages::{ArpgProc, ArpgSheet};
use benilla_protocol::{SessionEvent, SessionEventKind};

use crate::net::{GuidIndex, NetHandlerApp, SelfGuid};
use crate::ui_chat::{ChatEvent, ChatEventKind, ChatLog};

use super::*;

/// The floating colours of a proc's amount over the player: a heal green, mana blue.
const HEAL_ARGB: u32 = 0xFF20_FF20;
const MANA_ARGB: u32 = 0xFF50_90FF;

/// The procs waiting for the combat log, and the sheet the server last sent.
#[derive(Resource, Default)]
pub(crate) struct ArpgSheetState {
    procs: Vec<ArpgProc>,
    sheet: Option<ArpgSheet>,
    /// Bumped by every sheet the server sends.
    generation: u64,
    /// The VM and generation last handed to the addon.
    pushed: Option<(u64, u64)>,
}

impl ArpgSheetState {
    /// The sheet the server last sent.
    pub(crate) fn sheet(&self) -> Option<&ArpgSheet> {
        self.sheet.as_ref()
    }

    /// Bumped by every sheet the server sends.
    pub(crate) fn generation(&self) -> u64 {
        self.generation
    }

    /// The VM and generation last handed to the addon.
    pub(crate) fn pushed(&self) -> Option<(u64, u64)> {
        self.pushed
    }

    /// Note `mark`, a VM and generation, as handed to the addon.
    pub(crate) fn set_pushed(&mut self, mark: (u64, u64)) {
        self.pushed = Some(mark);
    }
}

/// Registered whatever the view, so every session event kind has its owner.
pub(super) fn register_net(app: &mut App) {
    app.init_resource::<ArpgSheetState>()
        .net_handler(SessionEventKind::ArpgProc, on_proc)
        .net_handler(SessionEventKind::ArpgSheet, on_sheet);
}

fn on_proc(In(ev): In<SessionEvent>, mut state: ResMut<ArpgSheetState>) {
    if let SessionEvent::ArpgProc { proc_ } = ev {
        // A flood (a stalled frame) keeps the newest.
        if state.procs.len() >= 64 {
            state.procs.remove(0);
        }
        state.procs.push(proc_);
    }
}

fn on_sheet(In(ev): In<SessionEvent>, mut state: ResMut<ArpgSheetState>) {
    if let SessionEvent::ArpgSheet { sheet } = ev {
        state.sheet = Some(sheet);
        state.generation += 1;
    }
}

pub(super) fn plugin(app: &mut App) {
    app.add_systems(Update, (log_procs, crate::ui_arpg::sheet::push_sheet));
}

/// The floating text of a proc's amount, if it floats: a heal "+42", mana "+30 mana".
fn proc_float(p: &ArpgProc) -> Option<(String, u32)> {
    if p.amount == 0 {
        return None;
    }
    match p.kind {
        PROC_HEAL => Some((format!("+{}", p.amount), HEAL_ARGB)),
        PROC_MANA => Some((format!("+{} mana", p.amount), MANA_ARGB)),
        _ => None,
    }
}

/// Each proc to the combat log, and its amount over the player.
fn log_procs(
    mut state: ResMut<ArpgSheetState>,
    log: Option<ResMut<ChatLog>>,
    self_guid: Res<SelfGuid>,
    index: Res<GuidIndex>,
    mut text: MessageWriter<crate::combat_text::CombatTextSpawn>,
    mut shown: MessageWriter<super::feel::ArpgProcShown>,
) {
    if state.procs.is_empty() {
        return;
    }
    let procs = std::mem::take(&mut state.procs);
    let me = self_guid.0.and_then(|g| index.0.get(&g).copied());
    let mut log = log;
    for p in procs {
        if let Some(log) = log.as_deref_mut() {
            log.push_event(ChatEvent::text_only(
                ChatEventKind::CombatMiscInfo,
                p.line.clone(),
            ));
        }
        shown.write(super::feel::ArpgProcShown(p.clone()));
        if let (Some(anchor), Some((body, color))) = (me, proc_float(&p)) {
            text.write(crate::combat_text::CombatTextSpawn {
                anchor,
                text: body,
                category: 0,
                color: Some(color),
                arpg: None,
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn heals_and_mana_float_and_the_rest_do_not() {
        let heal = ArpgProc {
            kind: PROC_HEAL,
            amount: 42,
            line: String::new(),
            ..Default::default()
        };
        assert_eq!(proc_float(&heal), Some(("+42".into(), HEAL_ARGB)));
        let mana = ArpgProc {
            kind: PROC_MANA,
            amount: 30,
            line: String::new(),
            ..Default::default()
        };
        assert_eq!(proc_float(&mana), Some(("+30 mana".into(), MANA_ARGB)));
        let hit = ArpgProc {
            kind: benilla_protocol::messages::arpg::PROC_DAMAGE,
            amount: 0,
            line: String::new(),
            ..Default::default()
        };
        assert_eq!(proc_float(&hit), None);
    }
}
