# ARPG roadmap: resources and difficulty

Where mana, rage and energy and the game's difficulty go next, from how other ARPGs do it and from
our own play statistics (`ARPG-HANDOFF.md`, "Play statistics"). Each phase is small enough to play
and measure before the next.

## What the stats say now

From the first sessions (a level 17 mage in the Deadmines, a level 25-26 paladin in Shadowfang
Keep):

- **No pressure.** 0 deaths across 6 bosses and 81 elites; under 30% health for 6 of 711 combat
  seconds. Vanilla dungeons are tuned for five players without flasks, globes or a roll.
- **Mana never matters.** 0 seconds under 15% mana, no cast ever refused.
- **One button.** Fireball is ~100% of the mage's damage; the procs beside it deal next to
  nothing (Frostbolt ~8 a hit, Smite ~10, against Fireball's ~316).

## The rule that matters most: never stuck

The designer's constraint is that a player is never left unable to act. Every successful ARPG
meets that the same way, and we adopt all three:

1. **Overdraw** (Last Epoch): a skill can be cast whenever the resource is above 0, even if it
   costs more than is left. Running low slows you down; it never locks you out.
2. **A free filler for every class** (PoE2's default attack, Diablo 3's signature spells,
   Diablo 4's basic skills): costs nothing, hits reliably, restores a little resource on hit,
   and does something of its own (a slow, a debuff) so pressing it is a choice.
3. **Mana is a % of the pool**: costs and regeneration as shares of max mana, so the pressure is
   the same at level 10 and level 60, and gear that adds regen or cuts cost has an obvious effect.

The 1.12 "five second rule" (no mana regen for 5 s after a cast) goes: it punishes casting, which
is the whole game.

## Phase 1: the resource model

**Built** (cmangos `Arpg/ArpgResources.{h,cpp}`, client `spell/usable.rs` and `spell/mods.rs`):
overdraw with at least half the cost left (rather than any mana at all, so Frostbolt's refund
cannot buy a Pyroblast), mana costs as a share of the pool by the tables below, the fillers'
refunds (Frostbolt and Wrath 3% once a cast, a sealed paladin swing 2%), free shapeshifting,
regeneration of 1.5% a second with no five-second rule, globes giving 8% mana, and
`SMSG_ARPG_COSTS` (0x350) so the client shows and checks the same costs. The play statistics count
overdraws and read low mana at 20% and low health at 35%.

**Not yet:** the mana flask, the orb's low-mana pulse and sound, a cast at nothing left trying the
filler, bear rage on entering the form, the cat's free fallback, the cost and mana-on-hit affixes,
Chill.

**All classes**

- Overdraw for mana and rage. Cat energy keeps its hard cap (it refills fast).
- Mana regen 1.5% of max a second in combat, 5% out of it.
- Globes give 8% mana (10% from an elite); the mana flask 40% over 3 s, its charges from kills
  weighted by the foe (normal 1, champion 2, rare 5, boss 20, after PoE's "Power").
- New affixes: "mana on hit", "skill cost -X%" (capped at 25%).
- Out-of-mana feedback: the orb pulses and sounds at 25% and 10%; a cast at 0 tries the class
  filler instead and is still logged (`refusedForPower` should become rare but not zero).

**Mage**

| Skill | Role | Cost | Notes |
|---|---|---|---|
| Frostbolt | Free filler | 0 | Restores 3% a hit; slows; stacks Chill |
| Fireball | Main spender | 9% | Consumes Chill for bonus damage |
| Arcane Missiles | Expensive burst | 18% per channel | Best damage per second |
| Blizzard | Area | 25% | Consumes Chill |
| Fire Blast | Cooldown | 0, 6 s | Reset by a Fireball crit (already a synergy) |
| Frost Nova | Defence | 0, 20 s | |

Fireball spam then drains a full pool in about 8-10 s; mixing in Frostbolt keeps it going and
adds damage through Chill. That answers "I'm only incentivised to cast Fireball".

**Paladin**

| Skill | Role | Cost | Notes |
|---|---|---|---|
| Melee with a Seal | Free generator | 0 | Each swing restores 2% |
| Seals | Stance | 6% to switch | Last until changed |
| Judgement | Cooldown | 0, 8 s | Refunds 6% |
| Consecration | Area | 22% | |
| Hammer of Wrath | Execute | 10%, 6 s | Refunded on a kill |

**Druid**

- Shapeshifting is free (1.5 s cooldown): in vanilla it costs mana, a trap that strands you in
  the wrong form.
- Bear: rage from dealing and taking damage, decaying out of combat; entering the form grants 20.
  Auto-attack is the free generator; Maul 15, Swipe 20.
- Cat: energy 100, regen as 1.12; Claw 35 builds combo points; each finisher refunds 5 energy a
  point; a free fallback swipe so cat is never idle.
- Caster: Wrath is the free filler (3% a hit); Moonfire 8%, Starfire 15%, Healing Touch 20%.

## Phase 2: difficulty

- **Monster damage by area level**, compounding about 8% a level, then rarity multipliers
  (champion x1.5, rare x2.5, boss x4) and a solo multiplier for dungeons built for five.
- **Tune by time to kill the player**: face-tanking a trash pack should kill in 6-10 s; a boss's
  telegraphed big hit should take 30-50% of health, and a roll avoids it entirely.
- Change one multiplier per round (damage first, then density, then costs) and read the stats
  between rounds.

## Phase 3: skill variety

- Make the procs and secondary hits scale with spell power so they matter (today they are
  near-zero).
- More synergies like Chill: one skill sets up, another pays off.
- Track each skill's share of damage; none should carry more than about 60% of a build.

## Targets to read from the play statistics

Proposed, not industry numbers (no studio publishes these); for a fresh character on normal.

| Metric | From | Target |
|---|---|---|
| Deaths per hour, open world | `death` lines / session minutes | 0.3-1.0 |
| Deaths per hour, dungeons | as above, by zone | 0.5-1.5 |
| Combat time under 35% health | fight `minHealthPct`, session `lowHealthSec` | 5-12% |
| Combat time under 20% mana | session `lowManaSec` | 10-25% |
| Casts tried at 0 resource | `refusedForPower` | 1-3 per 10 min |
| Flasks | `flasks` | 3-8 per 10 min |
| Top skill's share of damage | `dealt` | under 60% |
| Free filler's share of casts | `dealt` casts | 30-50% |

The stats need two additions for this: the low-health threshold at 35%, and a low-mana count at
20% (today 30% and 15%).

## Sources

- Path of Exile 2: mana and regeneration (4% of max a second), the free default attack, Spirit,
  leech, flask charges by monster power: poe2wiki.net (Mana, Mana_regeneration, Leech, Power),
  maxroll.gg/poe2/guides/skills-in-path-of-exile-2, mobalytics.gg/poe-2/guides/spirit.
- Diablo 3: signature spells, generators and spenders, Fury from taking damage:
  diablowiki.net/Resource, diablo.fandom.com/wiki/Fury, diablo.fandom.com/wiki/Electrocute.
- Diablo 4: basic, core and ultimate skills, resource generation, refunds on cooldowns:
  maxroll.gg/d4/getting-started/stats-for-beginners, maxroll.gg/d4/resources/cooldown-reduction.
- Last Epoch: casting above 0 mana, flat regen, Focus: support.lastepoch.com (Mana and Mana
  Regeneration), icy-veins.com/last-epoch/mage-overview.
- Grim Dawn: default attack replacers with a small cost: forums.crateentertainment.com/t/49376.
- Path of Exile monster levels and damage scaling: pathofexile.fandom.com/wiki/Monster_level.
