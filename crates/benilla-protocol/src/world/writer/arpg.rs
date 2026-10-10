//! Fork-only, not 1.12.1: `CMSG_ARPG_ACTION`, the ARPG client's one packet to a server built with
//! the ARPG patch (cmangos `Arpg/ArpgHandler.cpp`). A stock 1.12.1 server drops the connection on
//! an opcode past its table, so the app sends these only when the ARPG view is on.
//!
//! Body: `u8` kind, then by kind: hello `u8` version; swing start `u64` intended; swing stop
//! nothing; cast `u32` spell id, `u8` aim (0 an enemy, 1 an ally), three `f32` WoW world coords and
//! `u64` intended; aim three `f32` WoW world coords and `u64` intended, the cursor while a cast runs,
//! which re-aims it; loot `u64` corpse and `u8` loot slot (0xFF the gold), a ground pick-up; loot
//! query `u64` corpse, which asks for that corpse's ground loot list; dev loot `u8` quality
//! (0xFF a random mix), `u8` count, `u8` item level (0 the player's), which a server with
//! `Arpg.DevTools` on answers by dropping that loot at the player's feet, for testing; tree spend
//! `u16` node, tree respec and tree query (nothing more), the ARPG skill tree's.
//! `intended` is the unit under the cursor (0 for none): the server lets it catch
//! the swing or the spell even when it would not pick it itself (a neutral, a sheep).

use anyhow::Result;

use super::WorldWriter;

/// The opcode, one past 1.12.1's last (`SMSG_DEFENSE_MESSAGE` 0x33B).
pub const CMSG_ARPG_ACTION: u16 = 0x033C;
/// The ARPG protocol version this client speaks; the server refuses another.
pub const ARPG_PROTOCOL_VERSION: u8 = 2;

const KIND_HELLO: u8 = 0;
const KIND_SWING_START: u8 = 1;
const KIND_SWING_STOP: u8 = 2;
const KIND_CAST: u8 = 3;
const KIND_AIM: u8 = 4;
const KIND_LOOT: u8 = 5;
const KIND_LOOT_QUERY: u8 = 6;
const KIND_DEV_LOOT: u8 = 7;
const KIND_TREE_SPEND: u8 = 8;
const KIND_TREE_RESPEC: u8 = 9;
const KIND_TREE_QUERY: u8 = 10;
const KIND_TREE_REFUND: u8 = 11;
const KIND_SKILL_SLOT: u8 = 12;
const KIND_SKILL_SPEND: u8 = 13;
const KIND_SKILL_REFUND: u8 = 14;
const KIND_SKILL_RESPEC: u8 = 15;
const KIND_DEV_PACK: u8 = 16;
const KIND_DODGE: u8 = 17;
const KIND_FLASK: u8 = 18;
const KIND_TIER: u8 = 19;
const KIND_UNSEAL: u8 = 20;
const KIND_SOCKET: u8 = 21;
const KIND_TOWN_PORTAL: u8 = 22;

/// Dev loot's quality byte for a random mix of qualities.
pub const DEV_LOOT_MIXED: u8 = 0xFF;
/// Dev loot's quality byte for Codex pages, fragments and runes.
pub const DEV_LOOT_CODEX: u8 = 0xFE;

/// What an ARPG cast aims at, by the spell's own target word: the server resolves the unit.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ArpgAim {
    /// The first enemy along the aimed line, or the nearest in the swing arc for melee range.
    Enemy,
    /// The friend nearest the aim point, else the caster.
    Ally,
}

/// The hello body.
pub fn arpg_hello_body() -> Vec<u8> {
    vec![KIND_HELLO, ARPG_PROTOCOL_VERSION]
}

/// A swing start body, at the unit under the cursor (`intended`, 0 for none).
pub fn arpg_swing_start_body(intended: u64) -> Vec<u8> {
    let mut body = Vec::with_capacity(9);
    body.push(KIND_SWING_START);
    body.extend_from_slice(&intended.to_le_bytes());
    body
}

/// A swing stop body.
pub fn arpg_swing_stop_body() -> Vec<u8> {
    vec![KIND_SWING_STOP]
}

/// A cast body: the spell, what it aims at, the aim point in WoW coordinates, and the unit under
/// the cursor (`intended`, 0 for none).
pub fn arpg_cast_body(spell_id: u32, aim: ArpgAim, at: [f32; 3], intended: u64) -> Vec<u8> {
    let mut body = Vec::with_capacity(26);
    body.push(KIND_CAST);
    body.extend_from_slice(&spell_id.to_le_bytes());
    body.push(match aim {
        ArpgAim::Enemy => 0,
        ArpgAim::Ally => 1,
    });
    for c in at {
        body.extend_from_slice(&c.to_le_bytes());
    }
    body.extend_from_slice(&intended.to_le_bytes());
    body
}

/// An aim body: where the cursor is now, in WoW coordinates, and the unit under it (`intended`, 0
/// for none). The server re-aims the running cast with it, so the shot goes where the player points
/// when it is released.
pub fn arpg_aim_body(at: [f32; 3], intended: u64) -> Vec<u8> {
    let mut body = Vec::with_capacity(21);
    body.push(KIND_AIM);
    for c in at {
        body.extend_from_slice(&c.to_le_bytes());
    }
    body.extend_from_slice(&intended.to_le_bytes());
    body
}

/// A dodge body: roll toward the WoW-space point `(x, y)`.
pub fn arpg_dodge_body(x: f32, y: f32) -> Vec<u8> {
    let mut body = Vec::with_capacity(9);
    body.push(KIND_DODGE);
    body.extend_from_slice(&x.to_le_bytes());
    body.extend_from_slice(&y.to_le_bytes());
    body
}

/// A pick-up body: the item in loot `slot` off `corpse`'s ground loot, or its gold
/// ([`crate::messages::arpg::LOOT_SLOT_GOLD`]).
pub fn arpg_loot_body(corpse: u64, slot: u8) -> Vec<u8> {
    let mut body = Vec::with_capacity(10);
    body.push(KIND_LOOT);
    body.extend_from_slice(&corpse.to_le_bytes());
    body.push(slot);
    body
}

/// A loot query body: send `corpse`'s ground loot list.
pub fn arpg_loot_query_body(corpse: u64) -> Vec<u8> {
    let mut body = Vec::with_capacity(9);
    body.push(KIND_LOOT_QUERY);
    body.extend_from_slice(&corpse.to_le_bytes());
    body
}

/// The dev loot body: `count` items of `quality` (or [`DEV_LOOT_MIXED`]) around item `level`.
pub fn arpg_dev_loot_body(quality: u8, count: u8, level: u8) -> Vec<u8> {
    vec![KIND_DEV_LOOT, quality, count, level]
}

/// The web take body: a point into `node`.
pub fn arpg_tree_spend_body(node: u16) -> Vec<u8> {
    let mut body = vec![KIND_TREE_SPEND];
    body.extend_from_slice(&node.to_le_bytes());
    body
}

/// The skill slot body: `skill` into `slot` (0 empties it).
pub fn arpg_skill_slot_body(slot: u8, skill: u8) -> Vec<u8> {
    vec![KIND_SKILL_SLOT, slot, skill]
}

/// A skill node body: a rank into `node`, or `refund` one back.
pub fn arpg_skill_node_body(node: u16, refund: bool) -> Vec<u8> {
    let mut body = vec![if refund {
        KIND_SKILL_REFUND
    } else {
        KIND_SKILL_SPEND
    }];
    body.extend_from_slice(&node.to_le_bytes());
    body
}

/// The web give-back body: `node`'s point back.
pub fn arpg_tree_refund_body(node: u16) -> Vec<u8> {
    let mut body = vec![KIND_TREE_REFUND];
    body.extend_from_slice(&node.to_le_bytes());
    body
}

impl WorldWriter {
    /// Take a node of the ARPG passive web.
    pub fn arpg_tree_spend(&mut self, node: u16) -> Result<()> {
        self.send(CMSG_ARPG_ACTION, &arpg_tree_spend_body(node))
    }

    /// Put an ARPG skill in a specialisation slot.
    pub fn arpg_skill_slot(&mut self, slot: u8, skill: u8) -> Result<()> {
        self.send(CMSG_ARPG_ACTION, &arpg_skill_slot_body(slot, skill))
    }

    /// Take a rank of an ARPG skill tree node, or give one back.
    pub fn arpg_skill_node(&mut self, node: u16, refund: bool) -> Result<()> {
        self.send(CMSG_ARPG_ACTION, &arpg_skill_node_body(node, refund))
    }

    /// Dev tools: the nearest mob forms a pack of `size` (0: by level) and `tier` (0 by chance,
    /// 1 no champions, 2 a champion pack, 3 a rare).
    pub fn arpg_dev_pack(&mut self, size: u8, tier: u8) -> Result<()> {
        self.send(CMSG_ARPG_ACTION, &[KIND_DEV_PACK, size, tier])
    }

    /// Give back every point in an ARPG skill.
    pub fn arpg_skill_respec(&mut self, skill: u8) -> Result<()> {
        self.send(CMSG_ARPG_ACTION, &[KIND_SKILL_RESPEC, skill])
    }

    /// Give a node of the ARPG passive web back.
    pub fn arpg_tree_refund(&mut self, node: u16) -> Result<()> {
        self.send(CMSG_ARPG_ACTION, &arpg_tree_refund_body(node))
    }

    /// Give the whole ARPG passive web back.
    pub fn arpg_tree_respec(&mut self) -> Result<()> {
        self.send(CMSG_ARPG_ACTION, &[KIND_TREE_RESPEC])
    }

    /// Ask for the ARPG skill tree.
    pub fn arpg_tree_query(&mut self) -> Result<()> {
        self.send(CMSG_ARPG_ACTION, &[KIND_TREE_QUERY])
    }

    /// The hello: this player is on the ARPG client.
    pub fn arpg_hello(&mut self) -> Result<()> {
        self.send(CMSG_ARPG_ACTION, &arpg_hello_body())
    }

    /// Start the held swing at `intended` (0 for none), or stop it (`None`).
    pub fn arpg_swing(&mut self, start: Option<u64>) -> Result<()> {
        let body = match start {
            Some(intended) => arpg_swing_start_body(intended),
            None => arpg_swing_stop_body(),
        };
        self.send(CMSG_ARPG_ACTION, &body)
    }

    /// Cast `spell_id` aimed at a world point; answered as a `CMSG_CAST_SPELL` is.
    pub fn arpg_cast(
        &mut self,
        spell_id: u32,
        aim: ArpgAim,
        at: [f32; 3],
        intended: u64,
    ) -> Result<()> {
        self.send(
            CMSG_ARPG_ACTION,
            &arpg_cast_body(spell_id, aim, at, intended),
        )
    }

    /// Pick up the item in loot `slot` (or the gold) off `corpse`'s ground loot; answered with
    /// that corpse's list.
    pub fn arpg_loot(&mut self, corpse: u64, slot: u8) -> Result<()> {
        self.send(CMSG_ARPG_ACTION, &arpg_loot_body(corpse, slot))
    }

    /// Ask for `corpse`'s ground loot list; answered with it, if it is a corpse near us.
    pub fn arpg_loot_query(&mut self, corpse: u64) -> Result<()> {
        self.send(CMSG_ARPG_ACTION, &arpg_loot_query_body(corpse))
    }

    /// Ask a dev-tools server to drop test loot at the player's feet.
    pub fn arpg_dev_loot(&mut self, quality: u8, count: u8, level: u8) -> Result<()> {
        self.send(CMSG_ARPG_ACTION, &arpg_dev_loot_body(quality, count, level))
    }

    /// Re-aim the running cast at a world point; unanswered.
    pub fn arpg_aim(&mut self, at: [f32; 3], intended: u64) -> Result<()> {
        self.send(CMSG_ARPG_ACTION, &arpg_aim_body(at, intended))
    }

    /// Roll toward the WoW-space point `(x, y)`; the server answers with a knockback.
    pub fn arpg_dodge(&mut self, x: f32, y: f32) -> Result<()> {
        self.send(CMSG_ARPG_ACTION, &arpg_dodge_body(x, y))
    }

    /// Drink a charge of the flask.
    pub fn arpg_flask(&mut self) -> Result<()> {
        self.send(CMSG_ARPG_ACTION, &[KIND_FLASK])
    }

    /// Open a town portal, or go back through the open one (cmangos `Arpg/ArpgActions.h`).
    pub fn arpg_town_portal(&mut self) -> Result<()> {
        self.send(CMSG_ARPG_ACTION, &[KIND_TOWN_PORTAL])
    }

    /// Ask for dungeon difficulty `tier` (0 Normal to 5 Torment III).
    pub fn arpg_tier(&mut self, tier: u8) -> Result<()> {
        self.send(CMSG_ARPG_ACTION, &[KIND_TIER, tier])
    }

    /// Unseal capstone `node` with Codex fragments.
    pub fn arpg_unseal(&mut self, node: u16) -> Result<()> {
        let [lo, hi] = node.to_le_bytes();
        self.send(CMSG_ARPG_ACTION, &[KIND_UNSEAL, lo, hi])
    }

    /// Socket `rune` (0 empties the socket) in `skill`.
    pub fn arpg_socket(&mut self, skill: u8, rune: u8) -> Result<()> {
        self.send(CMSG_ARPG_ACTION, &[KIND_SOCKET, skill, rune])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_dodge_body_is_kind_then_the_point() {
        let body = arpg_dodge_body(1.5, -2.0);
        assert_eq!(body.len(), 9);
        assert_eq!(body[0], KIND_DODGE);
        assert_eq!(f32::from_le_bytes(body[1..5].try_into().unwrap()), 1.5);
        assert_eq!(f32::from_le_bytes(body[5..9].try_into().unwrap()), -2.0);
    }

    #[test]
    fn the_cast_body_is_kind_spell_aim_the_point_and_the_intended_unit() {
        let body = arpg_cast_body(133, ArpgAim::Ally, [1.0, -2.0, 3.5], 0xF130_0000_0000_002A);
        assert_eq!(body.len(), 26);
        assert_eq!(body[0], KIND_CAST);
        assert_eq!(u32::from_le_bytes(body[1..5].try_into().unwrap()), 133);
        assert_eq!(body[5], 1);
        assert_eq!(f32::from_le_bytes(body[10..14].try_into().unwrap()), -2.0);
        assert_eq!(
            u64::from_le_bytes(body[18..26].try_into().unwrap()),
            0xF130_0000_0000_002A
        );
    }

    #[test]
    fn the_tree_spend_body_is_kind_and_node() {
        assert_eq!(arpg_tree_spend_body(0x0102), vec![8, 0x02, 0x01]);
        assert_eq!(arpg_tree_refund_body(0x0102), vec![11, 0x02, 0x01]);
        assert_eq!(arpg_skill_slot_body(2, 3), vec![12, 2, 3]);
        assert_eq!(arpg_skill_node_body(301, false), vec![13, 0x2D, 0x01]);
        assert_eq!(arpg_skill_node_body(301, true), vec![14, 0x2D, 0x01]);
    }

    #[test]
    fn the_dev_loot_body_is_kind_quality_count_and_level() {
        assert_eq!(
            arpg_dev_loot_body(DEV_LOOT_MIXED, 6, 0),
            vec![7, 0xFF, 6, 0]
        );
    }

    #[test]
    fn the_aim_body_is_kind_the_point_and_the_intended_unit() {
        let body = arpg_aim_body([1.0, -2.0, 3.5], 9);
        assert_eq!(body.len(), 21);
        assert_eq!(body[0], KIND_AIM);
        assert_eq!(f32::from_le_bytes(body[5..9].try_into().unwrap()), -2.0);
        assert_eq!(u64::from_le_bytes(body[13..21].try_into().unwrap()), 9);
    }

    #[test]
    fn the_loot_body_is_kind_the_corpse_and_the_slot() {
        let body = arpg_loot_body(0x0102_0304_0506_0708, 0xFF);
        assert_eq!(body.len(), 10);
        assert_eq!(body[0], KIND_LOOT);
        assert_eq!(body[1], 0x08);
        assert_eq!(body[9], 0xFF);
        assert_eq!(
            arpg_loot_query_body(7),
            vec![KIND_LOOT_QUERY, 7, 0, 0, 0, 0, 0, 0, 0]
        );
    }

    #[test]
    fn hello_and_swing_bodies() {
        assert_eq!(arpg_hello_body(), vec![0, ARPG_PROTOCOL_VERSION]);
        assert_eq!(arpg_swing_start_body(7), vec![1, 7, 0, 0, 0, 0, 0, 0, 0]);
        assert_eq!(arpg_swing_stop_body(), vec![2]);
    }
}
