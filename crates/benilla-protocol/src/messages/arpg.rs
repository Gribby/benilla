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

/// The player's ARPG passive web (cmangos `Arpg/ArpgTree.h`), sent after the hello and every
/// take, give-back or respec.
pub const SMSG_ARPG_TREE: u16 = 0x033F;

/// One node of the passive web, where the server lays it out.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ArpgTreeNode {
    pub id: u16,
    /// 0 the start, 1 small, 2 notable, 3 keystone.
    pub kind: u8,
    /// Index into [`ArpgTree::regions`].
    pub region: u8,
    /// Web units from the start, y down.
    pub x: i16,
    pub y: i16,
    pub taken: bool,
    /// The spell whose icon the node shows; 0 for none.
    pub icon_spell: u32,
    pub name: String,
    pub text: String,
}

/// A region of the web: its name and where its label sits.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ArpgTreeRegion {
    pub name: String,
    pub x: i16,
    pub y: i16,
}

/// The whole web: points, regions, nodes and the links between them.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ArpgTree {
    pub points_total: u16,
    pub points_spent: u16,
    pub regions: Vec<ArpgTreeRegion>,
    pub nodes: Vec<ArpgTreeNode>,
    /// Node id pairs.
    pub links: Vec<(u16, u16)>,
}

fn read_i16_le(r: &mut impl io::Read) -> io::Result<i16> {
    read_u16_le(r).map(|v| v as i16)
}

/// `SMSG_ARPG_TREE`: `u8` version (2), `u16` points total, `u16` points spent; `u8` region count,
/// per region a C string name and `i16` x, y; `u16` node count, per node `u16` id, `u8` kind and
/// region, `i16` x, y, `u8` taken, `u32` icon spell, C strings name and text; `u16` link count,
/// per link two `u16` node ids.
pub fn read_arpg_tree(r: &mut impl io::Read) -> io::Result<ArpgTree> {
    let version = read_u8(r)?;
    if version != 2 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!("ARPG tree version {version}, this client reads 2"),
        ));
    }
    let points_total = read_u16_le(r)?;
    let points_spent = read_u16_le(r)?;
    let nr = read_u8(r)?;
    let mut regions = Vec::with_capacity(capacity_hint(nr, 16));
    for _ in 0..nr {
        regions.push(ArpgTreeRegion {
            name: read_cstring(r)?,
            x: read_i16_le(r)?,
            y: read_i16_le(r)?,
        });
    }
    let nn = read_u16_le(r)?;
    let mut nodes = Vec::with_capacity(capacity_hint(nn, 512));
    for _ in 0..nn {
        nodes.push(ArpgTreeNode {
            id: read_u16_le(r)?,
            kind: read_u8(r)?,
            region: read_u8(r)?,
            x: read_i16_le(r)?,
            y: read_i16_le(r)?,
            taken: read_u8(r)? != 0,
            icon_spell: read_u32_le(r)?,
            name: read_cstring(r)?,
            text: read_cstring(r)?,
        });
    }
    let nl = read_u16_le(r)?;
    let mut links = Vec::with_capacity(capacity_hint(nl, 1024));
    for _ in 0..nl {
        links.push((read_u16_le(r)?, read_u16_le(r)?));
    }
    Ok(ArpgTree {
        points_total,
        points_spent,
        regions,
        nodes,
        links,
    })
}

/// The player's specialised skills and their trees (cmangos `Arpg/ArpgSkills.h`), sent after the
/// hello and every change.
pub const SMSG_ARPG_SKILLS: u16 = 0x0340;

/// One specialisation slot: the level it opens at and the skill in it (0: empty).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ArpgSkillSlot {
    pub level: u8,
    pub skill: u8,
}

/// One node of a skill's tree.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ArpgSkillNode {
    /// The skill's id times 100 plus the node's number.
    pub id: u16,
    /// 1 modifier, 2 transformer, 3 synergy, 4 capstone.
    pub kind: u8,
    pub column: u8,
    pub row: u8,
    /// The node above it; 0 the skill itself.
    pub parent: u16,
    pub max_rank: u8,
    pub rank: u8,
    pub icon_spell: u32,
    pub name: String,
    pub text: String,
}

/// A skill that can be specialised, with its tree.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ArpgSkill {
    pub id: u8,
    pub icon_spell: u32,
    pub name: String,
    pub text: String,
    /// Points in its tree, and the most it takes.
    pub spent: u8,
    pub cap: u8,
    pub branches: Vec<String>,
    pub nodes: Vec<ArpgSkillNode>,
}

/// Every skill the class can specialise, the slots, and the points.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ArpgSkills {
    pub points_total: u16,
    pub points_spent: u16,
    /// The character's level, which the slots open by.
    pub level: u8,
    pub slots: Vec<ArpgSkillSlot>,
    pub skills: Vec<ArpgSkill>,
}

/// `SMSG_ARPG_SKILLS`: `u8` version (1), `u16` points total and spent, `u8` level; `u8` slot count,
/// per slot `u8` level and skill; `u8` skill count, per skill `u8` id, `u32` icon spell, C strings
/// name and text, `u8` spent and cap, `u8` branch count and a C string each, `u8` node count, per
/// node `u16` id, `u8` kind, column and row, `u16` parent, `u8` max rank and rank, `u32` icon
/// spell, C strings name and text.
pub fn read_arpg_skills(r: &mut impl io::Read) -> io::Result<ArpgSkills> {
    let version = read_u8(r)?;
    if version != 1 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!("ARPG skills version {version}, this client reads 1"),
        ));
    }
    let points_total = read_u16_le(r)?;
    let points_spent = read_u16_le(r)?;
    let level = read_u8(r)?;
    let ns = read_u8(r)?;
    let mut slots = Vec::with_capacity(capacity_hint(ns, 8));
    for _ in 0..ns {
        slots.push(ArpgSkillSlot {
            level: read_u8(r)?,
            skill: read_u8(r)?,
        });
    }
    let nk = read_u8(r)?;
    let mut skills = Vec::with_capacity(capacity_hint(nk, 16));
    for _ in 0..nk {
        let id = read_u8(r)?;
        let icon_spell = read_u32_le(r)?;
        let name = read_cstring(r)?;
        let text = read_cstring(r)?;
        let spent = read_u8(r)?;
        let cap = read_u8(r)?;
        let nb = read_u8(r)?;
        let mut branches = Vec::with_capacity(capacity_hint(nb, 8));
        for _ in 0..nb {
            branches.push(read_cstring(r)?);
        }
        let nn = read_u8(r)?;
        let mut nodes = Vec::with_capacity(capacity_hint(nn, 32));
        for _ in 0..nn {
            nodes.push(ArpgSkillNode {
                id: read_u16_le(r)?,
                kind: read_u8(r)?,
                column: read_u8(r)?,
                row: read_u8(r)?,
                parent: read_u16_le(r)?,
                max_rank: read_u8(r)?,
                rank: read_u8(r)?,
                icon_spell: read_u32_le(r)?,
                name: read_cstring(r)?,
                text: read_cstring(r)?,
            });
        }
        skills.push(ArpgSkill {
            id,
            icon_spell,
            name,
            text,
            spent,
            cap,
            branches,
            nodes,
        });
    }
    Ok(ArpgSkills {
        points_total,
        points_spent,
        level,
        slots,
        skills,
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
    fn a_web_reads_its_points_regions_nodes_and_links() {
        let mut body = vec![2];
        body.extend_from_slice(&59u16.to_le_bytes());
        body.extend_from_slice(&3u16.to_le_bytes());
        body.push(1);
        body.extend_from_slice(b"Crusader\0");
        body.extend_from_slice(&(-554i16).to_le_bytes());
        body.extend_from_slice(&(-320i16).to_le_bytes());
        body.extend_from_slice(&2u16.to_le_bytes());
        body.extend_from_slice(&1u16.to_le_bytes());
        body.extend_from_slice(&[0, 3]);
        body.extend_from_slice(&0i16.to_le_bytes());
        body.extend_from_slice(&0i16.to_le_bytes());
        body.push(1);
        body.extend_from_slice(&0u32.to_le_bytes());
        body.extend_from_slice(b"Paladin\0Start.\0");
        body.extend_from_slice(&2u16.to_le_bytes());
        body.extend_from_slice(&[1, 0]);
        body.extend_from_slice(&(-69i16).to_le_bytes());
        body.extend_from_slice(&(-40i16).to_le_bytes());
        body.push(0);
        body.extend_from_slice(&20111u32.to_le_bytes());
        body.extend_from_slice(b"Crusader\0+2 to all attributes\0");
        body.extend_from_slice(&1u16.to_le_bytes());
        body.extend_from_slice(&1u16.to_le_bytes());
        body.extend_from_slice(&2u16.to_le_bytes());
        let tree = read_arpg_tree(&mut body.as_slice()).unwrap();
        assert_eq!((tree.points_total, tree.points_spent), (59, 3));
        assert_eq!(
            tree.regions,
            vec![ArpgTreeRegion {
                name: "Crusader".into(),
                x: -554,
                y: -320
            }]
        );
        assert_eq!(
            tree.nodes[1],
            ArpgTreeNode {
                id: 2,
                kind: 1,
                region: 0,
                x: -69,
                y: -40,
                taken: false,
                icon_spell: 20111,
                name: "Crusader".into(),
                text: "+2 to all attributes".into(),
            }
        );
        assert!(tree.nodes[0].taken);
        assert_eq!(tree.links, vec![(1, 2)]);
        assert!(read_arpg_tree(&mut [1u8].as_slice()).is_err());
    }

    #[test]
    fn skills_read_their_slots_skills_and_nodes() {
        let mut body = vec![1];
        body.extend_from_slice(&40u16.to_le_bytes());
        body.extend_from_slice(&3u16.to_le_bytes());
        body.push(21);
        body.push(2);
        body.extend_from_slice(&[1, 3, 10, 0]);
        body.push(1);
        body.push(3);
        body.extend_from_slice(&20271u32.to_le_bytes());
        body.extend_from_slice(b"Judgement\0Unleash your Seal.\0");
        body.extend_from_slice(&[3, 20, 1]);
        body.extend_from_slice(b"Chain\0");
        body.push(1);
        body.extend_from_slice(&301u16.to_le_bytes());
        body.extend_from_slice(&[2, 0, 1]);
        body.extend_from_slice(&0u16.to_le_bytes());
        body.extend_from_slice(&[3, 2]);
        body.extend_from_slice(&20186u32.to_le_bytes());
        body.extend_from_slice(b"Chain of Judgement\0Chains.\0");
        let skills = read_arpg_skills(&mut body.as_slice()).unwrap();
        assert_eq!(
            (skills.points_total, skills.points_spent, skills.level),
            (40, 3, 21)
        );
        assert_eq!(
            skills.slots,
            vec![
                ArpgSkillSlot { level: 1, skill: 3 },
                ArpgSkillSlot {
                    level: 10,
                    skill: 0
                }
            ]
        );
        let judgement = &skills.skills[0];
        assert_eq!((judgement.id, judgement.spent, judgement.cap), (3, 3, 20));
        assert_eq!(judgement.branches, vec!["Chain".to_string()]);
        assert_eq!(
            judgement.nodes[0],
            ArpgSkillNode {
                id: 301,
                kind: 2,
                column: 0,
                row: 1,
                parent: 0,
                max_rank: 3,
                rank: 2,
                icon_spell: 20186,
                name: "Chain of Judgement".into(),
                text: "Chains.".into(),
            }
        );
        assert!(read_arpg_skills(&mut [2u8].as_slice()).is_err());
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
