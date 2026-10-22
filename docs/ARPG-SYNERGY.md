# ARPG synergy

How skills, passives and items combine. The aim (Jeff): not designed builds, but possibilities, so
a character grows powerful through what it connects rather than through bigger numbers. Modelled
on Path of Exile 2's combos (one skill sets up a state, another pays it off) and its meta gems,
and on Last Epoch's skill trees (triggers, conversions, skills scaling off states). No charges.

## The parts

### 1. Enemy states

A small fixed vocabulary of things an enemy can be in. Any source can put an enemy in one, any
source can read it, and some can consume it. Skills never name each other: they meet through the
state. cmangos `Arpg/ArpgStates.{h,cpp}`.

| State | Comes from | Its own effect | Consumed for |
|---|---|---|---|
| Chilled | Frost hits (chance), Frostbolt | slowed | its end |
| Frozen | a critical Frost hit on a Chilled enemy | rooted | its end |
| Ignited | Fire hits (chance) | burns | the burn left |
| Bleeding | Physical hits (chance) | bleeds, more while it moves | the bleed left |
| Poisoned | Nature hits (chance) | poison stacks | the poison left |
| Withered | Shadow hits (chance) | takes more of your damage | its stacks |
| Overloaded | Arcane hits (chance) | stores damage, bursts | the stored damage |
| Judged | Holy hits (chance) | stores damage, bursts at 5 stacks | the stored damage |
| **Dazed** | **Daze build-up** (below) | slowed, its cast broken, takes 10% more damage, 4 sec | its end |
| Stunned | any stun | can't act | (read only) |
| Consecrated | standing in a Consecration | (read only) | (read only) |
| Scorched Ground | standing in a Flamestrike or Rain of Fire | (read only) | (read only) |
| Frozen Ground | standing in a Blizzard | (read only) | (read only) |
| Storm Ground | standing in a Hurricane | (read only) | (read only) |
| **Vulnerable** | a set-up spell landing: Faerie Fire, Curse of Weakness, Recklessness, the Elements or Shadow, Hammer of Justice, Frost Nova, Judgement of the Crusader, Frost Shock, Earthbind, and the free fillers (Frostbolt, Wrath, Searing Pain, Lightning Bolt) | takes 20% more damage from everything, 6 sec | (read only) |
| **Shocked** | the shaman's Lightning hits (Lightning Bolt, Chain Lightning, Stormstrike, Lightning Shield with Charged Shield), a stack each | takes 5% more damage from everything a stack, up to 4 (8 under Overcharge), 4 sec | Earth Shock: an extra hit of 25% of its own per stack |

**Vulnerable** (Diablo IV's, 2026-10-21) is the set-up every class has: the debuff or control
that opens a fight now pays, a fifth more damage from every source, the minions' included. It
multiplies with Dazed's 10%. The free fillers set it up too (2026-10-22), so a filler then the
spenders beats the spender alone: the play statistics showed one spell at about 90% of every
class's casts.

**Daze** is the physical and holy state. Every physical or holy hit builds it by twice its share of
the enemy's health (a hit for a tenth of its health builds a fifth); a Consecration's tick builds
8%. Champions, rares, elites and bosses build it slower (bosses at a quarter), and anything that
Dazes outright (Stagger, Aftershock) only builds it on them, so a boss is never stun-locked; nor
is a boss's, a rare's or a champion's cast broken. It ebbs when the hits stop for three seconds.
At 100% the enemy is Dazed. A hit pays off a state set up before it, never one it completes
itself (its triggers run before its own Daze). The hover bar shows each enemy's states
and its Daze as it builds (`SMSG_ARPG_STATES`, 0x353), and each enemy wears the look of the
states the server keeps to itself: Judged a Judgement's debuff, Ignited Immolate's, Bleeding
Rend's, Poisoned Deadly Poison's, Withered Shadow Word: Pain's, Overloaded Detect Magic's (Chill,
Freeze, Daze and stuns are real auras, drawn as they always were).

### 2. Triggers

"When X happens, do Y", as rows of data. cmangos `Arpg/ArpgSynergy.{h,cpp}`.

- **Events:** a hit, a critical hit, a kill, a state applied, a state consumed, a block, a dodge
  (the enemy is the one whose blow it was).
- **Conditions:** the spell (by name), the skill, the hit's tags, a state the enemy must be in, the
  state the event is about.
- **Effects:** *apply* a state; *strike* (consume a state for an extra hit: a share of the hit plus
  what the state held); *burst* (consume a state for a hit on every enemy within 8 yards);
  *spread* a state to nearby enemies; *daze* (build Daze); *nova* (a share of the hit on every
  other enemy within 8 yards, consuming nothing: a ground's payoff, as grounds are read, not
  spent).
- **Owners:** a class (its own rules), a skill-tree node (its rank scales the effect), or a worn
  unique (by name). Each row has its own cooldown. Trigger damage never sets off another hit's
  triggers; state events nest at most three deep.

### 3. Modifiers by state and tag, not by skill

New affixes, so an item opens a pairing instead of adding a number:
- +% increased damage against Dazed, Judged, Chilled, Ignited, Bleeding, Poisoned or Stunned
  enemies, or enemies on Consecrated ground;
- +% increased Daze build-up.

They roll from a new affix pool: items rolled before keep their affixes (each pool is frozen at its
size, as an item re-rolls from its seed).

They join the existing tag modifiers ("+% Holy damage", "+% Area damage").

### 4. Conversions

A skill-tree node that makes a skill count as another damage type, for its ailments, its Daze and
the triggers: the skill moves into that type's pairings.

### 5. Flat damage cut

Added damage affixes were eight times vanilla's size (a fifth of the hit) and carried builds by
numbers. They are three times now (about a fifteenth).

## The paladin, first

**Its own rules** (every paladin): Daze is its currency, built by its swings, Seals (holy) and
Consecration's ticks.
- *Shattering Blow:* Hammer of Wrath consumes Daze for an extra hit of 100% of the hit.
- *Verdict:* Judgement consumes Daze to burst onto every enemy within 8 yards for 60% of the hit.
- *Sentence* and *Sentence Passed* count a Dazed enemy as stunned. *Stagger* (passive web) Dazes.

**New tree nodes:**

| Skill | Node | What it opens |
|---|---|---|
| Crusader | Concussive Blows (3 ranks) | critical physical or holy Crusader hits add 15% of the way to a Daze per rank |
| Crusader | Sun Seal (conversion) | swings and Seals count as Fire: they Ignite instead of Judge, and build no Daze |
| Judgement | Frozen Verdict (conversion) | Judgement counts as Frost: it Chills, and a critical one on a Chilled enemy Freezes |
| Hammers | Aftershock | consuming Daze Dazes the two nearest enemies (every 2 sec) |
| Wrath | Cleansing Fire | Exorcism and Holy Wrath consume Ignite to burst for 80% of the hit and the burn left |

Some pairings these make, none of them designed as a build:
- **Daze loop:** swings and Consecration build Daze, Hammer of Wrath cashes it in, Aftershock
  passes it on to the next two, and items against Dazed enemies scale all of it.
- **Sun paladin:** Sun Seal makes the swings ignite, Cleansing Fire bursts the burns with Exorcism,
  and "against Ignited" items scale it.
- **Frost judge:** Frozen Verdict chills, a frost mage's items and Shatter-style payoffs apply.

## The mage

**Its own rules:**
- *Shatter Point:* Fire Blast consumes Freeze for an extra hit of 120%.
- *Detonating Spark:* Arcane Explosion consumes Ignite to burst for 50% and the burn left.
- *Frost Step:* dodging a blow Chills its attacker.
- *Kindling Ground:* Fire hits Ignite an enemy standing in a Flamestrike (once a second).

**New tree nodes:**

| Skill | Node | What it opens |
|---|---|---|
| Fireball | Frostfire Bolt (conversion) | Fireball and Pyroblast count as Frost |
| Frostbolt | Cold Snap | consuming a Freeze Chills the 2 nearest enemies |
| Arcane Missiles | Arcane Detonation | Arcane Missiles consume Overload to burst |
| Arcane Explosion | Glacial Explosion (conversion) | Arcane Explosion counts as Frost |
| Storms | Rime Field | Frost hits Freeze an enemy standing in a Blizzard |
| Fire Blast | Combustion Point | a critical Fire Blast on an Ignited enemy spreads Ignite to 2 |

## The druid

**Its own rules:**
- *Feeding Bite:* Ferocious Bite consumes Bleeding for an extra hit of 100% and the bleed left.
- *Blooming Star:* Starfire consumes Poison to burst for 60% and the poison left.
- *Eye of the Storm:* Wrath on an enemy in a Hurricane also strikes every enemy within 8 yards of
  it for 40% (once a second). The Hurricane is the druid's ground (Storm Ground).
- Bear and cat hits are physical, so they build Daze like the paladin's.

**New tree nodes:**

| Skill | Node | What it opens |
|---|---|---|
| Wrath | Thorns of Cenarius | a critical Wrath on a Poisoned enemy spreads Poison to 2 |
| Moonfire | Solar Flare (conversion) | Moonfire's hit counts as Fire |
| Bear | Crushing Maul | Maul consumes Daze for an extra hit of 80% |
| Cat | Blood Frenzy | consuming Bleeding spreads it to 2, at the size it had |

## The warlock

**Its own rules:**
- *Feast of Souls:* Death Coil consumes Withered for an extra hit of 100% and the stacks.
- *Contagion:* a Withered enemy's death Withers the 2 nearest.
- *Fel Bond:* a minion's kill of a Withered enemy heals the warlock 2% of maximum health.
- Its minions' hits are its own for the states (`ARPG-CHARACTER.md`, "Minions"): the Imp
  Ignites, the Succubus Withers, the Felhunter Overloads, and the Voidwalker builds Daze.

**New tree nodes:**

| Skill | Node | What it opens |
|---|---|---|
| Shadow Bolt | Shadow Trance | Shadow Bolt consumes Withered for 100% of the hit and the stacks |
| Immolate | Conflagration | Conflagrate consumes Ignite to burst within 8 yards |

## The paladin's block

*Retribution:* blocking a blow takes its attacker a quarter of the way to a Daze.

## Uniques

A worn unique can own a trigger row (its tooltip shows it):

| Unique | What it opens |
|---|---|
| Kresh's Back (Wailing Caverns) | Hammer of Justice also Dazes the 3 nearest enemies (on its landing: a stun deals no damage) |
| Hammer of the Grand Crusader (Stratholme) | Hammer of Wrath consumes Judged to burst for the hit and the stored damage |
| Spinal Reaper (Molten Core) | a critical Seal of Command takes its target 30% of the way to a Daze |
| Azuresong Mageblade (Molten Core) | a critical Frostbolt on a Chilled enemy Freezes the 2 nearest |
| Staff of Dominance (Molten Core) | Fireball consumes Chill for an extra hit of 80% |
| Wolfshead Helm | Shred on a Bleeding enemy takes it 20% of the way to a Daze |
| Smite's Mighty Hammer (Deadmines) | Seal of Righteousness Judges a Dazed enemy |
| Taskmaster Axe (Deadmines) | Judgement consumes Bleeding for an extra hit of 100% and the bleed left |
| Mograine's Might (Scarlet Monastery) | Judgement on a stunned enemy Dazes it |
| Hand of Righteousness (Scarlet Monastery) | a critical Seal of Righteousness on a Judged enemy Judges the 2 nearest |
| Emberstone Staff (Deadmines) | a critical Fireball on an Ignited enemy Ignites the 2 nearest |
| Illusionary Rod (Scarlet Monastery) | Arcane Missiles Overload a Chilled enemy |
| Rod of the Sleepwalker (Blackfathom Deeps) | Wrath consumes Poison for an extra hit of 80% and the poison left |
| Ravager (Scarlet Monastery) | a critical swing on a Bleeding enemy spreads Bleeding to 2 |
| Mirah's Song (Scarlet Monastery) | a critical swing on a Dazed enemy Judges it |
| Aegis of the Scarlet Commander (Scarlet Monastery) | blocking takes the attacker 35% of the way to a Daze |
| Hypnotic Blade (Scarlet Monastery) | Arcane Explosion Chills an Overloaded enemy |
| The Rockpounder (Uldaman) | swings consume Daze to burst for 80% of the hit |
| Galgann's Firehammer (Uldaman) | swings Ignite a Dazed enemy (the bridge from Daze to the Sun paladin) |
| Archaedic Stone (Uldaman) | Flamestrike consumes Chill to burst for 60% |
| Gahz'rilla Fang (Zul'Farrak) | a Frozen enemy's death Freezes the 2 nearest |
| Witch Doctor's Cane (Zul'Farrak) | Moonfire on a Poisoned enemy spreads Poison to 2 |
| Rotgrip Mantle (Maraudon) | Nature and Arcane hits Poison an enemy in a Hurricane |
| Ironfoe (Blackrock Depths) | a critical swing consumes Judged for 100% of the hit and the stored damage |

The swing rows fit any class that swings. A row whose item name is not in the item templates
never fires, and the server log says so at the first tooltip request.

**Generated unique powers** (every unique without an authored power has one, `ARPG-UNIQUES.md`)
feed on their element's state: a damaging power deals 50% more to an enemy already in it (fire
on Ignited, frost on Chilled, holy on Judged, physical on Bleeding, nature on Poisoned, arcane on
Overloaded, shadow on Withered), and its tooltip says so. The powers themselves are dealt as
before, so no unique's power changed.

## Kills

A kill reads the fallen enemy's ailments (its auras are gone with it):
- *Contagious Verdict* (paladin): a Judged enemy's death Judges the 2 nearest.
- *Pyre* (mage): an Ignited enemy's death Ignites the 2 nearest.
- *Plague Bearer* (druid): a Poisoned enemy's death Poisons the 2 nearest.

A spread on a kill is sized as a hit of a tenth of the fallen's health.

## Next

- The shaman's synergy rows (it has its class rules: Shock, Static Discharge, Elemental
  Crossing, `ARPG-CHARACTER.md`), and the totems' grounds.
- Unique rows for Dire Maul, Scholomance, Stratholme and the raids as the later acts come in.
