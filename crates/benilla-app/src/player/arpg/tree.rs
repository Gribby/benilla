//! Fork-only, not 1.12.1: the ARPG skill tree's client half (cmangos `Arpg/ArpgTree.h`, the design
//! in `docs/ARPG-SKILL-TREES.md`). The server owns the tree: it sends the layout, points and ranks
//! (`SMSG_ARPG_TREE`) after the hello and every spend or respec, and this hands them to the ARPG
//! HUD addon's tree window (`ArpgTree_Update` in `arpg_hud.lua`), which the talent key opens in
//! place of the talent frame.
//!
//! The window answers through one session-only setting, `arpgTreeAction`: a number, `kind *
//! 100000 + node * 100 + nonce` with the nonce under 100 so a repeat still moves it (kind 1 spend,
//! 2 respec, 3 query), which [`on_tree_action`] turns into `ClientCommand::ArpgTreeSpend`, `ArpgTreeRespec` or
//! `ArpgTreeQuery`.

use benilla_protocol::messages::ArpgTree;
use benilla_protocol::{SessionEvent, SessionEventKind};
use benilla_ui::script::UiScript;

use crate::net::NetHandlerApp;

use super::*;

/// The window's action setting (session-owned, never saved).
pub(crate) const CVAR_TREE_ACTION: &str = "arpgTreeAction";

/// The tree as the server last sent it, and what the window has been given.
#[derive(Resource, Default)]
pub(crate) struct ArpgTreeState {
    tree: Option<ArpgTree>,
    /// Bumped by every tree the server sends.
    generation: u64,
    /// The VM and generation last handed to the window.
    pushed: Option<(u64, u64)>,
}

/// Registered whatever the view, so every session event kind has its owner.
pub(super) fn register_net(app: &mut App) {
    app.init_resource::<ArpgTreeState>()
        .net_handler(SessionEventKind::ArpgTree, on_tree);
}

fn on_tree(In(ev): In<SessionEvent>, mut state: ResMut<ArpgTreeState>) {
    if let SessionEvent::ArpgTree { tree } = ev {
        info!(
            "arpg: skill tree, {} of {} points spent, {} node(s)",
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
    app.add_systems(Update, push_tree)
        .add_observer(on_tree_action);
}

/// `s` as a Lua string literal.
fn lua_str(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 2);
    out.push('"');
    for c in s.chars() {
        match c {
            '\\' => out.push_str("\\\\"),
            '"' => out.push_str("\\\""),
            '\n' => out.push_str("\\n"),
            '\r' => {}
            c if c.is_ascii() && !c.is_ascii_control() => out.push(c),
            // Lua 5.0 has no unicode escapes; the tree's text is ASCII anyway.
            _ => out.push('?'),
        }
    }
    out.push('"');
    out
}

/// The Lua chunk that hands `tree` to the window, `icon` naming a spell's icon path; it returns
/// whether the window took it (the addon may not have loaded into this VM yet).
fn tree_chunk(tree: &ArpgTree, icon: impl Fn(u32) -> Option<String>) -> String {
    let branches: Vec<String> = tree.branches.iter().map(|b| lua_str(b)).collect();
    let nodes: Vec<String> = tree
        .nodes
        .iter()
        .map(|n| {
            format!(
                "{{id={},branch={},tier={},column={},kind={},max={},rank={},icon={},name={},text={}}}",
                n.id,
                n.branch,
                n.tier,
                n.column,
                n.kind,
                n.max_rank,
                n.rank,
                lua_str(&icon(n.icon_spell).unwrap_or_default()),
                lua_str(&n.name),
                lua_str(&n.text),
            )
        })
        .collect();
    format!(
        "if not ArpgTree_Update then return false end\n\
         ArpgTree_Update({{total={},spent={},branches={{{}}},nodes={{{}}}}})\n\
         return true",
        tree.points_total,
        tree.points_spent,
        branches.join(","),
        nodes.join(",\n"),
    )
}

/// Hand the window the newest tree, once per tree and VM (a reloaded UI is a new VM).
fn push_tree(
    script: Option<NonSendMut<UiScript>>,
    mut state: ResMut<ArpgTreeState>,
    spells: Option<Res<crate::ui_action::Spells>>,
) {
    let (Some(script), Some(tree)) = (script, state.tree.as_ref()) else {
        return;
    };
    let mark = (script.session(), state.generation);
    if state.pushed == Some(mark) {
        return;
    }
    let chunk = tree_chunk(tree, |id| {
        spells
            .as_deref()
            .and_then(|s| s.catalog.get(id))
            .and_then(|d| d.icon.clone())
    });
    match script.eval::<bool>(&chunk) {
        Ok(true) => state.pushed = Some(mark),
        Ok(false) => {}
        Err(e) => {
            warn!("arpg: the tree window refused the tree: {e}");
            state.pushed = Some(mark);
        }
    }
}

/// What the window asked for.
#[derive(Debug, PartialEq, Eq)]
enum TreeAction {
    Spend(u16),
    Respec,
    Query,
}

fn parse_action(value: &str) -> Option<TreeAction> {
    let code: u32 = value.trim().parse::<f64>().ok().filter(|v| *v >= 0.0)? as u32;
    match code / 100_000 {
        1 => u16::try_from(code % 100_000 / 100)
            .ok()
            .map(TreeAction::Spend),
        2 => Some(TreeAction::Respec),
        3 => Some(TreeAction::Query),
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
        None => return,
    };
    let _ = net.0.send(command);
}

#[cfg(test)]
mod tests {
    use super::*;
    use benilla_protocol::messages::ArpgTreeNode;

    #[test]
    fn actions_parse_and_junk_does_not() {
        assert_eq!(parse_action("100512"), Some(TreeAction::Spend(5)));
        assert_eq!(parse_action("104907"), Some(TreeAction::Spend(49)));
        assert_eq!(parse_action("200003"), Some(TreeAction::Respec));
        assert_eq!(parse_action("300001"), Some(TreeAction::Query));
        assert_eq!(parse_action("0"), None);
        assert_eq!(parse_action("spend"), None);
    }

    #[test]
    fn the_chunk_hands_the_window_the_tree_and_escapes_its_text() {
        let tree = ArpgTree {
            points_total: 59,
            points_spent: 1,
            branches: vec!["Crusader".into()],
            nodes: vec![ArpgTreeNode {
                id: 9,
                branch: 0,
                tier: 4,
                column: 1,
                kind: 3,
                max_rank: 1,
                rank: 0,
                icon_spell: 20271,
                name: "Avenger".into(),
                text: "Keeps the \"Seal\"\\on.".into(),
            }],
        };
        let chunk = tree_chunk(&tree, |_| Some("Interface\\Icons\\Spell_Holy".into()));
        let script = UiScript::new().unwrap();
        script
            .run("function ArpgTree_Update(t) GOT = t end")
            .unwrap();
        assert!(script.eval::<bool>(&chunk).unwrap());
        assert_eq!(script.eval::<u32>("return GOT.total").unwrap(), 59);
        assert_eq!(
            script.eval::<String>("return GOT.nodes[1].text").unwrap(),
            "Keeps the \"Seal\"\\on."
        );
        assert_eq!(
            script.eval::<String>("return GOT.nodes[1].icon").unwrap(),
            "Interface\\Icons\\Spell_Holy"
        );
        assert_eq!(
            script.eval::<String>("return GOT.branches[1]").unwrap(),
            "Crusader"
        );
        // No window yet: not taken.
        let bare = UiScript::new().unwrap();
        assert!(!bare.eval::<bool>(&chunk).unwrap());
    }
}
