//! Fork-only, not 1.12.1: the ARPG server's messages to the benilla ARPG client (cmangos
//! `Arpg/ArpgLoot.h`). A stock server never sends them; the opcodes sit past 1.12.1's last.

use std::io;

use crate::wire::{
    capacity_hint, read_cstring, read_f32_le, read_u16_le, read_u32_le, read_u64_le, read_u8,
};

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
    /// A capstone sealed until its Codex page is read (version 2).
    pub sealed: bool,
    /// Where a capstone's page drops ("Herod, Scarlet Monastery"); empty for other nodes.
    pub page_home: String,
}

/// A rune: a portable mechanic socketed in a skill tree (version 2).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ArpgRune {
    pub id: u8,
    pub icon_spell: u32,
    pub name: String,
    pub text: String,
    /// How many the character holds, socketed ones included.
    pub held: u8,
    /// The skills it fits, a bit per skill id.
    pub fits: u8,
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
    /// The rune in its socket (0 none), and the points the socket opens at (version 2).
    pub rune: u8,
    pub socket_points: u8,
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
    /// Codex fragments held, and how many unseal a capstone (version 2).
    pub fragments: u16,
    pub fragments_per_page: u8,
    pub runes: Vec<ArpgRune>,
}

/// `SMSG_ARPG_SKILLS`: `u8` version (1), `u16` points total and spent, `u8` level; `u8` slot count,
/// per slot `u8` level and skill; `u8` skill count, per skill `u8` id, `u32` icon spell, C strings
/// name and text, `u8` spent and cap, `u8` branch count and a C string each, `u8` node count, per
/// node `u16` id, `u8` kind, column and row, `u16` parent, `u8` max rank and rank, `u32` icon
/// spell, C strings name and text. Version 2 adds, per skill after the cap, `u8` socketed rune and
/// the points the socket opens at; per node after the text, `u8` sealed and a C string page home;
/// and at the end `u16` fragments, `u8` fragments a page takes, `u8` rune count and per rune `u8`
/// id, `u32` icon spell, C strings name and text, `u8` held and `u8` the skills it fits.
pub fn read_arpg_skills(r: &mut impl io::Read) -> io::Result<ArpgSkills> {
    let version = read_u8(r)?;
    if version != 1 && version != 2 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!("ARPG skills version {version}, this client reads 1 and 2"),
        ));
    }
    let v2 = version >= 2;
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
        let (rune, socket_points) = if v2 {
            (read_u8(r)?, read_u8(r)?)
        } else {
            (0, 0)
        };
        let nb = read_u8(r)?;
        let mut branches = Vec::with_capacity(capacity_hint(nb, 8));
        for _ in 0..nb {
            branches.push(read_cstring(r)?);
        }
        let nn = read_u8(r)?;
        let mut nodes = Vec::with_capacity(capacity_hint(nn, 32));
        for _ in 0..nn {
            let mut node = ArpgSkillNode {
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
                sealed: false,
                page_home: String::new(),
            };
            if v2 {
                node.sealed = read_u8(r)? != 0;
                node.page_home = read_cstring(r)?;
            }
            nodes.push(node);
        }
        skills.push(ArpgSkill {
            id,
            icon_spell,
            name,
            text,
            spent,
            cap,
            rune,
            socket_points,
            branches,
            nodes,
        });
    }
    let (mut fragments, mut fragments_per_page, mut runes) = (0, 0, Vec::new());
    if v2 {
        fragments = read_u16_le(r)?;
        fragments_per_page = read_u8(r)?;
        let nr = read_u8(r)?;
        runes.reserve(capacity_hint(nr, 16));
        for _ in 0..nr {
            runes.push(ArpgRune {
                id: read_u8(r)?,
                icon_spell: read_u32_le(r)?,
                name: read_cstring(r)?,
                text: read_cstring(r)?,
                held: read_u8(r)?,
                fits: read_u8(r)?,
            });
        }
    }
    Ok(ArpgSkills {
        points_total,
        points_spent,
        level,
        slots,
        skills,
        fragments,
        fragments_per_page,
        runes,
    })
}

/// The champions and rares near the player (cmangos `Arpg/ArpgPacks.h`), each sent once.
pub const SMSG_ARPG_CHAMPIONS: u16 = 0x0341;

/// The rolled affixes of the player's items (cmangos `Arpg/ArpgAffixes.h`): after the hello, and
/// when an affixed item is stored.
pub const SMSG_ARPG_ITEM_AFFIXES: u16 = 0x0343;

/// `SMSG_ARPG_ITEM_AFFIXES`: `u16` count, per item `u64` guid and a C string of lines, `\n` between.
pub fn read_arpg_item_affixes(r: &mut impl io::Read) -> io::Result<Vec<(u64, String)>> {
    let n = read_u16_le(r)?;
    let mut out = Vec::with_capacity(usize::from(n).min(256));
    for _ in 0..n {
        out.push((read_u64_le(r)?, read_cstring(r)?));
    }
    Ok(out)
}

/// The flask's charges and the dodge's cooldown (cmangos `Arpg/ArpgActions.h`), on a change.
pub const SMSG_ARPG_STATUS: u16 = 0x0342;

/// The player's flask and dodge roll.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct ArpgStatus {
    pub flask_charges: u8,
    pub flask_max: u8,
    /// The next charge's progress, 0 to 100.
    pub flask_progress: u8,
    /// Milliseconds until the dodge is ready, 0 when it is.
    pub dodge_ready_ms: u32,
    pub dodge_cooldown_ms: u32,
}

/// `SMSG_ARPG_STATUS`: `u8` charges, `u8` max, `u8` progress, `u32` ms until the dodge is ready,
/// `u32` the dodge's cooldown.
pub fn read_arpg_status(r: &mut impl io::Read) -> io::Result<ArpgStatus> {
    Ok(ArpgStatus {
        flask_charges: read_u8(r)?,
        flask_max: read_u8(r)?,
        flask_progress: read_u8(r)?,
        dodge_ready_ms: read_u32_le(r)?,
        dodge_cooldown_ms: read_u32_le(r)?,
    })
}

/// A telegraphed attack winding up or broken off (cmangos `Arpg/ArpgThreats.h`).
pub const SMSG_ARPG_TELEGRAPH: u16 = 0x0344;

/// A telegraph's message kinds.
pub const TELEGRAPH_WIND_UP: u8 = 1;
pub const TELEGRAPH_BROKEN: u8 = 2;
/// Its shapes: a ring round the caster, a cone in front of it, a circle at a player's feet.
pub const TELEGRAPH_RING: u8 = 1;
pub const TELEGRAPH_CONE: u8 = 2;
pub const TELEGRAPH_BLAST: u8 = 3;
/// A line from the caster along its facing: a lunge or a charge. Its radius is its length and
/// its half angle field its half width, in yards.
pub const TELEGRAPH_LINE: u8 = 4;
/// The grade of an ordinary creature's move, below an elite's.
pub const TELEGRAPH_GRADE_MINOR: u8 = 5;

/// A health globe dropped or taken (cmangos `Arpg/ArpgActions.h`).
pub const SMSG_ARPG_GLOBE: u16 = 0x0345;
pub const GLOBE_DROPPED: u8 = 1;
pub const GLOBE_GONE: u8 = 2;

/// A health globe on the ground.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct ArpgGlobe {
    /// [`GLOBE_DROPPED`] or [`GLOBE_GONE`].
    pub kind: u8,
    pub id: u32,
    /// Where it lies, in WoW coordinates.
    pub pos: [f32; 3],
}

/// `SMSG_ARPG_GLOBE`: `u8` kind, `u32` globe, `f32` x y z.
pub fn read_arpg_globe(r: &mut impl io::Read) -> io::Result<ArpgGlobe> {
    Ok(ArpgGlobe {
        kind: read_u8(r)?,
        id: read_u32_le(r)?,
        pos: [read_f32_le(r)?, read_f32_le(r)?, read_f32_le(r)?],
    })
}

/// One telegraph: where it lands and when.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct ArpgTelegraph {
    /// [`TELEGRAPH_WIND_UP`] or [`TELEGRAPH_BROKEN`].
    pub kind: u8,
    pub serial: u32,
    pub caster: u64,
    pub shape: u8,
    /// 0 elite, 1 champion, 2 rare, 3 boss, 4 raid boss.
    pub grade: u8,
    /// The centre, or the cone's apex, in WoW coordinates.
    pub pos: [f32; 3],
    /// The cone's facing, radians, WoW's.
    pub orientation: f32,
    pub radius: f32,
    /// The cone's half angle, radians.
    pub half_angle: f32,
    pub wind_up_ms: u32,
}

/// `SMSG_ARPG_TELEGRAPH`: `u8` kind, `u32` serial, `u64` caster, `u8` shape, `u8` grade, `f32`
/// x y z, `f32` orientation, `f32` radius, `f32` half angle, `u32` wind-up ms.
pub fn read_arpg_telegraph(r: &mut impl io::Read) -> io::Result<ArpgTelegraph> {
    Ok(ArpgTelegraph {
        kind: read_u8(r)?,
        serial: read_u32_le(r)?,
        caster: read_u64_le(r)?,
        shape: read_u8(r)?,
        grade: read_u8(r)?,
        pos: [read_f32_le(r)?, read_f32_le(r)?, read_f32_le(r)?],
        orientation: read_f32_le(r)?,
        radius: read_f32_le(r)?,
        half_angle: read_f32_le(r)?,
        wind_up_ms: read_u32_le(r)?,
    })
}

/// A champion or rare: its tier (1 champion, 2 rare), its own name (empty: the creature's), and
/// its affixes' names.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ArpgChampion {
    pub guid: u64,
    pub tier: u8,
    pub name: String,
    pub affixes: Vec<String>,
}

/// `SMSG_ARPG_CHAMPIONS`: `u8` count, per champion `u64` guid, `u8` tier, a C string name, `u8`
/// affix count and a C string each.
pub fn read_arpg_champions(r: &mut impl io::Read) -> io::Result<Vec<ArpgChampion>> {
    let n = read_u8(r)?;
    let mut out = Vec::with_capacity(capacity_hint(n, 64));
    for _ in 0..n {
        let guid = read_u64_le(r)?;
        let tier = read_u8(r)?;
        let name = read_cstring(r)?;
        let na = read_u8(r)?;
        let mut affixes = Vec::with_capacity(capacity_hint(na, 8));
        for _ in 0..na {
            affixes.push(read_cstring(r)?);
        }
        out.push(ArpgChampion {
            guid,
            tier,
            name,
            affixes,
        });
    }
    Ok(out)
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
                sealed: false,
                page_home: String::new(),
            }
        );
        assert!(skills.runes.is_empty());
        assert!(read_arpg_skills(&mut [3u8].as_slice()).is_err());
    }

    #[test]
    fn skills_v2_read_sockets_seals_fragments_and_runes() {
        let mut body = vec![2];
        body.extend_from_slice(&40u16.to_le_bytes());
        body.extend_from_slice(&12u16.to_le_bytes());
        body.push(30);
        body.push(1);
        body.extend_from_slice(&[1, 3]);
        body.push(1);
        body.push(3);
        body.extend_from_slice(&20271u32.to_le_bytes());
        body.extend_from_slice(b"Judgement\0Unleash your Seal.\0");
        body.extend_from_slice(&[12, 20, 1, 10, 0]);
        body.push(1);
        body.extend_from_slice(&303u16.to_le_bytes());
        body.extend_from_slice(&[4, 0, 3]);
        body.extend_from_slice(&302u16.to_le_bytes());
        body.extend_from_slice(&[1, 0]);
        body.extend_from_slice(&20184u32.to_le_bytes());
        body.extend_from_slice(b"Final Verdict\0Readies Judgement.\0");
        body.push(1);
        body.extend_from_slice(b"Scarlet Commander Mograine, Scarlet Monastery\0");
        body.extend_from_slice(&7u16.to_le_bytes());
        body.push(5);
        body.push(1);
        body.push(1);
        body.extend_from_slice(&20186u32.to_le_bytes());
        body.extend_from_slice(b"Rune of Chains\0Chains.\0");
        body.extend_from_slice(&[2, 0b0010_1000]);
        let skills = read_arpg_skills(&mut body.as_slice()).unwrap();
        let judgement = &skills.skills[0];
        assert_eq!((judgement.rune, judgement.socket_points), (1, 10));
        assert!(judgement.nodes[0].sealed);
        assert_eq!(
            judgement.nodes[0].page_home,
            "Scarlet Commander Mograine, Scarlet Monastery"
        );
        assert_eq!((skills.fragments, skills.fragments_per_page), (7, 5));
        assert_eq!(
            skills.runes,
            vec![ArpgRune {
                id: 1,
                icon_spell: 20186,
                name: "Rune of Chains".into(),
                text: "Chains.".into(),
                held: 2,
                fits: 0b0010_1000,
            }]
        );
    }

    #[test]
    fn item_affixes_read_as_guid_and_lines_pairs() {
        let mut body = 1u16.to_le_bytes().to_vec();
        body.extend_from_slice(&0x4000_0000_0000_0123u64.to_le_bytes());
        body.extend_from_slice(b"+8 Strength\n+5% Holy damage\0");
        assert_eq!(
            read_arpg_item_affixes(&mut body.as_slice()).unwrap(),
            vec![(
                0x4000_0000_0000_0123,
                "+8 Strength\n+5% Holy damage".to_string()
            )]
        );
    }

    #[test]
    fn a_globe_reads_its_kind_id_and_place() {
        let mut body = vec![GLOBE_DROPPED];
        body.extend_from_slice(&9u32.to_le_bytes());
        for f in [1.0f32, 2.0, 3.0] {
            body.extend_from_slice(&f.to_le_bytes());
        }
        assert_eq!(
            read_arpg_globe(&mut body.as_slice()).unwrap(),
            ArpgGlobe {
                kind: GLOBE_DROPPED,
                id: 9,
                pos: [1.0, 2.0, 3.0],
            }
        );
    }

    #[test]
    fn a_telegraph_reads_its_shape_place_and_wind_up() {
        let mut body = vec![1];
        body.extend_from_slice(&7u32.to_le_bytes());
        body.extend_from_slice(&0xF130_0000_0000_0042u64.to_le_bytes());
        body.extend_from_slice(&[2, 3]);
        for f in [10.0f32, -20.0, 5.5, 1.25, 14.0, 0.785] {
            body.extend_from_slice(&f.to_le_bytes());
        }
        body.extend_from_slice(&1500u32.to_le_bytes());
        let t = read_arpg_telegraph(&mut body.as_slice()).unwrap();
        assert_eq!(
            t,
            ArpgTelegraph {
                kind: TELEGRAPH_WIND_UP,
                serial: 7,
                caster: 0xF130_0000_0000_0042,
                shape: TELEGRAPH_CONE,
                grade: 3,
                pos: [10.0, -20.0, 5.5],
                orientation: 1.25,
                radius: 14.0,
                half_angle: 0.785,
                wind_up_ms: 1500,
            }
        );
    }

    #[test]
    fn the_status_reads_the_flask_then_the_dodge() {
        let mut body = vec![2, 3, 40];
        body.extend_from_slice(&1200u32.to_le_bytes());
        body.extend_from_slice(&2500u32.to_le_bytes());
        assert_eq!(
            read_arpg_status(&mut body.as_slice()).unwrap(),
            ArpgStatus {
                flask_charges: 2,
                flask_max: 3,
                flask_progress: 40,
                dodge_ready_ms: 1200,
                dodge_cooldown_ms: 2500,
            }
        );
    }

    #[test]
    fn champions_read_their_tier_name_and_affixes() {
        let mut body = vec![1];
        body.extend_from_slice(&0xF130_0000_0000_0042u64.to_le_bytes());
        body.push(2);
        body.extend_from_slice(b"Gorefang the Swift\0");
        body.push(2);
        body.extend_from_slice(b"Extra Fast\0Vampiric\0");
        let champions = read_arpg_champions(&mut body.as_slice()).unwrap();
        assert_eq!(
            champions,
            vec![ArpgChampion {
                guid: 0xF130_0000_0000_0042,
                tier: 2,
                name: "Gorefang the Swift".into(),
                affixes: vec!["Extra Fast".into(), "Vampiric".into()],
            }]
        );
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
