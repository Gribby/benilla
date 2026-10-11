//! Fork-only, not 1.12.1: the ARPG server's uniques, named items whose wearer's spells work
//! differently (cmangos `Arpg/ArpgUniques.h`, the design in `docs/ARPG-UNIQUES.md`). The server
//! sends each one's tooltip line after the hello (`SMSG_ARPG_ITEM_MECHANICS`); the item tooltip
//! shows it in the unique colour under the item's own lines (`crate::ui_items::feed`), and its
//! ground loot wears that colour as a label border and a beam core (`super::loot`).
//!
//! The colour is the client's seventh quality colour, "Artifact" pale gold, which no 1.12 item
//! wears: a unique keeps its blue or purple name, so its tier still reads at a glance.

use std::collections::HashMap;

use benilla_protocol::{SessionEvent, SessionEventKind};

use crate::net::NetHandlerApp;

use super::*;

/// Item id → the unique's tooltip line, as the server last sent them; empty on a stock server.
#[derive(Resource, Default, Debug)]
pub(crate) struct ArpgUniques(pub(crate) HashMap<u32, String>);

impl ArpgUniques {
    /// The tooltip line of `item`, if it is a unique.
    pub(crate) fn line(&self, item: u32) -> Option<&str> {
        self.0.get(&item).map(String::as_str)
    }
}

/// Registered whatever the view, so every session event kind has its owner and the tooltip feed
/// always finds the table: a stock server never sends the message.
pub(super) fn register_net(app: &mut App) {
    app.init_resource::<ArpgUniques>()
        .net_handler(SessionEventKind::ArpgItemMechanics, on_item_mechanics);
}

fn on_item_mechanics(In(ev): In<SessionEvent>, mut uniques: ResMut<ArpgUniques>) {
    if let SessionEvent::ArpgItemMechanics { rows } = ev {
        let fresh = merged(rows);
        // Every hello resends the table: only a change touches the resource, which re-feeds the
        // tooltips.
        if uniques.0 != fresh {
            info!("arpg: {} unique item(s) from the server", fresh.len());
            uniques.0 = fresh;
        }
    }
}

/// The table by item: an item with a mechanic and a proc (`Arpg/ArpgProcs.h`) has two rows, its
/// lines joined in the order sent.
fn merged(rows: Vec<(u32, String)>) -> HashMap<u32, String> {
    let mut out: HashMap<u32, String> = HashMap::new();
    for (item, line) in rows {
        out.entry(item)
            .and_modify(|text| {
                text.push('\n');
                text.push_str(&line);
            })
            .or_insert(line);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn an_item_with_two_rows_keeps_both_lines() {
        let table = merged(vec![
            (873, "Frostbolt shatters.".into()),
            (2243, "Chains.".into()),
            (873, "A critical Frostbolt readies Fireball.".into()),
        ]);
        assert_eq!(
            table[&873],
            "Frostbolt shatters.\nA critical Frostbolt readies Fireball."
        );
        assert_eq!(table[&2243], "Chains.");
    }

    #[test]
    fn the_table_answers_by_item_id() {
        let mut uniques = ArpgUniques::default();
        uniques
            .0
            .insert(5201, "Fireball launches 1 extra fireball.".into());
        assert_eq!(
            uniques.line(5201),
            Some("Fireball launches 1 extra fireball.")
        );
        assert_eq!(uniques.line(25), None);
    }
}
