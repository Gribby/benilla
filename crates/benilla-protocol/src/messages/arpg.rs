//! Fork-only, not 1.12.1: the ARPG server's messages to the benilla ARPG client (cmangos
//! `Arpg/ArpgLoot.h`). A stock server never sends them; the opcodes sit past 1.12.1's last.

use std::io;

use crate::wire::{capacity_hint, read_cstring, read_u16_le, read_u32_le, read_u64_le, read_u8};

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

/// The player's ARPG skill tree (cmangos `Arpg/ArpgTree.h`), sent after the hello and every spend
/// or respec.
pub const SMSG_ARPG_TREE: u16 = 0x033F;

/// One node of the skill tree, as the server lays it out.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ArpgTreeNode {
    pub id: u16,
    pub branch: u8,
    /// 1 to 3, 4 the branch's keystone.
    pub tier: u8,
    /// 0 to 2.
    pub column: u8,
    /// 0 passive, 1 skill, 2 modifier, 3 keystone.
    pub kind: u8,
    pub max_rank: u8,
    pub rank: u8,
    /// The spell whose icon the node shows.
    pub icon_spell: u32,
    pub name: String,
    pub text: String,
}

/// The whole tree: points, branch names and nodes.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ArpgTree {
    pub points_total: u16,
    pub points_spent: u16,
    pub branches: Vec<String>,
    pub nodes: Vec<ArpgTreeNode>,
}

/// `SMSG_ARPG_TREE`: `u8` version (1), `u16` points total, `u16` points spent, `u8` branch count
/// and a C string each, `u8` node count, then per node `u16` id, `u8` branch, tier, column, kind,
/// max rank and rank, `u32` icon spell, and C strings name and text.
pub fn read_arpg_tree(r: &mut impl io::Read) -> io::Result<ArpgTree> {
    let version = read_u8(r)?;
    if version != 1 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!("ARPG tree version {version}, this client reads 1"),
        ));
    }
    let points_total = read_u16_le(r)?;
    let points_spent = read_u16_le(r)?;
    let nb = read_u8(r)?;
    let mut branches = Vec::with_capacity(capacity_hint(nb, 16));
    for _ in 0..nb {
        branches.push(read_cstring(r)?);
    }
    let nn = read_u8(r)?;
    let mut nodes = Vec::with_capacity(capacity_hint(nn, 255));
    for _ in 0..nn {
        nodes.push(ArpgTreeNode {
            id: read_u16_le(r)?,
            branch: read_u8(r)?,
            tier: read_u8(r)?,
            column: read_u8(r)?,
            kind: read_u8(r)?,
            max_rank: read_u8(r)?,
            rank: read_u8(r)?,
            icon_spell: read_u32_le(r)?,
            name: read_cstring(r)?,
            text: read_cstring(r)?,
        });
    }
    Ok(ArpgTree {
        points_total,
        points_spent,
        branches,
        nodes,
    })
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
    fn a_tree_reads_its_points_branches_and_nodes() {
        let mut body = vec![1];
        body.extend_from_slice(&59u16.to_le_bytes());
        body.extend_from_slice(&3u16.to_le_bytes());
        body.push(1);
        body.extend_from_slice(b"Crusader\0");
        body.push(1);
        body.extend_from_slice(&5u16.to_le_bytes());
        body.extend_from_slice(&[0, 2, 1, 2, 2, 1]);
        body.extend_from_slice(&20271u32.to_le_bytes());
        body.extend_from_slice(b"Chain of Judgement\0Judgement chains.\0");
        let tree = read_arpg_tree(&mut body.as_slice()).unwrap();
        assert_eq!((tree.points_total, tree.points_spent), (59, 3));
        assert_eq!(tree.branches, vec!["Crusader".to_string()]);
        assert_eq!(
            tree.nodes[0],
            ArpgTreeNode {
                id: 5,
                branch: 0,
                tier: 2,
                column: 1,
                kind: 2,
                max_rank: 2,
                rank: 1,
                icon_spell: 20271,
                name: "Chain of Judgement".into(),
                text: "Judgement chains.".into(),
            }
        );
        assert!(read_arpg_tree(&mut [2u8].as_slice()).is_err());
    }

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
