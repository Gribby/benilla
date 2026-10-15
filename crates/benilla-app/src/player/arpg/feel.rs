//! Fork-only, not 1.12.1: the ARPG view's feel on top of the hits and kills (`super::juice`,
//! `super::impact`, `super::strikes`):
//!
//! - The player's wounds float in red over them ([`crate::combat_text::ArpgText::Taken`]).
//! - A proc the server names with a target (`SMSG_ARPG_PROC`'s tail: an ailment's burst, a
//!   freeze, Pyroclasm, Detonate...) floats its name and damage over that enemy in its damage
//!   type's colour, plays its type's impact kit there (whose sound comes with it) and swells a
//!   ring of light; one without a target floats nothing (its line is in the combat log). Each
//!   name floats at most once in [`LABEL_GAP_SECS`] and any at most once in
//!   [`LABEL_ANY_GAP_SECS`], so a pack fight stays readable.
//! - A heavy kill (a crit, or a big share of the health) bursts in the killing blow's damage type:
//!   its type's impact kit on the corpse (fire, ice, holy light...), beside `strikes`' gore.
//! - Kills in a row count up on the HUD (`ArpgHud_Streak`), with a flourish at 10, 25, 50, 100.
//! - A champion's or a rare's kill holds every model near it for a beat (`super::impact`). No
//!   screen flash: the beat, the burst and the shake carry it.
//! - The world view blooms: bright light bleeds ([`benilla_world::ffx_glow::FfxBloom`]).

use std::collections::HashMap;

use bevy::prelude::*;

use benilla_protocol::messages::arpg::{PROC_DAMAGE, PROC_EFFECT};
use benilla_protocol::messages::ArpgProc;
use benilla_ui::script::UiScript;

use crate::combat_text::{ArpgText, CombatTextSpawn};
use crate::creature_anim::spell_visual::impact_kit_id;
use crate::creature_anim::{KitPush, PlaySeq, SpellVisuals};
use crate::net::{GuidIndex, SelfPlayer};
use crate::ui_unit::UnitCombatFeedback;

use super::juice::{ArpgKill, ArpgRing};

/// A proc the server sent, for its label and effect.
#[derive(Message, Clone, Debug)]
pub(crate) struct ArpgProcShown(pub(crate) ArpgProc);

/// The combat text category of a crit, which pops.
const CATEGORY_CRIT: u8 = 2;
/// A proc's name floats at most once in this many seconds.
const LABEL_GAP_SECS: f64 = 1.5;
/// Any label floats at most once in this many seconds.
const LABEL_ANY_GAP_SECS: f64 = 0.6;
/// How long the killing blow's type is remembered for a unit, in seconds.
const BLOW_MEMORY_SECS: f64 = 4.0;
/// Kills further apart than this end the streak, in seconds (the HUD's `STREAK_SECS`).
const STREAK_WINDOW_SECS: f64 = 4.0;
/// The world view's bloom: strength and threshold (`FfxBloom`).
const BLOOM_STRENGTH: f32 = 0.55;
const BLOOM_THRESHOLD: f32 = 0.62;
/// A proc's ring, against a kill's.
const PROC_RING_SCALE: f32 = 0.8;

/// The impact kits that stand for a damage type's burst, by school index (Physical to Arcane),
/// best first: the first with an impact kit plays.
const SCHOOL_BURSTS: [&[u32]; 7] = [
    &[],           // Physical: the gore says it
    &[2812, 585],  // Holy: Holy Wrath, Smite
    &[2120, 133],  // Fire: Flamestrike, Fireball
    &[421, 403],   // Nature: Chain Lightning, Lightning Bolt
    &[122, 116],   // Frost: Frost Nova, Frostbolt
    &[17877, 686], // Shadow: Shadowburn, Shadow Bolt
    &[1449, 7268], // Arcane: Arcane Explosion, Arcane Missiles
];

/// A damage type's light, by school index.
const SCHOOL_RGB: [[f32; 3]; 7] = [
    [1.0, 0.92, 0.75],
    [1.0, 0.9, 0.45],
    [1.0, 0.45, 0.12],
    [0.45, 0.95, 0.35],
    [0.5, 0.85, 1.0],
    [0.7, 0.4, 1.0],
    [1.0, 0.5, 0.95],
];

/// The server's damage tag (`Arpg/ArpgTags.h`: Physical, Holy, Fire, Frost, Arcane, Nature,
/// Shadow) as the combat log's school index; `None` for no type.
fn school_of_tag(tag: u8) -> Option<u8> {
    match tag {
        0 => Some(0),
        1 => Some(1),
        2 => Some(2),
        3 => Some(4),
        4 => Some(6),
        5 => Some(3),
        6 => Some(5),
        _ => None,
    }
}

/// The milestone a streak reaches at `n`, if any: its word for the HUD.
fn milestone(n: u32) -> Option<&'static str> {
    match n {
        10 => Some("Rampage"),
        25 => Some("Massacre"),
        50 => Some("Carnage"),
        100 => Some("Annihilation"),
        _ => None,
    }
}

/// The player's run of kills.
#[derive(Resource, Default)]
struct Streak {
    count: u32,
    last: f64,
    /// The highest milestone crossed since the HUD was last told: several kills can land at
    /// once and step over one.
    word: Option<&'static str>,
    /// The count the HUD has not yet been told of.
    dirty: bool,
}

pub(super) fn plugin(app: &mut App) {
    app.add_message::<ArpgProcShown>()
        .add_message::<KilledBy>()
        .init_resource::<Streak>()
        .insert_resource(benilla_world::ffx_glow::FfxBloom {
            strength: BLOOM_STRENGTH,
            threshold: BLOOM_THRESHOLD,
        })
        .add_systems(
            Update,
            (
                wounds_over_me,
                show_procs,
                (remember_blows, burst_kills).chain(),
                push_streak,
            ),
        );
}

/// The player's wounds, in red over them.
fn wounds_over_me(
    mut feedback: MessageReader<UnitCombatFeedback>,
    me: Query<Entity, With<SelfPlayer>>,
    mut text: MessageWriter<CombatTextSpawn>,
) {
    let Ok(me) = me.single() else {
        for _ in feedback.read() {}
        return;
    };
    for f in feedback.read() {
        if f.unit != me || f.action != "WOUND" || f.amount == 0 {
            continue;
        }
        text.write(CombatTextSpawn {
            anchor: me,
            text: f.amount.to_string(),
            category: 0,
            color: None,
            arpg: Some((ArpgText::Taken, f.amount)),
        });
    }
}

/// The impact kit standing for school `school`'s burst, if the client data has one.
fn burst_kit(spells: &crate::ui_action::Spells, visuals: &SpellVisuals, school: u8) -> Option<u32> {
    SCHOOL_BURSTS
        .get(usize::from(school))?
        .iter()
        .find_map(|&spell| impact_kit_id(spells, visuals, spell))
}

/// Each named proc: its label, and over an enemy its type's kit and ring.
#[allow(clippy::too_many_arguments)]
fn show_procs(
    mut procs: MessageReader<ArpgProcShown>,
    index: Res<GuidIndex>,
    places: Query<&Transform>,
    time: Res<Time>,
    spells: Option<Res<crate::ui_action::Spells>>,
    visuals: Option<Res<SpellVisuals>>,
    mut seq: ResMut<PlaySeq>,
    mut kits: MessageWriter<KitPush>,
    mut rings: MessageWriter<ArpgRing>,
    mut text: MessageWriter<CombatTextSpawn>,
    mut last: Local<HashMap<String, f64>>,
    mut last_any: Local<f64>,
) {
    let now = time.elapsed_secs_f64();
    for ArpgProcShown(p) in procs.read() {
        if p.name.is_empty() || !(p.kind == PROC_DAMAGE || p.kind == PROC_EFFECT) {
            continue; // heals and mana float their amounts already (`super::sheet`)
        }
        let target = (p.target != 0)
            .then(|| index.0.get(&p.target).copied())
            .flatten();
        let school = school_of_tag(p.tag);
        // Over an enemy: its type's burst and light, every time.
        if let Some(target) = target {
            if let (Some(school), Some(spells), Some(visuals)) = (school, &spells, &visuals) {
                if let Some(kit_id) = burst_kit(spells, visuals, school) {
                    kits.write(KitPush {
                        entity: target,
                        kit_id,
                        seq: seq.next(),
                    });
                }
                if let Ok(tf) = places.get(target) {
                    rings.write(ArpgRing {
                        at: tf.translation,
                        rgb: SCHOOL_RGB[usize::from(school)],
                        scale: PROC_RING_SCALE,
                    });
                }
            }
        }
        // The label: only over the enemy it struck, once in a while per name, and one at a time
        // at most every LABEL_ANY_GAP_SECS. A proc with no enemy (a champion's affix, a kit's
        // every hit) floats nothing: its line is in the combat log.
        let Some(anchor) = target else {
            continue;
        };
        if last
            .get(&p.name)
            .is_some_and(|at| now - at < LABEL_GAP_SECS)
            || now - *last_any < LABEL_ANY_GAP_SECS
        {
            continue;
        }
        last.insert(p.name.clone(), now);
        *last_any = now;
        text.write(CombatTextSpawn {
            anchor,
            text: p.name.clone(),
            category: CATEGORY_CRIT,
            color: None,
            arpg: Some((
                ArpgText::Label {
                    school: school.unwrap_or(u8::MAX),
                },
                p.amount,
            )),
        });
    }
    last.retain(|_, at| now - *at < LABEL_GAP_SECS * 4.0);
}

/// The damage type of the player's last blow on each unit.
#[derive(Default)]
struct LastBlows(HashMap<Entity, (u8, f64)>);

fn remember_blows(
    mut spawns: MessageReader<CombatTextSpawn>,
    me: Query<(), With<SelfPlayer>>,
    time: Res<Time>,
    mut blows: Local<LastBlows>,
    mut kills: MessageReader<ArpgKill>,
    mut out: MessageWriter<KilledBy>,
) {
    let now = time.elapsed_secs_f64();
    for spawn in spawns.read() {
        if me.contains(spawn.anchor) {
            continue;
        }
        if let Some((ArpgText::Hit { school, .. } | ArpgText::Tick { school }, _)) = spawn.arpg {
            blows.0.insert(spawn.anchor, (school, now));
        }
    }
    for kill in kills.read() {
        let school = blows.0.get(&kill.unit).map_or(0, |&(s, _)| s);
        out.write(KilledBy {
            kill: *kill,
            school,
        });
    }
    blows.0.retain(|_, (_, at)| now - *at < BLOW_MEMORY_SECS);
}

/// A kill and its killing blow's damage type.
#[derive(Message, Clone, Copy)]
struct KilledBy {
    kill: ArpgKill,
    school: u8,
}

/// A heavy kill's burst in its type, and the streak.
#[allow(clippy::too_many_arguments)]
fn burst_kills(
    mut killed: MessageReader<KilledBy>,
    time: Res<Time>,
    spells: Option<Res<crate::ui_action::Spells>>,
    visuals: Option<Res<SpellVisuals>>,
    mut seq: ResMut<PlaySeq>,
    mut kits: MessageWriter<KitPush>,
    mut streak: ResMut<Streak>,
) {
    let now = time.elapsed_secs_f64();
    for KilledBy { kill, school } in killed.read() {
        if kill.overkill {
            if let (Some(spells), Some(visuals)) = (&spells, &visuals) {
                if let Some(kit_id) = burst_kit(spells, visuals, *school) {
                    kits.write(KitPush {
                        entity: kill.unit,
                        kit_id,
                        seq: seq.next(),
                    });
                }
            }
        }
        streak.count = if now - streak.last <= STREAK_WINDOW_SECS {
            streak.count + 1
        } else {
            1
        };
        if let Some(word) = milestone(streak.count) {
            streak.word = Some(word);
        }
        streak.last = now;
        streak.dirty = true;
    }
}

/// Tell the HUD of the streak.
fn push_streak(script: Option<NonSendMut<UiScript>>, mut streak: ResMut<Streak>) {
    let Some(script) = script else {
        return;
    };
    if !streak.dirty {
        return;
    }
    streak.dirty = false;
    let word = streak
        .word
        .take()
        .map_or("nil".to_string(), |w| format!("{w:?}"));
    let chunk = format!(
        "if ArpgHud_Streak then ArpgHud_Streak({}, {word}) end\n",
        streak.count
    );
    if let Err(e) = script.eval::<()>(&chunk) {
        warn!("arpg: the HUD refused the kill streak: {e}");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tags_map_to_schools() {
        assert_eq!(school_of_tag(0), Some(0));
        assert_eq!(school_of_tag(2), Some(2), "Fire");
        assert_eq!(school_of_tag(3), Some(4), "Frost");
        assert_eq!(school_of_tag(4), Some(6), "Arcane");
        assert_eq!(school_of_tag(5), Some(3), "Nature");
        assert_eq!(school_of_tag(6), Some(5), "Shadow");
        assert_eq!(school_of_tag(0xFF), None);
    }

    #[test]
    fn milestones() {
        assert_eq!(milestone(9), None);
        assert_eq!(milestone(10), Some("Rampage"));
        assert_eq!(milestone(100), Some("Annihilation"));
    }
}
