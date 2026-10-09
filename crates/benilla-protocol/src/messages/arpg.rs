//! Fork-only, not 1.12.1: the ARPG server's messages to the benilla ARPG client (cmangos
//! `Arpg/ArpgLoot.h`). A stock server never sends them; the opcodes sit past 1.12.1's last.

use std::io;

use crate::wire::{capacity_hint, read_cstring, read_u32_le, read_u64_le, read_u8};

/// What a corpse holds for this player, shown on the ground around it: `u64` corpse, `u32` gold,
/// `u8` count, then per item `u8` loot slot, `u32` item id, `u32` display id, `u8` quality,
/// `u8` count. An empty list means nothing is left there for this player.
pub const SMSG_ARPG_LOOT: u16 = 0x033D;

/// The loot slot that names the corpse's gold in a pick-up.
pub const LOOT_SLOT_GOLD: u8 = 0xFF;

/// One item lying on the ground by a corpse.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ArpgLootItem {
    /// The corpse loot's slot, which a pick-up names.
    pub slot: u8,
    pub item_id: u32,
    /// `item_template.display_id`.
    pub display_id: u32,
    /// 0 poor (grey) to 5 legendary.
    pub quality: u8,
    pub count: u8,
}

/// The uniques' tooltip lines (cmangos `Arpg/ArpgUniques.h`), sent after every hello: `u8` count,
/// then per row `u32` item id and a C string, the line the item's tooltip shows.
pub const SMSG_ARPG_ITEM_MECHANICS: u16 = 0x033E;

/// `SMSG_ARPG_ITEM_MECHANICS`: `(item id, tooltip line)` per unique.
pub fn read_arpg_item_mechanics(r: &mut impl io::Read) -> io::Result<Vec<(u32, String)>> {
    let n = read_u8(r)?;
    let mut rows = Vec::with_capacity(capacity_hint(n, 255));
    for _ in 0..n {
        let item = read_u32_le(r)?;
        rows.push((item, read_cstring(r)?));
    }
    Ok(rows)
}

/// `SMSG_ARPG_LOOT`: the corpse, its gold, and the items.
pub fn read_arpg_loot(r: &mut impl io::Read) -> io::Result<(u64, u32, Vec<ArpgLootItem>)> {
    let corpse = read_u64_le(r)?;
    let gold = read_u32_le(r)?;
    let n = read_u8(r)?;
    let mut items = Vec::with_capacity(capacity_hint(n, 255));
    for _ in 0..n {
        items.push(ArpgLootItem {
            slot: read_u8(r)?,
            item_id: read_u32_le(r)?,
            display_id: read_u32_le(r)?,
            quality: read_u8(r)?,
            count: read_u8(r)?,
        });
    }
    Ok((corpse, gold, items))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_uniques_read_as_item_and_line_pairs() {
        let mut body = vec![2];
        body.extend_from_slice(&5201u32.to_le_bytes());
        body.extend_from_slice(b"Fireball launches 1 extra fireball.\0");
        body.extend_from_slice(&18842u32.to_le_bytes());
        body.extend_from_slice(b"Fireball bursts.\0");
        let rows = read_arpg_item_mechanics(&mut body.as_slice()).unwrap();
        assert_eq!(
            rows,
            vec![
                (5201, "Fireball launches 1 extra fireball.".to_string()),
                (18842, "Fireball bursts.".to_string())
            ]
        );
    }

    #[test]
    fn a_loot_list_reads_the_corpse_gold_and_items() {
        let mut body = Vec::new();
        body.extend_from_slice(&0xF130_0000_0000_0042u64.to_le_bytes());
        body.extend_from_slice(&125u32.to_le_bytes());
        body.push(2);
        for (slot, id, display, quality, count) in
            [(0u8, 2589u32, 7418u32, 1u8, 2u8), (3, 1411, 8473, 2, 1)]
        {
            body.push(slot);
            body.extend_from_slice(&id.to_le_bytes());
            body.extend_from_slice(&display.to_le_bytes());
            body.push(quality);
            body.push(count);
        }
        let (corpse, gold, items) = read_arpg_loot(&mut body.as_slice()).unwrap();
        assert_eq!(corpse, 0xF130_0000_0000_0042);
        assert_eq!(gold, 125);
        assert_eq!(items.len(), 2);
        assert_eq!(
            items[1],
            ArpgLootItem {
                slot: 3,
                item_id: 1411,
                display_id: 8473,
                quality: 2,
                count: 1
            }
        );
    }
}
