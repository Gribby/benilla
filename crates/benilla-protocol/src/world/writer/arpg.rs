//! Fork-only, not 1.12.1: `CMSG_ARPG_ACTION`, the ARPG client's one packet to a server built with
//! the ARPG patch (cmangos `Arpg/ArpgHandler.cpp`). A stock 1.12.1 server drops the connection on
//! an opcode past its table, so the app sends these only when the ARPG view is on.
//!
//! Body: `u8` kind, then by kind: hello `u8` version; swing start `u64` intended; swing stop
//! nothing; cast `u32` spell id, `u8` aim (0 an enemy, 1 an ally), three `f32` WoW world coords and
//! `u64` intended; aim three `f32` WoW world coords and `u64` intended, the cursor while a cast runs,
//! which re-aims it; loot `u64` corpse and `u8` loot slot (0xFF the gold), a ground pick-up; loot
//! query `u64` corpse, which asks for that corpse's ground loot list.
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

impl WorldWriter {
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

    /// Re-aim the running cast at a world point; unanswered.
    pub fn arpg_aim(&mut self, at: [f32; 3], intended: u64) -> Result<()> {
        self.send(CMSG_ARPG_ACTION, &arpg_aim_body(at, intended))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
