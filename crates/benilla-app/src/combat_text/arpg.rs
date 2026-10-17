//! Fork-only, not 1.12.1: the ARPG view's numbers. Under [`ArpgNumbers`] a spawn that says what it
//! is ([`ArpgText`]) draws in its damage type's colour, a critical strike bigger and outlined,
//! a damage over time tick smaller and dimmer (and merged with the same type's tick on the unit
//! just before it), a proc's burst as a labelled number, and the player's own wounds in red
//! over them. Big numbers take thousands separators. The stock look stays for every other spawn
//! and for the stock client.

use bevy::prelude::*;

use benilla_ui::script::Outline;

/// Present only while the ARPG view is on: the numbers take [`ArpgText`]'s look.
#[derive(Resource)]
pub(crate) struct ArpgNumbers;

/// What an ARPG number is. `school` is the spell school index (0 Physical, 1 Holy, 2 Fire,
/// 3 Nature, 4 Frost, 5 Shadow, 6 Arcane), as the combat log sends it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ArpgText {
    /// A hit dealt, critical or not.
    Hit { school: u8, crit: bool },
    /// A damage over time tick dealt.
    Tick { school: u8 },
    /// A hit one of the player's minions dealt (its pet, a guardian, the horde): quieter than the
    /// player's own, so a minion build's numbers read as the minions'.
    MinionHit { school: u8, crit: bool },
    /// Damage the player took, over the player.
    Taken,
    /// A proc's named burst ("Overload 4,210").
    Label { school: u8 },
}

/// How long a tick waits for the next one of its type on the unit to merge into it.
pub(super) const MERGE_SECS: f64 = 0.6;
/// The per-unit cap under the ARPG look (the stock client's is 4): a pack's numbers overlap more.
pub(super) const MAX_PER_UNIT: usize = 6;

/// The damage types' colours, ARGB, by school index.
const SCHOOL_ARGB: [u32; 7] = [
    0xFFFF_FFFF, // Physical
    0xFFFF_E98A, // Holy
    0xFFFF_8A2E, // Fire
    0xFF7C_E85A, // Nature
    0xFF8C_D8FF, // Frost
    0xFFB8_7CFF, // Shadow
    0xFFFF_8CF4, // Arcane
];
/// A label of no damage type: the proc gold.
const PROC_GOLD_ARGB: u32 = 0xFFFF_D200;
/// A physical critical strike, which would otherwise be white like the rest.
const PHYSICAL_CRIT_ARGB: u32 = 0xFFFF_D040;
/// The player's wounds.
const TAKEN_ARGB: u32 = 0xFFFF_3A3A;

/// The look of one ARPG number: colour, size against the stock number, whether it pops as a
/// critical strike does, alpha, and its outline.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct Look {
    pub(super) color: u32,
    pub(super) size: f32,
    pub(super) pops: bool,
    pub(super) alpha: f32,
    pub(super) outline: Outline,
}

fn school_argb(school: u8) -> u32 {
    SCHOOL_ARGB
        .get(usize::from(school))
        .copied()
        .unwrap_or(PROC_GOLD_ARGB)
}

pub(super) fn look(text: ArpgText) -> Look {
    match text {
        ArpgText::Hit {
            school,
            crit: false,
        } => Look {
            color: school_argb(school),
            size: 1.0,
            pops: false,
            alpha: 1.0,
            outline: Outline::Normal,
        },
        ArpgText::Hit { school, crit: true } => Look {
            color: if school == 0 {
                PHYSICAL_CRIT_ARGB
            } else {
                school_argb(school)
            },
            size: 1.2,
            pops: true,
            alpha: 1.0,
            outline: Outline::Thick,
        },
        ArpgText::MinionHit { school, crit } => Look {
            color: school_argb(school),
            size: if crit { 0.95 } else { 0.8 },
            pops: false,
            alpha: 0.75,
            outline: Outline::Normal,
        },
        ArpgText::Tick { school } => Look {
            color: school_argb(school),
            size: 0.72,
            pops: false,
            alpha: 0.8,
            outline: Outline::Normal,
        },
        ArpgText::Taken => Look {
            color: TAKEN_ARGB,
            size: 0.85,
            pops: false,
            alpha: 1.0,
            outline: Outline::Normal,
        },
        ArpgText::Label { school } => Look {
            color: school_argb(school),
            size: 1.3,
            pops: true,
            alpha: 1.0,
            outline: Outline::Thick,
        },
    }
}

/// `n` with thousands separators: `4210` to `"4,210"`.
pub(super) fn grouped(n: u32) -> String {
    let digits = n.to_string();
    let mut out = String::with_capacity(digits.len() + digits.len() / 3);
    for (i, c) in digits.chars().enumerate() {
        if i > 0 && (digits.len() - i).is_multiple_of(3) {
            out.push(',');
        }
        out.push(c);
    }
    out
}

/// The body an ARPG number shows for `value` (a label leads with its name).
pub(super) fn body(text: ArpgText, label: Option<&str>, value: u32) -> String {
    match (text, label) {
        (ArpgText::Label { .. }, Some(name)) if value > 0 => format!("{name} {}", grouped(value)),
        (ArpgText::Label { .. }, Some(name)) => name.to_string(),
        (ArpgText::Taken, _) => format!("-{}", grouped(value)),
        _ => grouped(value),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_minions_hit_is_quieter_than_the_players() {
        let mine = look(ArpgText::Hit {
            school: 2,
            crit: true,
        });
        let minion = look(ArpgText::MinionHit {
            school: 2,
            crit: true,
        });
        assert_eq!(minion.color, mine.color);
        assert!(minion.size < mine.size && minion.alpha < mine.alpha && !minion.pops);
    }

    #[test]
    fn big_numbers_take_separators() {
        assert_eq!(grouped(0), "0");
        assert_eq!(grouped(999), "999");
        assert_eq!(grouped(1000), "1,000");
        assert_eq!(grouped(4210), "4,210");
        assert_eq!(grouped(1_234_567), "1,234,567");
    }

    #[test]
    fn crits_pop_bigger_and_ticks_are_smaller() {
        let hit = look(ArpgText::Hit {
            school: 2,
            crit: false,
        });
        let crit = look(ArpgText::Hit {
            school: 2,
            crit: true,
        });
        let tick = look(ArpgText::Tick { school: 2 });
        assert!(crit.size > hit.size && crit.pops && !hit.pops);
        assert!(tick.size < hit.size && tick.alpha < 1.0);
        assert_eq!(hit.color, crit.color, "a fire crit stays fire-coloured");
        assert_ne!(
            look(ArpgText::Hit {
                school: 0,
                crit: true
            })
            .color,
            look(ArpgText::Hit {
                school: 0,
                crit: false
            })
            .color,
            "a physical crit turns gold"
        );
    }

    #[test]
    fn bodies() {
        let label = ArpgText::Label { school: 6 };
        assert_eq!(body(label, Some("Overload"), 4210), "Overload 4,210");
        assert_eq!(body(label, Some("Freeze"), 0), "Freeze");
        assert_eq!(body(ArpgText::Taken, None, 1200), "-1,200");
        assert_eq!(body(ArpgText::Tick { school: 2 }, None, 75), "75");
    }
}
