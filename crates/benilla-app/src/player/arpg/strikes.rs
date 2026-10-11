//! Fork-only, not 1.12.1: the ARPG view's game-art hits. The server's added hits (an arc, a burst,
//! a cleave, a keystone's strike) carry no missile of their own, so each sends its impact
//! (`SMSG_ARPG_IMPACT`, cmangos `Arpg/ArpgUniques.h`): the spell's own impact kit plays on the
//! struck unit, as a missile's arrival would play it, or its school's stock one (Smite for Holy,
//! Fireball for Fire...) for a spell that has none, such as Seal of Righteousness's strike.
//!
//! Gore, from the same `UnitBlood` models the melee spurt plays: a crit by the player sprays
//! blood from the front of the struck unit whatever struck it, and a kill by a heavy blow bursts
//! the large pair, front and back, as the corpse is flung (`super::impact`).

use bevy::prelude::*;

use benilla_protocol::messages::ArpgImpact;
use benilla_protocol::{SessionEvent, SessionEventKind};

use crate::combat_text::CombatTextSpawn;
use crate::creature_anim::spell_visual::impact_kit_id;
use crate::creature_anim::{
    arpg_spurts, BloodTables, FxClass, FxStage, KitPush, PlaySeq, SpellKitFx, SpellVisuals,
};
use crate::net::{GuidIndex, NetEntity, NetHandlerApp, SelfPlayer};

use super::juice::ArpgKill;

/// The combat text category of a crit.
const CATEGORY_CRIT: u8 = 2;

/// The stock spell whose impact stands in for a school's, by school mask bit (Holy to Arcane).
const SCHOOL_IMPACT: [(u32, u32); 6] = [
    (0x02, 585),  // Holy: Smite
    (0x04, 133),  // Fire: Fireball
    (0x08, 403),  // Nature: Lightning Bolt
    (0x10, 116),  // Frost: Frostbolt
    (0x20, 686),  // Shadow: Shadow Bolt
    (0x40, 7268), // Arcane: Arcane Missiles' missile
];

/// The impacts waiting for the view.
#[derive(Resource, Default)]
pub(crate) struct ArpgImpacts(Vec<ArpgImpact>);

/// Registered whatever the view, so every session event kind has its owner.
pub(super) fn register_net(app: &mut App) {
    app.init_resource::<ArpgImpacts>()
        .net_handler(SessionEventKind::ArpgImpact, on_impact);
}

fn on_impact(In(ev): In<SessionEvent>, mut state: ResMut<ArpgImpacts>) {
    if let SessionEvent::ArpgImpact { impact } = ev {
        if state.0.len() >= 128 {
            state.0.remove(0);
        }
        state.0.push(impact);
    }
}

pub(super) fn plugin(app: &mut App) {
    app.add_systems(Update, (play_impacts, spray_gore));
}

/// The stand-in spell for a hit of `school_mask` with no impact of its own.
fn school_stand_in(school_mask: u32) -> Option<u32> {
    SCHOOL_IMPACT
        .iter()
        .find(|(bit, _)| school_mask & bit != 0)
        .map(|&(_, spell)| spell)
}

/// Each impact's kit on its unit.
fn play_impacts(
    mut state: ResMut<ArpgImpacts>,
    index: Res<GuidIndex>,
    spells: Option<Res<crate::ui_action::Spells>>,
    visuals: Option<Res<SpellVisuals>>,
    mut seq: ResMut<PlaySeq>,
    mut out: MessageWriter<KitPush>,
) {
    if state.0.is_empty() {
        return;
    }
    let impacts = std::mem::take(&mut state.0);
    let (Some(spells), Some(visuals)) = (spells, visuals) else {
        return;
    };
    for impact in impacts {
        let Some(&entity) = index.0.get(&impact.unit) else {
            continue;
        };
        let kit = impact_kit_id(&spells, &visuals, impact.spell).or_else(|| {
            school_stand_in(impact.school_mask)
                .and_then(|spell| impact_kit_id(&spells, &visuals, spell))
        });
        if let Some(kit_id) = kit {
            out.write(KitPush {
                entity,
                kit_id,
                seq: seq.next(),
            });
        }
    }
}

/// Blood off a crit, and a burst off a heavy kill.
#[allow(clippy::too_many_arguments)]
fn spray_gore(
    mut spawns: MessageReader<CombatTextSpawn>,
    mut kills: MessageReader<ArpgKill>,
    me: Query<Entity, With<SelfPlayer>>,
    nets: Query<&NetEntity>,
    blood: Option<Res<BloodTables>>,
    creatures: Option<Res<crate::entities::Creatures>>,
    visuals: Option<Res<SpellVisuals>>,
    mut fx: MessageWriter<SpellKitFx>,
) {
    let (Some(blood), Some(creatures), Some(visuals)) = (blood, creatures, visuals) else {
        for _ in spawns.read() {}
        for _ in kills.read() {}
        return;
    };
    let me = me.single().ok();
    let mut spray = |entity: Entity, large: bool| {
        let Some(display) = nets.get(entity).ok().and_then(|n| n.display_id) else {
            return;
        };
        let effects = arpg_spurts(&blood, &creatures, &visuals, display, large);
        if !effects.is_empty() {
            fx.write(SpellKitFx::Begin {
                entity,
                spell_id: 0,
                persistent: false,
                class: FxClass::Hold,
                stage: FxStage::OneShot,
                effects,
            });
        }
    };
    for spawn in spawns.read() {
        if Some(spawn.anchor) != me
            && spawn.category == CATEGORY_CRIT
            && super::fx::is_damage_number(&spawn.text)
        {
            spray(spawn.anchor, false);
        }
    }
    for kill in kills.read() {
        if kill.overkill {
            spray(kill.unit, true);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_school_picks_its_stock_impact_and_physical_has_none() {
        assert_eq!(school_stand_in(0x02), Some(585));
        assert_eq!(school_stand_in(0x10), Some(116));
        assert_eq!(school_stand_in(0x01), None);
    }
}
