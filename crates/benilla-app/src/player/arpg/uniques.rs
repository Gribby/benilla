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

/// The unique colour, sRGB: the client's Artifact quality colour (quality 6, `e6cc80`).
pub(crate) const UNIQUE_RGB: [f32; 3] = [0.902, 0.8, 0.502];

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
        let fresh: HashMap<u32, String> = rows.into_iter().collect();
        // Every hello resends the table: only a change touches the resource, which re-feeds the
        // tooltips.
        if uniques.0 != fresh {
            info!("arpg: {} unique item(s) from the server", fresh.len());
            uniques.0 = fresh;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
