//! Fork-only, not 1.12.1: the ARPG passive web's client half (cmangos `Arpg/ArpgTree.h`, the
//! design in `docs/ARPG-CHARACTER.md`). The server owns the web: it sends the layout, points and
//! taken nodes (`SMSG_ARPG_TREE`) after the hello and every change, and this hands them to the ARPG
//! HUD addon's web window (`ArpgTree_Update` in `arpg_hud.lua`), which the talent key opens in
//! place of the talent frame. The window's feed is `crate::ui_arpg::tree`.
//!
//! The same window's Skills tab shows the specialised skills and their trees
//! (`SMSG_ARPG_SKILLS`, cmangos `Arpg/ArpgSkills.h`), handed over as `ArpgSkills_Update`.
//!
//! The window answers through one session-only setting, `arpgTreeAction`: a number, `kind *
//! 100000 + node * 100 + nonce` with the nonce under 100 so a repeat still moves it (kind 1 take,
//! 2 respec, 3 query, 4 give back; 5 slot, its node `slot * 100 + skill`; 6 take a skill rank,
//! 7 give one back, 8 respec a skill, its node the skill; 9 the dev tools' test pack, its node
//! `tier * 100 + size`; 10 unseal a capstone with Codex fragments; 11 socket a rune, its node
//! `skill * 100 + rune`, rune 0 emptying it; 12 the town portal, from the HUD's button and its
//! key; 13 abandon the bounty in slot `node`), which [`on_tree_action`] turns into a `ClientCommand`.

use benilla_protocol::messages::{ArpgSkills, ArpgTree};
use benilla_protocol::{SessionEvent, SessionEventKind};

use crate::net::NetHandlerApp;

use super::*;

/// The window's action setting (session-owned, never saved).
pub(crate) const CVAR_TREE_ACTION: &str = "arpgTreeAction";

/// The tree as the server last sent it, and what the window has been given.
#[derive(Resource, Default)]
pub(crate) struct ArpgTreeState {
    pub(crate) tree: Option<ArpgTree>,
    /// Bumped by every tree the server sends.
    pub(crate) generation: u64,
    /// The VM and generation last handed to the window.
    pub(crate) pushed: Option<(u64, u64)>,
    pub(crate) skills: Option<ArpgSkills>,
    pub(crate) skills_generation: u64,
    pub(crate) skills_pushed: Option<(u64, u64)>,
}

/// Registered whatever the view, so every session event kind has its owner.
pub(super) fn register_net(app: &mut App) {
    app.init_resource::<ArpgTreeState>()
        .net_handler(SessionEventKind::ArpgTree, on_tree)
        .net_handler(SessionEventKind::ArpgSkills, on_skills);
}

fn on_skills(In(ev): In<SessionEvent>, mut state: ResMut<ArpgTreeState>) {
    if let SessionEvent::ArpgSkills { skills } = ev {
        info!(
            "arpg: skills, {} of {} points spent, {} skill(s)",
            skills.points_spent,
            skills.points_total,
            skills.skills.len()
        );
        state.skills = Some(skills);
        state.skills_generation += 1;
    }
}

fn on_tree(In(ev): In<SessionEvent>, mut state: ResMut<ArpgTreeState>) {
    if let SessionEvent::ArpgTree { tree } = ev {
        info!(
            "arpg: passive web, {} of {} points spent, {} node(s)",
            tree.points_spent,
            tree.points_total,
            tree.nodes.len()
        );
        state.tree = Some(tree);
        state.generation += 1;
    }
}

/// The view's half: hand the window each tree, and send what it asks.
pub(super) fn plugin(app: &mut App) {
    app.add_systems(
        Update,
        (forget_on_entry, crate::ui_arpg::tree::push_tree).chain(),
    )
    .add_observer(on_tree_action);
}

/// A world entry (a new character, or the same one again) empties both tabs until the server
/// sends this character's: another character's web never shows.
fn forget_on_entry(
    mut entered: MessageReader<crate::net::EnteredWorldMessage>,
    mut state: ResMut<ArpgTreeState>,
) {
    if entered.read().count() == 0 {
        return;
    }
    state.tree = Some(ArpgTree::default());
    state.generation += 1;
    state.skills = Some(ArpgSkills::default());
    state.skills_generation += 1;
}

/// What the window asked for.
#[derive(Debug, PartialEq, Eq)]
enum TreeAction {
    Spend(u16),
    Respec,
    Query,
    Refund(u16),
    Slot {
        slot: u8,
        skill: u8,
    },
    SkillNode {
        node: u16,
        refund: bool,
    },
    SkillRespec(u8),
    DevPack {
        size: u8,
        tier: u8,
    },
    /// Unseal a capstone with Codex fragments.
    Unseal(u16),
    /// Socket a rune (0 empties the socket) in a skill.
    Socket {
        skill: u8,
        rune: u8,
    },
    /// Open a town portal, or go back through it.
    TownPortal,
    /// Abandon the bounty in a slot for a new one.
    AbandonBounty(u8),
}

fn parse_action(value: &str) -> Option<TreeAction> {
    let code: u32 = value.trim().parse::<f64>().ok().filter(|v| *v >= 0.0)? as u32;
    match code / 100_000 {
        1 => u16::try_from(code % 100_000 / 100)
            .ok()
            .map(TreeAction::Spend),
        2 => Some(TreeAction::Respec),
        3 => Some(TreeAction::Query),
        4 => u16::try_from(code % 100_000 / 100)
            .ok()
            .map(TreeAction::Refund),
        5 => {
            let field = code % 100_000 / 100;
            Some(TreeAction::Slot {
                slot: u8::try_from(field / 100).ok()?,
                skill: u8::try_from(field % 100).ok()?,
            })
        }
        6 | 7 => u16::try_from(code % 100_000 / 100)
            .ok()
            .map(|node| TreeAction::SkillNode {
                node,
                refund: code / 100_000 == 7,
            }),
        8 => u8::try_from(code % 100_000 / 100)
            .ok()
            .map(TreeAction::SkillRespec),
        9 => {
            let field = code % 100_000 / 100;
            Some(TreeAction::DevPack {
                size: u8::try_from(field % 100).ok()?,
                tier: u8::try_from(field / 100).ok()?,
            })
        }
        10 => u16::try_from(code % 100_000 / 100)
            .ok()
            .map(TreeAction::Unseal),
        11 => {
            let field = code % 100_000 / 100;
            Some(TreeAction::Socket {
                skill: u8::try_from(field / 100).ok()?,
                rune: u8::try_from(field % 100).ok()?,
            })
        }
        12 => Some(TreeAction::TownPortal),
        13 => u8::try_from(code % 100_000 / 100)
            .ok()
            .map(TreeAction::AbandonBounty),
        _ => None,
    }
}

/// The window's action setting moved: send the request.
fn on_tree_action(ev: On<crate::cvars::CvarChanged>, net: Res<NetCommands>) {
    if !ev.is(CVAR_TREE_ACTION) {
        return;
    }
    let command = match parse_action(&ev.new) {
        Some(TreeAction::Spend(node)) => ClientCommand::ArpgTreeSpend { node },
        Some(TreeAction::Respec) => ClientCommand::ArpgTreeRespec,
        Some(TreeAction::Query) => ClientCommand::ArpgTreeQuery,
        Some(TreeAction::Refund(node)) => ClientCommand::ArpgTreeRefund { node },
        Some(TreeAction::Slot { slot, skill }) => ClientCommand::ArpgSkillSlot { slot, skill },
        Some(TreeAction::SkillNode { node, refund }) => {
            ClientCommand::ArpgSkillNode { node, refund }
        }
        Some(TreeAction::SkillRespec(skill)) => ClientCommand::ArpgSkillRespec { skill },
        Some(TreeAction::DevPack { size, tier }) => ClientCommand::ArpgDevPack { size, tier },
        Some(TreeAction::Unseal(node)) => ClientCommand::ArpgUnseal { node },
        Some(TreeAction::Socket { skill, rune }) => ClientCommand::ArpgSocket { skill, rune },
        Some(TreeAction::TownPortal) => ClientCommand::ArpgTownPortal,
        Some(TreeAction::AbandonBounty(slot)) => ClientCommand::ArpgBountyAbandon { slot },
        None => return,
    };
    let _ = net.0.send(command);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn actions_parse_and_junk_does_not() {
        assert_eq!(parse_action("100512"), Some(TreeAction::Spend(5)));
        assert_eq!(parse_action("104907"), Some(TreeAction::Spend(49)));
        assert_eq!(parse_action("200003"), Some(TreeAction::Respec));
        assert_eq!(parse_action("300001"), Some(TreeAction::Query));
        assert_eq!(parse_action("408811"), Some(TreeAction::Refund(88)));
        assert_eq!(
            parse_action("520305"),
            Some(TreeAction::Slot { slot: 2, skill: 3 })
        );
        assert_eq!(
            parse_action("630101"),
            Some(TreeAction::SkillNode {
                node: 301,
                refund: false
            })
        );
        assert_eq!(
            parse_action("730102"),
            Some(TreeAction::SkillNode {
                node: 301,
                refund: true
            })
        );
        assert_eq!(parse_action("800403"), Some(TreeAction::SkillRespec(4)));
        assert_eq!(
            parse_action("920507"),
            Some(TreeAction::DevPack { size: 5, tier: 2 })
        );
        assert_eq!(parse_action("1030301"), Some(TreeAction::Unseal(303)));
        assert_eq!(
            parse_action("1130102"),
            Some(TreeAction::Socket { skill: 3, rune: 1 })
        );
        assert_eq!(parse_action("1200007"), Some(TreeAction::TownPortal));
        assert_eq!(parse_action("1300204"), Some(TreeAction::AbandonBounty(2)));
        assert_eq!(parse_action("0"), None);
        assert_eq!(parse_action("spend"), None);
    }
}
