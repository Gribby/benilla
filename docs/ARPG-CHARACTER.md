# ARPG character design: skills, passives, attributes and items

Design, not built; Jeff's decisions are recorded where they apply. This replaces the first skill tree (`ARPG-SKILL-TREES.md`, a vanilla talent
grid with ARPG nodes mixed in), after Jeff's review of it in game: it didn't look or play like an
ARPG tree. This doc rethinks how the character systems fit together, then plans the paladin in
full.

Read `ARPG-UNIQUES.md` first: its mechanic kit (chain, arc, burst, pierce…) is one of the
building blocks here.

## What was wrong with the first tree

- **It was a talent grid.** Three columns, tiers, no paths. ARPG trees are about routes: you see
  where you want to end up and plan a path there, and the nodes on the way are part of the choice.
- **Most nodes were vanilla talents.** "+1% crit per rank" five times is the least ARPG thing
  there is.
- **It didn't talk to anything.** Items gave stats, the tree gave stats, skills did what they
  always did. In an ARPG the fun is in the overlaps: an item that makes a tree node better, a
  stat that makes a skill change shape, a skill node that only shines with a certain item.
- **Skills had no depth.** A Judgement build and a Consecration build were the same character
  with different talents ticked.

## The four pillars

1. **Build around skills.** A character is defined by the five skills it specialises in, and
   each of those skills has its own tree that changes how it plays. This is the Last Epoch model,
   and it fits vanilla well: vanilla classes already have the spells, they just lack depth.
2. **One shared language: tags.** Every skill, node and item affix speaks in the same tags
   (Holy, Area, Seal, Melee…). "+20% Area damage" on a ring means something to Consecration and
   Holy Wrath and nothing to Judgement, so items and nodes pick the skills they help without a
   special case each.
3. **Attributes mean something in ARPG terms.** Strength, Agility, Stamina, Intellect and Spirit
   keep what vanilla gives them and gain one ARPG effect each (bigger areas, faster movement,
   shorter cooldowns…). Gearing for Spirit then becomes a build decision, not a healer stat.
4. **Items feed the trees.** Affixes can add levels to a skill (more points in its tree), boost a
   tag, or raise an attribute that a skill node scales with. Uniques transform skills outright.

## How the systems connect

```mermaid
flowchart LR
  subgraph Character
    L[Level] -->|1 point per 2 levels| W[Passive web]
    L -->|1 skill point per level| ST[Skill trees x5]
  end
  I[Items] -->|+skill levels| ST
  D[Boss and champion drops] -->|Codex pages: capstones| ST
  D -->|runes: one socket per tree| ST
  I -->|attributes| A[Attributes]
  I -->|tag affixes| T((Tags))
  I -->|uniques: transform a skill| S[Skills]
  W -->|small nodes| A
  W -->|notables: tag bonuses| T
  W -->|keystones: rule changes| S
  A -->|ARPG secondary effects| S
  T -->|scale every skill carrying the tag| S
  ST -->|reshape one skill| S
```

Five layers, one direction of flow: **everything ends up changing what a skill does.** That's
the test for any new node or affix: which skills does it change, and how would a player notice?

## Tags

Every class spell gets tags in server data. Nodes and affixes reference tags, never spells,
unless they belong to one skill's own tree.

| Group | Tags |
|---|---|
| Damage type | Physical, Holy, Fire, Frost, Arcane, Nature, Shadow |
| Delivery | Melee, Spell, Projectile, Area, Ground (lingering zones), Channel, Duration (DoTs, buffs) |
| Role | Heal, Control (stuns, roots, sheep), Movement, Minion, Aura, Seal, Curse, Shout, Totem |

Paladin examples:

| Skill | Tags |
|---|---|
| Strike (the held swing) | Melee, Physical |
| Seals | Seal, Holy, Melee |
| Judgement | Spell, Holy |
| Consecration | Spell, Holy, Area, Ground, Duration |
| Hammer of Justice | Spell, Control, Projectile (once its tree makes it one) |
| Holy Light / Flash of Light | Spell, Holy, Heal |
| Auras | Aura, Holy |

A tag bonus is a damage or healing multiplier the server applies when a spell carrying the tag
lands: one hook in the spell damage and healing bonus path, summing the player's bonuses for
the spell's tags.

## No hit

There is no hit stat. An ARPG player's swings, shots and spells, and those of its pets and
totems, never miss: the melee and ranged miss roll, the spell miss roll and the hit share of the
magic resist roll are all zero. Dodge, parry, block and resistances still apply. Vanilla's +hit
items do nothing, and no affix gives hit.

## Damage

**Built:** `Arpg/ArpgDamage.{h,cpp}` and `DamagePartsFor` in `Arpg/ArpgCharacter.cpp` on the server.
The first damage model added every bonus into one percentage, so each upgrade was the same small
bump, worth less the more you had. A hit is now built in layers, as Path of Exile and Last Epoch
build theirs, and the numbers are ARPG-sized.

| Layer | What it is | Where it comes from |
|---|---|---|
| **Base** | Vanilla's: the weapon's swing or the spell's points, with attack power or spell damage by vanilla's coefficients | Gear's vanilla stats |
| **× Size** | The power scale: 1 + 0.1 per level past the first (×6.9 at 60). Every creature's health grows to match, so the size of the numbers changes, not the fight | `Arpg.Power.PerLevel` |
| **× Increased** | One sum of every "increased" bonus that fits the hit's tags | The web's small nodes, item affixes, Strength, Intellect, Spirit, Holy Weapons |
| **× More** | Each "more" bonus multiplies on its own | Notables, keystones, skill tree nodes, the uniques' readied procs |
| **× Critical** | Vanilla's multiplier (2 for attacks, 1.5 for spells) plus Agility's and gear's | Agility, "+% critical strike multiplier", "+% spell critical strike damage" |

So "+12% Holy damage" on a ring joins a pool that may already hold 150%, while Radiance's "8% more
Judgement damage" per rank multiplies everything else. Specialising a skill (its tree's more
nodes) is a build's big multiplier; gear's increased bonuses are its breadth.

The size applies only against creatures: a duel stays vanilla's. Everything that turns a flat
number into damage grows with it (thorns, Martyr, Martyr's Ward, Holy Hammer's and Blessed
Hammer's attack-power hits, the Lightforged and Dawnbringer bolts made from a heal, damage shields
such as Retribution Aura, spells immune to caster modifiers), and everything that turns damage
into health or mana shrinks back (leech, Rune of Leech, Light of the Crusader, a thorned
champion's recoil).

**Creature health:** vanilla's × the size at the creature's level × (1 + 0.5 × level ÷ 60), about
×10 at 60; the extra half is for the bonuses a character builds up. A raid's creatures grow again
with the gear the raid drops: × (1 + (raid item level − 60) ÷ 20), so Molten Core ×1.3, Blackwing
Lair ×1.7, Temple of Ahn'Qiraj ×2, Naxxramas ×2.4, on top of the solo scaling and the tier. Both
are `mangosd.conf` settings (`Arpg.Power.PerLevel`, `Arpg.Power.HealthGrowth`). The ARPG world
hosts ARPG players only: a stock client there would face the health without the damage.

### Added damage

"Adds 70 to 164 Fire damage to attacks" (weapons, gloves and jewellery). Each damage type the
gear adds lands as **its own hit** beside the main one. It hits in its own school with its own
look (Fireball's impact for Fire, Smite's for Holy...), and takes its own type's increased
bonuses with the main hit's delivery (Melee, Spell, Area) and the skill's more bonuses. It is
critical when the main hit was, and it can inflict its own ailment. Swings take the attack
affixes, spells the spell ones; a Seal's proc takes none, as the swing it rides on has them
already; at most the two biggest types land beside one hit. The
affix shows and deals its numbers at the ARPG's size for the item's level, three times vanilla's
stat growth, so an affix lands near a fifteenth of the hit it rides on (at vanilla's size it was
about 2%, noise beside a skill). It was eight times, a fifth of the hit, and carried builds by
brute numbers; Jeff wants power from synergy instead (`ARPG-SYNERGY.md`). An off-type affix is
still worth having for what its type opens: a Holy paladin with Fire added to attacks ignites.

### Ailments

Each damage type has one. **A critical strike always inflicts its type's ailment**, and "% chance
to Ignite" (and the others) adds a chance on any hit. Ailment damage grows with "increased damage
with ailments", "damage over time" and Spirit, on top of the hit it came from.

| Type | Ailment | Effect |
|---|---|---|
| Physical | **Bleed** | 60% of the hit over 5 sec, doubled while the target moves; the strongest counts; armour reduces its ticks |
| Fire | **Ignite** | 80% of the hit over 4 sec; the strongest counts |
| Nature | **Poison** | 30% of the hit over 3 sec; stacks (20 at once), their ticks landing as one hit |
| Frost | **Chill** | Slowed (Improved Blizzard's chill); a critical hit on a chilled target **freezes** it (Frostbite) |
| Shadow | **Wither** | It takes 4% more damage from you per stack, 10 at most, for 4 sec after the last |
| Arcane | **Overload** | Half the hit is stored; 3 sec after the first, it bursts on the target and those within 6 yd |
| Holy | **Judged** | A quarter of the hit is stored per stack; at 5 stacks a Holy blast (Holy Wrath's) strikes the target and those within 6 yd for all of it |

Judged is the paladin's: every critical Seal, Judgement, Exorcism or Holy Wrath judges, and a
pack that takes five goes up in light. Burst radii grow with spell area.

### Showing it

The Skills tab's Bonuses button lists, per skill: the size, the increased sum, the more product,
each bonus that applies now and where it comes from, the critical multiplier, the added damage,
and the ailments with their chance and bonus. The estimate and the last minute's hits follow.

**Comparisons** (`Arpg/ArpgCompare.{h,cpp}`, `SMSG_ARPG_COMPARE` 0x34C): every wearable item in
the bags says, under its affixes, what it would do to each slotted skill's damage if worn in
place of what is there ("Worn in place of Band of Flesh: Judgement +12% damage, Strike −3%
damage"). Each skill gets a score, its expected damage a use (Strike: a second), from the same
pieces as a hit: attack power and the weapon or the spell's points and spell damage, the
increased sum, the more product, critical strike chance and multiplier, added damage, and the
ailments it should inflict. The score is worked out with the gear worn and with the item swapped
in. The item's vanilla stats count (its own, its equip spells' and its "of the Bear" suffix's),
and so do its affixes. It is an estimate: Agility's and Intellect's vanilla critical strike chance,
set bonuses and the target's armour are left out. The server sends the whole set once a second
when the gear, the bags or the character's numbers changed; the client shows the lines after the
item's affixes.

## Attributes

The five vanilla attributes keep their vanilla effects, so every vanilla item still works as
designed. Each gains one ARPG effect, at rates that make a dedicated gearer notice without
making the stat mandatory:

| Attribute | Vanilla (unchanged) | ARPG effect | Rate (proposed) | Cap |
|---|---|---|---|---|
| **Strength** | Attack power, block | Melee area: bigger arcs, cleaves and shockwaves; increased Melee and Physical damage | +1% size per 10 Str; +1% damage per 5 | +40% size |
| **Agility** | Crit, armour, dodge, ranged AP | Movement speed (over a base +10%); critical strike multiplier | +1% per 20 Agi; +1% multiplier per 5 | +30% speed with the base |
| **Stamina** | Health | Life on kill | Stamina ÷ 4 per kill | none |
| **Intellect** | Mana, spell crit | Spell area: bigger Consecration, Holy Wrath, Blizzard; increased Spell damage | +1% radius per 10 Int; +1% damage per 5 | +40% radius |
| **Spirit** | Regeneration | Cooldown recovery; increased damage over time and with ailments | +1% per 10 Spirit; +1% damage per 5 | +30% cooldown |

Pacing (built, `Arpg/ArpgCharacter.cpp`): mana returns from fighting, slowly enough that spells
and drains show (0.3% of the maximum per swing that lands, 0.5% for a skill hit at most once a
second, 1% per kill, and each swing and skill hit adds the mana on
hit of the web's Bloodthirst and Devotion nodes and the "+N mana on hit" item affix), and five seconds out of a fight (out of combat, no hit dealt or
taken) health flows back at 5% a second and mana at 2%, so nobody sits down to drink, but a run
between packs is not a refill.

Other ARPG stats exist only on items and tree nodes, never as base attributes: area size,
cooldown recovery, movement speed, life on kill, mana on hit, damage per tag, extra projectiles,
extra chain jumps, and skill levels.

**Why these five:** each is something ARPG builds chase, each maps to a vanilla stat's flavour
(Strength is big swings, Intellect is big spells, Spirit is staying in the fight), and none
replaces a vanilla effect. The rates are a starting point, to tune in play.

### Casting: no cast times, a recovery instead

Every class spell casts at once, on the move (`Arpg::CastsAtOnce`, `Spell::Prepare`): what was
its cast time is now its recovery, the global cooldown it starts, so you are never stuck rooted
in a cast bar but a big spell still costs its time. The recovery is half the vanilla cast time
(Fireball's 3.5 sec is 1.75), and at least the global cooldown, a third of vanilla's (0.5 sec).
Cooldown reduction cuts both, and cast speed the recovery too; they multiply, so no stack of
them reaches zero (`SetPacing`). The cast time nodes ("-0.15 sec Fireball recovery") and talents
shorten the recovery, and the "next spell instant" procs now let it skip its recovery (only
the global cooldown is left). Channels (Blizzard, Arcane Missiles), and anything that took over
6 sec to cast (resurrections, teleports, portals), cast as before, and so do the generic-family
spells a class uses (Holy Wrath; Exorcism is instant anyway), which the client cannot tell apart. The client mirrors the rule
(`Spells::arpg_recovery_ms`): no moving refusal, the GCD sweep lasts the recovery, and the
tooltip reads "Instant" with "N sec recovery".

### Melee

What a melee character has that a caster does not, for standing in the pack (the PoE / Last
Epoch / Diablo IV pattern: melee pays in risk, so it gets area, sustain and a defence layer of
its own):

- **Pace.** A swing comes twice as often as the weapon's own speed (never faster than 1 sec), and
  each white swing and Seal of Righteousness hit is half: the same damage in twice the hits, so a
  two-hander swings every ~1.8 sec, not 3.6. Procs a minute (Seal of Command, weapon enchants,
  item procs) stay at the weapon's own rate.
- **Cleave.** Every swing strikes the other enemies in front of it for 55% of the hit (40% until
  2026-10-22); Wide Swing's ranks make it 70/85/100%.
- **Fortify.** A swing that lands means 15% less damage taken for 4 sec.
- **Life on hit.** A swing heals 1% of maximum health for each enemy it strikes, the first five.

### Barrier and boons

Diablo IV's **Barrier** (cmangos `Arpg/ArpgBoons.h`): a pool that takes blows before health.

- Each gain adds to the pool, up to half of maximum health, and holds it 6 sec from the latest
  gain; then it is gone. Every blow of any school spends it first, shown as an absorb.
- On the HUD it is a silver film over the health orb, from the bottom up to its share of maximum
  health, a silver rim while it holds, and its amount beside the health ("412 +120").
- **Sources:** the warlock's Fel Ward (below); shrines and nodes to come.

**Boons** are the named, timed effects on the player, listed right of the health orb with their
time left (a shrine's), and the standing effects of the place the player is in, in red with no
timer (a nightmare dungeon's affixes). One message carries both: `SMSG_ARPG_BOONS` (0x355): `u32`
barrier, `u32` maximum health, `u8` n, per boon `u8` id, `u8` kind (0 timed, 1 standing), `u32` ms
left, `u32` ms in all, cstring name.

## Skills and skill trees

### Specialisation

Jeff's calls: slots unlock by level and show on the bar; points come from character levels;
drops gate the biggest upgrades; and spells have no ranks.

- **Five slots, unlocked by level:** the first at level 1, then 10, 20, 30 and 40. Vanilla hands
  out spells slowly, so this follows its pace, and each new slot is a levelling milestone.
- **The bar is the build:** a specialised skill sits on the ARPG bar (left-click, right-click,
  keys 1 to 4), so what's on the bar is what the character is. Buffs, Blessings and utility spells
  stay castable from the spellbook, without a tree.
- **Skill points: one per level from level 2,** 59 at 60 against five trees of 20, so a character
  fills two or three skills and dabbles in the rest (playtest at 20: two a level was far too
  strong). Points go into whichever specialised skill you like, so a character can rush one skill
  early. A character with more spent than its level gives gets them all back at login.
- **Items raise a skill past 20:** "+2 to Judgement" raises that skill's cap to 22 and gives the 2
  points, spendable only in Judgement.
- **Respec is free:** points move freely within and between trees, and unslotting a skill refunds
  its points.
- **Drops gate the biggest upgrades:** capstones need a Codex page, and each tree has a rune
  socket. See "Drops that unlock skills" below.

### Support and defence skills feed the attacks

Nothing forces a defensive or support pick, but every defence and support tree has a branch that
makes the attack skills hit harder, so five attack skills is the weaker build. Paladin examples:

- **Auras:** Sanctity Aura gives +10% Holy damage; Wide Aura spreads it to the whole pack you're
  fighting.
- **Divine:** when Divine Shield ends, your next three Judgements are critical strikes.
- **Hammer of Justice:** your damage is 15% more on stunned or Dazed enemies (Sentence Passed).
- **Seals:** they are the swing's damage, and one skill with it (Crusader). A swing build without
  Seals is half a build.

### What a skill tree looks like

Each tree is a small graph of 12 to 16 nodes, rooted at the skill itself, with three or four
branches and one or two capstones at the ends. The branches compete for the same 20 points.

| Node kind | Look | What it does | Built from |
|---|---|---|---|
| Modifier | Small circle | Numbers: more damage, less cost, bigger radius, longer stun | `SpellModifier` (cmangos spell mods by the skill's family mask) |
| Transformer | Diamond | Changes how the skill works: chains, splits, becomes a projectile, changes damage type | The uniques kit, plus per-skill hooks |
| Synergy | Hexagon | Interacts with another skill or a tag: "Judgement on an enemy in your Consecration..." | Small per-node hooks |
| Capstone | Large star | A big rule change at the end of a branch; needs its Codex page to learn | A hook per capstone |
| Rune socket | Ring | Holds one rune: a portable mechanic for any skill whose tags fit | The uniques kit |

**Choices** (Diablo IV's mutually exclusive upgrades, 2026-10-21): two nodes may share a choice
group, and only one of them may hold ranks. They stand side by side in their cell, the other
greys out once one is taken ("A choice: you took ..."), and refunding the taken one opens the
other. Saved ranks that a reshaped tree no longer allows (a node whose parent holds none, the
second side of a choice) are dropped at login and their points come back. `SMSG_ARPG_SKILLS`
version 3 sends each node's group (`u8`, after its page home). The first choice: the warlock's
Demon tree, Torment → Sacrifice (the horde and Soul Link beneath) or Demonic Sacrifice.

## Drops that unlock skills

Points from levels keep progress steady, so a build never waits on luck for its basics. Drops
gate two things on top: the capstones, and a rune socket per skill.

### Codex pages: the capstones

- **Every capstone is sealed until you read its Codex page,** a book that drops from one named boss
  whose theme fits it (Whirling Strikes from Herod, Walking Consecration from High Inquisitor
  Whitemane). Read once and it's yours for that character; then you still spend points on it.
- **Each page has a home boss and a fallback.** The home boss drops it often enough that a few
  runs find it. Each dungeon's final boss drops one **Codex fragment** in its Cache, and five
  fragments make any page for a skill you have specialised. Bad luck is never a wall, only a
  slower road.
- **Why capstones:** they're the build-defining part, so finding one is an event ("I finally got
  Blessed Hammer"), and every dungeon has something a build wants from it. Leaving the rest of the
  tree open means a character is never weak for lack of a drop.

### Runes: a socket in every skill tree

- **Each skill tree has one rune socket,** opening once 10 points are in that skill.
- **Runes drop as loot** (blue and better, from bosses and champion packs) and add one portable
  mechanic to the socketed skill, if the skill carries the rune's tag. Socketing is free and
  reversible, so a new rune is an experiment, not a commitment.
- **Runes reuse the uniques kit,** so they're cheap to build and they make every skill a
  candidate for chaining, splitting or echoing, while uniques stay the strongest version of each.
  The same kit on the same skill from a rune, a unique and a tree node doesn't stack: the strongest
  applies.

| Rune | Fits | Effect |
|---|---|---|
| Chains | Spell, single target | The skill chains to 1 more enemy at 50% |
| Splitting | Projectile | +2 projectiles at 40% |
| Echoes | Spell | Every 4th cast repeats at 50% |
| Expanse | Area | +30% area |
| Lingering | Ground, Duration | +50% duration |
| Fury | Melee | Also strikes every enemy in front of you at 30% |
| Haste | Any | −25% cooldown, −15% damage |
| Leech | Any damage | 3% of the damage dealt heals you |
| Command | Control | The control effect spreads to 1 enemy nearby |
| Sanctity | Holy | +15% damage, and the skill counts as Area for Area bonuses |

**Built (Codex pages and runes):** `Arpg/ArpgSkills.{h,cpp}` and `Arpg/ArpgCodex.{h,cpp}` on the
server; the Skills tab on the client.
- *Pages:* each of the eleven built capstones is sealed until its page is read (Dual Aura waits for
  the Auras tree). A page is an item the server writes into `item_template` itself at start, so no
  SQL needs applying; picked up, it is read at once rather than going into the bags. Home bosses
  drop their page 35% of the time, plus 7 points per difficulty tier. A page already read tears
  into two fragments. Capstones taken before pages existed count as read.
- *Fragments:* one from each final boss's Cache; no other kill drops them (a page already read
  still tears into two). Five unseal any capstone of a specialised
  skill: Shift-click it in the Skills tab.
- *Runes:* the socket (top right of a skill's tree) opens at 10 points; click it to choose. Nine
  runes, each fitting some skills: Chains (Judgement, Wrath), Shattering (Judgement, Crusader,
  Wrath), Expanse (Consecration, Wrath), Lingering (Consecration, Hammers), Haste (Judgement,
  Consecration, Hammers, Wrath, Holy Light), Leech (Crusader, Judgement, Wrath, Hammers), Command
  (Hammers), Sanctity (every skill), Fury (Crusader). A kit rune on a skill whose tree already has that kit adds to it
  (Chains: one more jump on Chain of Judgement; Shattering: +15% on Hammer of Light). Runes drop
  from champions 2%, rares 8%, dungeon bosses 12%, Wardens 30%, Caches 40%. Socketing is free and
  the rune stays held; unslotting the skill empties its socket.
- *Dev tools:* the ARPG View page's Test Loot Quality has a **Codex** stop, which drops a page,
  fragments and runes.
- Saved in `character_arpg_codex`, `character_arpg_held` and `character_arpg_socket`, which the
  server creates.

### Cross-skill procs

Jeff's example: "if you crit with a melee weapon then your next Holy Light is instant cast". One
skill sets another up, so a build plays as a rotation rather than one button. Each proc has an
internal cooldown, so it paces the rotation rather than replacing it.

**Built:** `Arpg/ArpgProcs.{h,cpp}` on the server.

How a proc works:
- *Triggers:* a swing's crit, a damage spell's hit or crit, a heal's crit, a spell landing on an
  enemy (a stun), or a kill.
- *Effects:* the next cast of the named spells is instant, deals more, is a sure critical strike or
  costs no mana; or their cooldowns end now.
- *The readied cast* is a spell modifier on the player, so the client's cast bar shows the instant
  cast. It is spent by the next cast it fits and taken away when that spell is done. An instant
  spell, such as a Seal, does not spend an instant-cast proc. It also goes if it waits too long
  (8-12 seconds), or if its rune is unsocketed or its item taken off.
- *The combat log* says each one: "**Rune of Grace** readies your next Holy Light or Flash of
  Light: instant."

| Rune | Fits | Proc (internal cooldown) |
|---|---|---|
| Grace | Strike | A critical swing: the next Holy Light or Flash of Light is instant (10 s) |
| Verdict | Judgement | A Judgement that hits: the next Exorcism or Hammer of Wrath deals 50% more (8 s) |
| the Anvil | Hammer of Justice | Stunning an enemy ends Judgement's cooldown (10 s) |
| Fervour | Holy Wrath, Exorcism | A critical hit: the next Judgement is a sure critical strike (8 s) |
| Zeal | Crusader | A critical Seal hit: the next Holy Light or Flash of Light costs no mana (10 s) |

| Unique | Proc (internal cooldown) |
|---|---|
| Hand of Edward the Odd | A critical swing: the next spell is instant (20 s) |
| Ironfoe | A critical swing: the next Judgement deals 60% more (6 s) |
| Verigan's Fist | A kill ends the cooldowns of Hammer of Wrath and Exorcism (8 s) |
| Benediction | A critical heal: the next Exorcism, Holy Wrath or Holy Shock deals 50% more (8 s) |
| Staff of Jordan | A critical Frostbolt: the next Fireball or Pyroblast is instant (8 s) |
| Meteor Shard | A critical swing: the next Sinister Strike, Eviscerate, Heroic Strike or Mortal Strike costs nothing (8 s) |

The uniques' proc lines reach the tooltip with the mechanic lines (`SMSG_ARPG_ITEM_MECHANICS`).
The runes drop as the others do.

**Every unique has a power of its own.** A named blue or purple without an authored mechanic or
proc (the tables above and `ARPG-UNIQUES.md`) carries a generated one (`ArpgProcs.cpp`,
`GeneratedPowers`). It is built from four parts, with numbers that grow with the item's level:

- **Trigger:** a critical swing, a critical spell, a kill, a heavy blow (a fifth of your health),
  or a critical heal.
- **Effect:**
  - a nova of a damage type around you;
  - an arc to the 3 enemies nearest the target;
  - the slain enemy exploding (kills only);
  - an ailment burst on every enemy within 8 yards (Bleed, Judged, Ignite, Chill, Overload,
    Poison or Wither);
  - a heal, mana, a fury (more damage for 6 sec) or swiftness (movement speed for 4 sec).
- **Damage type:** for the four damaging effects, one of the seven.
- **Rider:** a smaller second effect, or none: a heal, mana, a short fury or swiftness, or an
  ailment on the target.

A hit, heal or blow sets the size of the damage (a heal's or blow's at the ARPG's size against
each creature); a kill's comes from the slain enemy's maximum health, held under 60% of yours.
Each has its own cooldown by trigger: swings 2 sec, spells 3, kills 1, blows 8, heals 5, and a
fury or swiftness no sooner than it runs out.

**The numbers** at level 60, for a hit-based trigger:
- nova 30% of the hit, arc 35%, ailment burst as a hit of 60%;
- kills 8% / 10% / 15% of the slain enemy's maximum health, explosion 12%;
- heal or mana 4% (12% and 8% on a heavy blow);
- fury 15% more damage, swiftness 20%.

They scale with the item's level: half at level 1, all at 60, then a quarter more per ten levels
(about ×1.7 in Naxxramas). So a level 20 unique's power is about two thirds of a level 60 one's.

**No two uniques of a kind share a power.** Each kind (weapons; armour; jewellery, cloaks and
off-hands) has about 1,500 combinations of trigger, effect, damage type and rider. They are dealt
out over that kind's uniques in entry order, a coprime stride apart, so set pieces that sit next
to each other draw very different powers. The server log says how many uniques of each kind
were dealt powers. The name is built from the parts ("Cinder Nova", "Rime Lash", "Greater
Bloodrush"), and the tooltip shows it in gold under the "Unique" line with its full text, for
example "A critical swing sends out a nova of fire: 30% of the hit to every enemy within 8
yards, and heals you for 2% of your maximum health (once every 2 sec)."

A readied "+damage" or "sure crit" waits for a spell that deals damage or heals, so a Seal or a
blessing in between does not waste it.

### Levels past 20

"+N to <skill>" item affixes are the only route past 20, so the third source of skill power is
plain gear. Tomes (a third drop type) aren't needed.

**Built:** four affixes, on weapons and amulets only, so they stay rare: "+1 to all Melee skills",
"+1 to all Spell skills", "+1 to all Holy skills", and "+1 to all Fire, Frost, Nature, Arcane and
Shadow skills". From about item level 69 they roll +2. They work by the skill's tags (its spells'),
so they suit every class without a table per skill: for a paladin, Holy reaches Crusader, Holy Light,
Judgement, Consecration, Hammers and Wrath, and Melee reaches Crusader.
- Each level the gear gives a skill raises its cap by one (20 + levels) and gives a point that
  only that skill can spend; those are spent first. The Skills tab's total counts them for the
  slotted skills (`ArpgSkills.cpp`, `GearSkillLevels`).
- Taking the gear off gives the points back from the deepest ranked nodes first, one at a time,
  so the rest of the tree stays; the same trim handles points rebalanced by a level change
  (it used to empty every tree). It waits until the excess has lasted 10 seconds out of combat
  with the character's gear known, so a login (the gear's affixes load after the skills) or a
  weapon swap never costs ranks. The client is sent the skills again whenever gear's levels
  change.

## Spells without ranks

- **A skill is one spell with no "Rank N".** The spellbook shows one Judgement; trainers no
  longer teach spells. A class's skills unlock at the level vanilla first gives them, for
  free; a talent-granted spell at its talent row's level (Consecration at 6, the paladin's one
  exception). Built: phase 1 gives the talent spells, phase 2 the trainer spells.
- **Base power follows your level smoothly.** The server takes vanilla's rank data for your level
  and blends between the two ranks either side of it, so damage and mana cost climb a little every
  level instead of jumping at rank 2, 3, 4.
- **On top of the base:** gear (spell power and attack power, through vanilla's coefficients),
  attributes (their ARPG effects), tags (passive and item bonuses), and the skill's own tree.
- **Downranking goes away** (a cheaper low rank for mana). Mana pacing moves into the trees and
  the web instead: Mana Strike on Crusader, Righteous Mind on Judgement, Spirit, Illumination.
- **How it's built:** first, at each level-up the server teaches the vanilla rank for that level
  and removes the lower one, and the client hides "Rank N". The smooth blend between ranks comes
  second, as a damage and cost multiplier.

## The passive web

The class-wide tree. This is the part that should look like an ARPG at a glance: a web of
nodes joined by paths, spreading out from a start node in the middle.

- **About 90 nodes per class** in three regions, one per class fantasy, joined around the start
  and by bridges between regions.
- **Allocation by adjacency:** a node can be taken only if it connects to one you have. The
  start is free.
- **Small nodes** (most of the web, 1 point): +attributes, +% tag damage, +ARPG stats.
- **Notables** (about 18, 1 point, larger and named): one meaningful effect each. Some revive the
  best vanilla talents as single strong nodes ("Vengeance" in one point, not five).
- **Keystones** (5 per class, at the region edges): rules that change how the class plays, with
  a cost attached.
- **Points:** one every second level (30 at 60), plus one for the first kill of each dungeon and
  raid final boss (about 20 more), about 50 of the web's 88 nodes. A level 60 takes about 80 of 90, so the last few choices still matter, and
  raid progress grows the build.
- **Respec:** free, out of combat.

## Items

Vanilla items keep their stats. On top of them, the ARPG layer adds:

- **Affixes on generic greens and blues** (`ARPG-ITEMISATION.md` steps 2 and 3), now drawn from
  the same vocabulary as the web:
  - **Attribute:** +Strength, +Spirit…
  - **Tag:** +12% Holy damage, +15% Area damage, +8% Seal damage.
  - **ARPG stat:** +6% movement speed, +8% cooldown recovery, +12 life on kill, +3 mana on hit.
  - **Skill:** +1 to Judgement, +1 to all Holy skills. These are rare and valuable, and they go
    in the item's fourth enchantment slot.
- **Built (rarities and affixes):** `Arpg/ArpgAffixes.{h,cpp}` on the server. Loot is Diablo's
  on vanilla's items: every weapon and armour piece dropped for an ARPG player is a base and rolls
  its rarity at the drop.
  - **Ordinary items** (vanilla's whites and greens; a green's "of the Bear" suffix stays as its
    implicit) roll **white** (no affixes), **green** (1-2), **blue** (3-4) or **purple** (5-6).
    The odds follow who dropped it (trash 50/33/13/4, a champion or open-world elite 15/40/33/12, a rare,
    Warden or boss 0/25/45/30) and lean up with the dungeon tier. The name shows the rolled
    colour, in the tooltip, the chat link and on the ground.
  - **Uniques** are vanilla's named blues and purples (and every mechanic item, `ARPG-UNIQUES.md`):
    gold, their authored stats their implicits, a power of their own, and affixes of their own.
    Which affixes, and how many (a vanilla blue 4, a vanilla purple 6), are fixed by the item, so
    every copy of a unique has the same ones; only their values roll within their ranges. The ground label's border and the beam's core, and a "Unique
    (Rare: 3 affixes)" line in the tooltip, show which.
  - A raid drop (item level 66+) or a Torment drop rolls the top of its rarity's range.
  - Wardens drop two blues, a 30% purple and a 35% unique; a final boss's Cache, per player, a
    blue, a green, a 25% purple and a 50% unique. Test loot's quality is the rarity asked for.
- **The affixes** (item level 60 values on Normal, and by the dungeon tier, x1.15 Hard up to x2
  Torment III). Flat ones (attributes, attack power, spell damage, added damage, resistances,
  life on kill, regeneration, thorns) grow as vanilla's own stats do: by half again every nine
  item levels or so, a level 20 drop about a seventh of a level 60 one. Percentages grow in step
  with the item level to 60 (a level 30 item rolls half the level 60 value, a level 40 two
  thirds, a tenth at least), so ten levels always show: "+% Holy damage" is 3-10% at 20, 5-15% at
  30, 7-20% at 40, 10-30% at 60. Past 60 both grow alike, Molten Core a third more, Blackwing Lair
  about twice, Naxxramas three to four times (Holy damage 38-114% there).
  - **Attributes:** +Strength, Agility, Stamina, Intellect, Spirit (8-30); +all attributes
    (jewellery, 4-14).
  - **Damage:** +% Holy, Physical, Fire, Frost, Arcane, Nature, Shadow (10-30), Area (12-36),
    Projectile (10-27), Melee and Spell (8-24), damage over time and channelled (10-30), Seal
    (weapons, 10-30); +% to elites and bosses (8-22); +% to Undead, Demons, Beasts, Humanoids,
    Dragonkin, Elementals (weapons, 12-36).
  - **Offence:** +attack power (10-40), +spell damage (6-24), +% attack speed (weapons, 3-8, at
    most 20), +% cast speed (3-8, at most 20), +% critical strike chance and spell critical
    strike chance (1-3, at most 8), +% spell critical strike damage (jewellery, 12-36, added to a spell's critical strike multiplier), +% melee and
    spell area (3-10), +% cooldown recovery (3-10), less mana cost (3-8%).
  - **Sustain:** life leech (weapons, 0.5-2% of damage dealt) and mana leech (jewellery,
    0.3-1.2%), +life on kill (5-40), +mana on hit (2-10), +health a second (armour, 2-10), +mana a
    second (jewellery, 1-5), +% healing (8-26), +% globe and flask healing (armour, 5-20).
  - **Defence:** +% maximum health (armour, 3-8) and mana (3-8), +% armour (armour, 5-25), +%
    block (shields, 2-6), less damage taken (armour, 2-6%, 40% in all at most), thorns (armour,
    6-30 Nature damage back at each attacker in reach), +Fire, Nature, Frost, Shadow or Arcane
    resistance (5-20), +all resistances (jewellery, 3-10).
  - **Utility:** +% movement speed (boots, 3-8), +% gold found (5-25), +% experience (jewellery,
    2-6).
  - **Damage** (see Damage above; weapons, gloves and jewellery unless said): adds Physical, Holy,
    Fire, Frost, Nature or Shadow damage to attacks (an average of 10-24 at vanilla's size, shown
    at the ARPG's: "Adds 70 to 164" at item level 60), adds Holy, Fire, Frost, Arcane, Nature or
    Shadow damage to spells (12-28), +% critical strike multiplier (10-30, at most 60), % chance to
    cause Bleeding, Judge, Ignite, Chill, Overload, Poison or Wither (any slot, 5-15, at most 40),
    +% increased damage with ailments (any slot, 10-30).
    An item stored before these existed rolls from the old list (`character_arpg_item.pool` 0), so
    its affixes never change; new drops roll from the whole list. A unique's fixed affixes always
    come from the old list, so every copy of it, old or new, has the same ones.
  - **Skill levels** (weapons and amulets, 1, 2 from about item level 69): +to all Melee, Spell,
    Holy, or Fire/Frost/Nature/Arcane/Shadow skills (see "Levels past 20").
  The roll is one seed per item, kept in `character_arpg_item` with its tier and rarity; worn
  affixes add into the same totals as the web, and the character sheet's Totals list them. The
  tooltip shows them as green lines after the item's own enchants (`SMSG_ARPG_ITEM_AFFIXES`,
  0x343, which carries the rolled colour and a unique's border too). The "+N to a skill" affixes
  are built (see "Levels past 20").
- **Built (crafting):** `Arpg/ArpgCraft.{h,cpp}` and the crafting half of `Arpg/ArpgAffixes.cpp`
  on the server, the Forge window on the client (the anvil button beside the town portal's, or
  the Forge key binding). Put an item in by dragging it onto the Forge's slot, or Alt-clicking it
  in the bags or on the character. Any weapon or armour piece takes crafting, a quest reward
  included.
  - **Orbs**, PoE's currency on vanilla's gems, drop on the ground:
    - champions 20%, rares one plus 25% for a second, dungeon bosses 1-2;
    - anything near the looter's level 1%;
    - each dungeon tier adds 5 points to these chances;
    - zone event caches 2-4, bounties 1-3, and salvage now and then.

    Picked up, they go into the character's purse, not the bags. The common ones drop most:
    | Orb | Effect |
    |---|---|
    | Orb of Awakening | A white item turns green, with 1-2 affixes |
    | Prism of Refinement | A green turns blue, adding affixes up to 3-4 |
    | Orb of Ascension | A blue turns purple, adding up to 5-6 |
    | Fel Orb | Every affix rerolled, as many as before |
    | Star of Invocation | One more affix, while the rarity has room |
    | Void Pearl | One affix, at random, gone |
    | Arcane Lens | Every affix's value rerolled |

    A unique's affixes are its own: it takes the Arcane Lens (new values) and shards that raise an
    affix it already has, but no orb or shard that changes which affixes it has.
  - **Shards**, Last Epoch's:
    - *Salvage* destroys an item from the bags. It gives a shard of one of its affixes (two from a
      blue or better), and a chance at an orb: 20% green, 35% blue, 50% purple, 60% unique (none from an item
      with no affixes).
    - *A shard* on an item adds its affix, when the affix fits the item and its rarity has room, or
      raises the value it already has a third of the way to the item's top.
    - An item takes four shard crafts in all; its tooltip counts them ("Forged 2/4").
  - **Gambling**, Diablo IV's (2026-10-22): the Forge's bottom rows name sixteen slots (head,
    chest, finger, a two-hand weapon...). A click spends gold on a random item for that slot near
    the character's level, rolled as a drop is but with better odds: 4% unique, 14% purple, 36%
    blue, the rest green. The price is 30 copper times the level squared (30 silver at 10, 4.8
    gold at 40, about 11 gold at 60), shown on the Forge. Not in combat. It gives gold a use
    beyond the vendors, and a way to chase a slot the drops keep missing.
  - A crafted item keeps its affixes as a list in place of its seed's
    (`character_arpg_item.affixes`, `forged`). An affix's id is its place in the list; new affixes
    only ever go on its end. The orbs' `item_template` rows (92001-92007) are written at start, as
    the Codex's are.
  - `CMSG_ARPG_ACTION` kind 24: `u8` op (1-7 an orb, 8 salvage, 9 a shard, 10 a gamble), `u8`
    bag (255 the equipped slots, 0 the backpack, 1-4 the bags), `u8` slot from 1, `u16` affix (a
    gamble's slot kind, 1-16, with bag and slot 0).
    `SMSG_ARPG_PURSE` (0x34B): `u8` orb count, per orb `u8` id, `u16` held, cstring name and
    text; `u16` shard count, per shard `u16` affix, `u16` held, cstring name.
- **Built (base items for the raids):** `Arpg/ArpgBases.{h,cpp}` on the server. Vanilla's whites
  and greens stop near item level 63, so a rolled item from a raid or a high dungeon tier used to
  be a dungeon item (a weak base, and affixes scaled from its own item level).
  - **The bank:** at start the server writes one base for every armour slot and type, cloak,
    neck, ring, shield, off-hand and weapon kind, at item levels 64, 68 ... 92 (entries 94000 up).
    Each kind comes in up to three implicit families, so that is about 140 per level, 1,100 in
    all. Each:
    - has implicits, vanilla's green suffixes, scaled to its item level and slot: of the Bear
      (Strength, Stamina), of the Tiger (Strength, Agility), of the Monkey (Agility, Stamina), of
      the Eagle (Stamina, Intellect), of the Owl (Intellect, Spirit), of Sorcery (Intellect and
      spell damage), of Healing (Intellect and healing). Spell damage and healing are vanilla's
      own equip spells. A level 64 chest carries about +14/+15, a Naxxramas one about +20/+20.
      The families fit the kind: plate is Bear, Tiger or Healing; cloth Sorcery, Healing or Owl;
      mail Monkey, Eagle or Healing; and so on;
    - has its armour, weapon damage or block fitted from vanilla's own blues and epics of its
      kind at that item level, at 90% of them;
    - wears the look of a vanilla blue or epic of its kind nearby;
    - takes a name for its content and its family: Emberforged (64), Molten (68), Draconic (72),
      Chromatic (76), Qiraji (80), Obsidian (84), Necrotic (88), Frostbound (92), as in "Molten
      Breastplate of the Bear".

    The first bank's bases (93000 up, no implicits) stay valid for anyone holding one, but no
    longer drop.
  - **Where they drop:** any rolled drop at item level 64 or more takes its base from the bank, so
    its affixes scale from that level. A dungeon's high tiers reach it too: Brutal and Torment
    caches, Wardens, champions and rares.
  - **In a raid**, a white or green the corpse drops becomes the bank's base of its kind at the
    raid's item level, plus 2 per tier:
    | Raid | Item level |
    |---|---|
    | Zul'Gurub | 64 |
    | Molten Core, Ruins of Ahn'Qiraj | 66 |
    | Onyxia | 68 |
    | Blackwing Lair | 74 |
    | Temple of Ahn'Qiraj | 80 |
    | Naxxramas | 88 |

    A raid foe drops a rolled base 12% of the time (green 50%, blue 35%, purple 15%), a boss 2 (one
    more per two tiers), blue or purple.
  - **Never uniques:** rolled drops, champions' and rares' included, are bases. Uniques are
    vanilla's named blues and purples, from their own loot tables.
- **Uniques get one more hook:** "+2 to Judgement's Chain node", raising one node past its cap.
  That ties a unique to a build rather than a class.
- **Affix text** reaches the client the way uniques' lines do, as a table sent at the hello, so
  the tooltip shows "+1 to Judgement" in the item's own colour.

### Synergies

Built. One skill sets another up: the cross-skill proc engine (`Arpg/ArpgProcs.cpp`, "Cross-skill
procs" above) now takes its rows from the skill trees too. Every major skill has a **synergy
node** [Syn], one rank, in place of a plain numbers node: a hit, crit, stun or landing with one
skill readies the next cast of another, within a few seconds, once every few seconds. A readied
cast is instant, free, empowered (more damage), a sure crit, wider (more area), or carries a
uniques-kit row of its own for that one cast (a burst, a chain, shards, a spread, a pierce),
spent by the cast and gone when its hits are done. The client hears of the instant cast as stock
(its cast bar), and the proc line names what was readied.

| Class | Node | Skill | Sets up |
|---|---|---|---|
| Mage | Kindling Blast | Fire Blast | Critical Fireball → Fire Blast 60% more, bursts within 6 yd for 50% (4 sec) |
| Mage | Hot Streak | Fireball | Critical Fire Blast or Scorch → instant Pyroblast |
| Mage | Winter's Grasp | Frostbolt | Frost Nova → instant Frostbolt that shatters into 3 shards for 50% |
| Mage | Arcane Echoes | Arcane Missiles | Critical Arcane Explosion → free Arcane Missiles, each missile chaining once for 50% |
| Mage | Resonance | Arcane Explosion | Critical Arcane Missile → Arcane Explosion 50% wider |
| Mage | Firestorm | Storms | Critical Fire Blast or Scorch → instant Flamestrike |
| Paladin | Crusader's Verdict | Judgement | Critical swing → Judgement 50% more |
| Paladin | Hallowed Strike | Consecration | Judgement hits → Consecration off cooldown |
| Paladin | Wrath of the Light | Hammers | Critical Judgement or Exorcism → Hammer of Wrath 60% more, piercing every enemy for 60% |
| Paladin | Righteous Thunder | Wrath | Hammer of Justice stuns → instant Holy Wrath |
| Paladin | Crusader's Grace | Holy Light | Critical Judgement, Exorcism or Hammer of Wrath → instant Holy Light or Flash of Light |
| Druid | Lunar Eclipse | Wrath | Critical Wrath → instant Starfire bursting within 6 yd for 40% |
| Druid | Moonkissed | Moonfire | Critical Wrath or Starfire → Moonfire lands on 3 more |
| Druid | Gathering Storm | Hurricane | Critical Moonfire or Starfire → free Hurricane |
| Druid | Roaring Swipe | Bear | Demoralizing Roar → Swipe 60% more |
| Druid | Bloodied Bite | Cat | Critical Claw or Shred → Ferocious Bite a sure crit |
| Druid | Overgrowth's Gift | Rejuvenation | Entangling Roots → instant Regrowth or Healing Touch |

**Synergy uniques** (found by item name, as the druid's are):

| Item | Sets up |
|---|---|
| Mana Igniting Cord | Critical Fireball → Fire Blast off cooldown |
| Robe of the Archmage | Frost Nova → Fire Blast a sure crit |
| Wolfshead Helm | Critical Claw or Shred → Rake or Rip free (beside its Shred arc) |
| Staff of the Shadow Flame | Critical Wrath → Starfire 50% more (beside its Starfire echo) |

The paladin's synergy runes (Grace, Verdict, Anvil, Fervour, Zeal) and its proc uniques stay.

### Sets

Built. Twelve sets, one per class at levels 20, 30, 40 and 50, for the classes there are so far
(cmangos `Arpg/ArpgSets.{h,cpp}`).

- **Pieces:** four at 20 and 30 (chest, legs, hands, feet), five at 40 and 50 (head, shoulders,
  chest, legs, hands), in the class's armour: the paladin's mail, then plate from 40; the mage's
  cloth; the druid's leather. Green, wearable by the class alone, with the class's attributes
  (paladin Strength, Stamina, Intellect; mage Intellect, Stamina, Spirit; druid Agility,
  Intellect, Stamina) on a budget of the level and slot. They never roll affixes.
- **Looks:** each piece wears a vanilla item's model and armour, from gear at that level. The
  server picks a look family for the set: the vanilla greens and blues of that armour at the
  set's level (3 below to 6 above) whose names share their first word ("Glimmering",
  "Scorpashi"...) and cover the most of its slots, so the pieces match; a slot the family lacks
  takes the nearest item of its armour and slot. The server log names each set's family.
- **Drops:** a creature in the set's level band (its level to 9 above; the 50s' to 63) drops a
  random piece of the looter's class's set: a boss 25%, a rare 12%, a champion or elite 4%, the
  rest 0.6%.
- **Bonuses** add into the item totals as affixes do; the whole set gives a power, a uniques-kit
  row on a class spell, named for the set.
- **Tooltips:** the pieces' `itemset` is the set's own id (1001 up, past `ItemSet.dbc`'s), and the
  server sends the catalog with the hello (`SMSG_ARPG_SETS`, 0x34F); the client's stock set block
  draws it, worn pieces white and met bonuses green.

| Set | Class | Level | 2 pieces | 3 pieces | 4 pieces | Whole set |
|---|---|---|---|---|---|---|
| Squire's Resolve | Paladin | 20 | +8 Strength | +10% Holy damage | | Judgement chains to 1 more within 10 yd, 50% |
| Crusader's Vow | Paladin | 30 | +12 Stamina | +6% attack speed | | Seal of Righteousness strikes all in front, 30% |
| Lightbringer's Bulwark | Paladin | 40 | +10% armour | +12% Holy damage | +1 to all Holy skills | Judgement bursts within 6 yd, 40% |
| Ashen Oath | Paladin | 50 | +15 Strength | +15% critical strike damage | +1 to all Melee skills | Exorcism chains to 2 more within 10 yd, 60% |
| Apprentice's Embers | Mage | 20 | +8 Intellect | +10% Fire damage | | Fireball launches 1 extra, 40% |
| Frostweave Mantle | Mage | 30 | +10% Frost damage | +5% cast speed | | Frostbolt shatters into 2 shards within 8 yd, 35% |
| Arcanist's Lattice | Mage | 40 | +15 Intellect | +12% Arcane damage | +10% maximum mana | Each Arcane Missile jumps to 1 more, 40% |
| Archmage's Conflux | Mage | 50 | +20 Intellect | +15% critical strike damage | +1 to all Spell skills | Every 3rd Fireball echoes, 60% |
| Grovewalker's Hide | Druid | 20 | +8 Agility | +10% Nature damage | | Wrath chains to 1 more within 10 yd, 50% |
| Moonshadow Garb | Druid | 30 | +10 Intellect | +10% Arcane damage | | Moonfire also lands on 1 more within 8 yd |
| Feralheart Raiment | Druid | 40 | +15 Agility | +12% Melee damage | +6% attack speed | Claw strikes all in front, 40% |
| Wildheart Vestments | Druid | 50 | +15 Intellect | +12% Nature damage | +1 to all Spell skills | Starfire bursts within 8 yd, 40% |
| Shadowsworn Rags | Warlock | 20 | +8 Intellect | +10% Shadow damage | | Shadow Bolt pierces 1, 60% |
| Summoner's Bindings | Warlock | 30 | +10% Minion damage | +15% Minion life | | Corruption lands on 1 more within 8 yd |
| Felfire Regalia | Warlock | 40 | +15 Intellect | +12% Fire damage | +1 to the horde's size | Immolate bursts within 6 yd, 40% |
| Dreadlord's Mantle | Warlock | 50 | +20% Minion damage | +12% Shadow damage | +1 to all Spell skills | Shadow Bolt chains to 2 more, 50% |

## The paladin, in full

### Identity

The ARPG paladin is a holy warrior who **controls ground**: consecrates it, stands in it, and
punishes anything that comes close. Its three fantasies:

- **Crusader:** a hammer-swinging Seal fighter, cleaving packs with every swing.
- **Templar:** an aura-and-shield fortress that hurts everything around it.
- **Lightbringer:** a holy caster whose heals and holy fire are one and the same.

Two vanilla gaps get fixed: **Consecration becomes a baseline spell at level 6** (it's a talent in
1.12), because a paladin without area damage can't play an ARPG; and **Exorcism and Holy Wrath
hit any creature type** for ARPG players (their undead-and-demon limit is a raid mechanic).

### Paladin skills that can be specialised

**Built (revision 2):** six skills for five slots, so one is left out. Each tree has three branches
(Holy Light two) of four or five tiers, about 40 points of nodes against the 20 a skill takes
(more with gear's skill levels), so two characters of one skill can play differently. The first
version had eight thin skills, three tiers deep; a character from it has its skill points back
once, its runes back unsocketed, its Seals and Exorcism slots emptied, and the Codex pages it
had read carried to the capstones that took their place (`ArpgSkills.cpp`, `TREE_REVISION`).

| Skill | Covers | Branches | Role |
|---|---|---|---|
| **Crusader** | The held swing and every Seal | Reach, Righteous, Zeal | Melee: a Seal is the swing's damage |
| **Holy Light** | Holy Light, Flash of Light | Radiance, Grace, Retribution | Healing, and healing as offence |
| **Judgement** | Judgement | Chain, Burst, Verdict | Burst, chaining |
| **Consecration** | Consecration | Ground, Smite, Refuge | Ground control |
| **Hammers** | Hammer of Justice, Hammer of Wrath | Thrown, Stun, Execute | Stun and the thrown execute |
| **Wrath** | Exorcism, Holy Wrath | Exorcism, Nova, Zeal | Holy fire: a bolt and a nova; specialised, at any foe |

**[A]** a node from the uniques kit, **[M]** a spell modifier (a "more" when it is damage),
**[H]** a hook, **[Ail]** an ailment node: the skill's hits have that chance per rank to inflict
the ailment, whatever their own damage type (`Ail` in `ArpgSkills.cpp`, rolled in
`ArpgDamage.cpp`). A modifier on a two-spell skill can name one spell ("Hammer of Wrath" only).
**[More]** a share of "more" damage for the skill's hits, its ticks and its minions' (`More` in
`ArpgSkills.cpp`, read in `DamagePartsFor`; the sheet names it). Every tree holds 25-30 ranks
against a skill's cap of 20, so filling one is a choice (2026-10-22).

#### Crusader

| Branch | Node | Ranks | Effect |
|---|---|---|---|
| Reach | Wide Swing [H] | 3 | Swings strike every other enemy in front for 70/85/100%, not the base 55% |
| Reach | Long Arm [H] | 2 | +1 yd melee reach per rank |
| Reach | Heavy Hand [H] | 4 | 5% more Physical damage per rank with a two-hander |
| Reach | Rending Strikes [Ail] | 3 | 8% per rank to cause Bleeding |
| Reach | **Whirling Strikes** [H] | 1 | Every 4th swing strikes all around, 75% |
| Righteous | Sweeping Seal [A] | 3 | Seal of Righteousness strikes every enemy in front, 20/35/50% |
| Righteous | Holy Edge [H] | 4 | 6% more Seal damage per rank |
| Righteous | Judging Seals [Ail] | 3 | 8% per rank to Judge |
| Righteous | Mana Strike [H] | 3 | Seal hits give 1% of maximum mana per rank |
| Righteous | **Twin Seals** [H] | 1 | Two different Seals at once |
| Zeal | Quickened [H] | 4 | +4% attack speed per rank |
| Zeal | Momentum [H] | 3 | 1% more Crusader damage per rank per hit in a row, 5 at most |
| Zeal | Zeal [H] | 3 | +10% attack speed per rank under Seal of the Crusader |
| Zeal | Light of the Crusader [H] | 1 | Seal hits heal 10% of their damage |
| Zeal | **Shockwave** [H] | 1 | Critical swings send a 10 yd shockwave, 40% |

#### Holy Light

| Branch | Node | Ranks | Effect |
|---|---|---|---|
| Radiance | Healing Light [M] | 4 | 6% more healing per rank |
| Radiance | Light's Grace [M] | 3 | +4% critical strike chance per rank |
| Radiance | Swift Light [M] | 3 | -0.2 sec recovery per rank |
| Radiance | **Crusader's Grace** [Syn] | 1 | A critical Judgement, Exorcism or Hammer of Wrath makes the next Holy Light or Flash of Light instant |
| Radiance | **Dawnbringer** [H] | 1 | Heals on yourself also send half their amount as a Holy bolt |
| Grace | Pure Light [M] | 3 | 10% less Holy Light and Flash of Light mana cost per rank |
| Grace | Radiant Light [M] | 3 | Critical heals heal 20% more of their bonus per rank |
| Retribution | Searing Radiance [H] | 3 | Heals strike the nearest enemy within 20 yd for 15% per rank |
| Retribution | Judging Light [H] | 2 | Heals Judge every enemy within 8 yd, a hit of 25% per rank (3 sec) |
| Retribution | Avenging Light [H] | 3 | Healing yourself: 5% more damage per rank for 5 sec |
| Retribution | **Holy Nova** [H] | 1 | Heals burst outward: 50% as Holy to every enemy within 8 yd |

A heal's damage is the heal's size turned to the ARPG's (Arpg::ToDamage) for each creature. With
Lightforged (the web's keystone) a self-heal heals nothing but still strikes.

#### Judgement

| Branch | Node | Ranks | Effect |
|---|---|---|---|
| Chain | Chain of Judgement [A] | 3 | Chains to 1/2/3 more within 10 yd, 50% |
| Chain | Echoing Verdict [H] | 3 | Chain jumps deal 10% more per rank |
| Chain | Condemn [Ail] | 3 | 10% per rank to Judge, beside its crits |
| Chain | **Final Verdict** [H] | 1 | A Judgement kill readies Judgement |
| Burst | Hammer of Light [A] | 3 | Bursts 4/6/8 yd round its target, 40% |
| Burst | Radiance [H] | 4 | 8% more Judgement damage per rank |
| Burst | Severity [M] | 3 | Critical strikes' bonus 20% more per rank |
| Burst | **Sentence** [H] | 1 | 100% more on stunned or Dazed enemies |
| Verdict | Avenger [H] | 1 | Judgement keeps the Seal; its cooldown doubles |
| Verdict | Swift Judgement [M] | 3 | -1 sec cooldown per rank |
| Verdict | **Crusader's Verdict** [Syn] | 1 | A critical swing makes the next Judgement deal 50% more |
| Verdict | Righteous Mind [H] | 2 | Restores 5% of maximum mana per rank |
| Verdict | **Sanctified** [H] | 1 | On an enemy in your Consecration, bursts 8 yd at 50% |

#### Consecration

A paladin's Consecration deals 150% more, innately (2026-10-22): its ticks were a twentieth of a
swing, for 22% of the mana.

| Branch | Node | Ranks | Effect |
|---|---|---|---|
| Ground | Wide Blessing [M] | 4 | +10% radius per rank |
| Ground | Lasting Ground [M] | 3 | +2 sec duration per rank |
| Ground | Burning Ground [H] | 4 | 8% more damage per rank |
| Ground | **Walking Consecration** [H] | 1 | It follows you |
| Smite | Searing Light [H] | 2 | Your Holy damage 5% more per rank on enemies in it |
| Smite | **Hallowed Strike** [Syn] | 1 | A Judgement that hits ends Consecration's cooldown |
| Smite | **Sacred Seal** [H] | 1 | Seals strike twice on enemies in it |
| Refuge | Hallowed Ground [H] | 3 | 1% health a second per rank inside |
| Refuge | Steadfast [H] | 1 | No stuns inside |
| Refuge | **Martyr's Ward** [H] | 1 | Retribution Aura burns every enemy within 10 yd each second, doubled |
| Smite | Devotion [M] | 3 | 10% less Consecration mana cost per rank (after Sacred Seal) |
| Refuge | Zealous Ground [More] | 3 | 6% more Consecration damage per rank (after Martyr's Ward) |

#### Hammers

| Branch | Node | Ranks | Effect |
|---|---|---|---|
| Thrown | Hurled Hammer [M] | 1 | Hammer of Justice flies 10 yd farther |
| Thrown | Ricochet [H] | 3 | Hammer of Justice bounces to 1 more per rank |
| Thrown | Holy Hammer [H] | 3 | Hammer of Justice adds Holy of 30/45/60% of attack power |
| Thrown | **Blessed Hammer** [H] | 1 | Also three blessed hammers at the nearest enemies |
| Stun | Shockwave Hammer [A] | 2 | Stuns 1/2 more within 8 yd |
| Stun | Long Stun [M] | 3 | +0.5 sec stun per rank |
| Stun | Quick Hammer [M] | 3 | -10 sec Hammer of Justice cooldown per rank |
| Stun | **Sentence Passed** [H] | 1 | 15% more damage on stunned or Dazed enemies |
| Execute | Hammer Volley [A] | 2 | Hammer of Wrath throws 1/2 more, 60% |
| Execute | Weighted Hammer [M] | 4 | 8% more Hammer of Wrath damage per rank |
| Execute | **Wrath of the Light** [Syn] | 1 | A critical Judgement or Exorcism makes the next Hammer of Wrath deal 60% more and pierce every enemy for 60% |
| Execute | Piercing Hammer [A] | 3 | Hammer of Wrath flies through 2/4/10, 60% |
| Execute | **Executioner** [H] | 1 | 100% more below 20% health |

#### Wrath

| Branch | Node | Ranks | Effect |
|---|---|---|---|
| Exorcism | Holy Fire [A] | 3 | Exorcism bursts 4/6/8 yd, 35% |
| Exorcism | Searing Word [M] | 4 | 10% more Exorcism damage per rank |
| Exorcism | Chain Exorcism [A] | 3 | Leaps to 1/2/3 more within 10 yd, 60% |
| Exorcism | **Purging Light** [A] | 1 | Passes through: everything within 10 yd behind takes the full hit |
| Nova | Wide Wrath [M] | 4 | +12% Holy Wrath radius per rank |
| Nova | Searing Wrath [M] | 3 | 10% more Holy Wrath damage per rank |
| Nova | Shattering Wrath [A] | 2 | Each enemy struck bursts 4/6 yd, 20/30% |
| Nova | **Wrathful** [H] | 1 | Holy Wrath fires on its own with 4+ enemies within 10 yd (12 sec) |
| Zeal | Swift Exorcism [M] | 3 | -2 sec Exorcism cooldown per rank |
| Zeal | **Righteous Thunder** [Syn] | 1 | A Hammer of Justice stun makes the next Holy Wrath instant |
| Zeal | Exorcist [M] | 3 | +5% critical strike chance per rank, both |
| Zeal | Quick Wrath [M] | 3 | -0.4 sec Holy Wrath cast time per rank (Holy Wrath, a generic-family spell, keeps its cast) |
| Zeal | **Twin Exorcism** [A] | 1 | A second bolt beside the first, 70% |

#### Paladin Codex pages

| Capstone | Skill | Home boss |
|---|---|---|
| Whirling Strikes | Crusader | Herod, Scarlet Monastery |
| Twin Seals | Crusader | Arcanist Doan, Scarlet Monastery |
| Shockwave | Crusader | Ironaya, Uldaman |
| Dawnbringer | Holy Light | Warchief Rend Blackhand, Upper Blackrock Spire |
| Holy Nova | Holy Light | High Inquisitor Fairbanks, Scarlet Monastery |
| Final Verdict | Judgement | Scarlet Commander Mograine, Scarlet Monastery |
| Sentence | Judgement | Emperor Dagran Thaurissan, Blackrock Depths |
| Walking Consecration | Consecration | High Inquisitor Whitemane, Scarlet Monastery |
| Sacred Seal | Consecration | Balnazzar, Stratholme |
| Martyr's Ward | Consecration | Mr. Smite, the Deadmines |
| Blessed Hammer | Hammers | Baron Rivendare, Stratholme |
| Sentence Passed | Hammers | Overlord Wyrmthalak, Lower Blackrock Spire |
| Executioner | Hammers | Pyroguard Emberseer, Upper Blackrock Spire |
| Purging Light | Wrath | Ras Frostwhisper, Scholomance |
| Wrathful | Wrath | Archivist Galford, Stratholme |
| Twin Exorcism | Wrath | Magistrate Barthilas, Stratholme |

Sanctified has no page. Auras, Divine and Holy Shock stay out of the trees: castable from the
spellbook, fed by the web and gear.

**The Skills window** spaces a tree's rows to the window's height, however deep it runs.

### The paladin passive web

**Built (phase 1).** 88 nodes in three regions around a start node. Each region has two entry
nodes, three arms of five small nodes leading to a notable and then a keystone or a second
notable, two side nodes between its arms, and cross-links. Three small "Swiftness" nodes join the
regions near the start, and three bridges join them farther out.

**Crusader region (north-west):** Strength, Melee damage, Seals.

| Kind | Name | Effect |
|---|---|---|
| Entry (×2) | Crusader | +2 to all attributes |
| Arm (×5) | Might | +5 Strength |
| Arm (×5) | Edge | +3% Melee damage |
| Arm (×5) | Fervour | +4% Seal damage |
| Side | Reach | +5% melee area |
| Side | Bloodthirst | +5 life on kill |
| Notable | Two-Handed Mastery | 12% more Melee damage with a two-handed weapon |
| Notable | Conviction | Vanilla's Conviction at rank 5: +5% melee crit |
| Notable | Vengeance | Vanilla's Vengeance at rank 5: +15% Physical and Holy damage for 8 sec after a crit |
| Notable | Righteous Fervour | +15% Seal damage, and vanilla's Benediction at rank 5 (Seals and Judgement cost 15% less) |
| **Keystone** | **Zealot** | +25% attack speed; while a Seal is on you, it drains 1% of your mana a second |
| **Keystone** | **Crusade** | Each kill in the last 5 sec gives 4% more damage (10 kills at most); out of combat you're 20% slower |

**Lightbringer region (north-east):** Intellect, Spirit, Holy, healing.

| Kind | Name | Effect |
|---|---|---|
| Entry (×2) | Lightbringer | +2 to all attributes |
| Arm (×5) | Wisdom | +5 Intellect |
| Arm (×5) | Radiant | +4% Holy damage |
| Arm (×5) | Devotion | +5 Spirit |
| Side | Expanse | +5% spell area |
| Side | Tempo | +4% cooldown recovery |
| Notable | Illumination | Vanilla's Illumination at rank 5: healing crits refund their mana |
| Notable | Holy Power | Vanilla's Holy Power at rank 5: +5% Holy crit |
| Notable | Divine Favour | Every 20 sec, your next Holy spell is a critical strike |
| Notable | Healing Light | Vanilla's Healing Light at rank 3 (+12% Holy Light and Flash of Light), and +10% all healing |
| Notable | Blessed Recovery | Healing yourself gives +10% cooldown recovery for 4 sec |
| **Keystone** | **Lightforged** | Your heals also fly as a Holy bolt at the nearest enemy for their full amount, and no longer heal you; 30% more Holy damage |

**Templar region (south):** Stamina, armour, blocking, area.

| Kind | Name | Effect |
|---|---|---|
| Entry (×2) | Templar | +2 to all attributes |
| Arm (×5) | Fortitude | +6 Stamina |
| Arm (×5) | Plate | +4% armour |
| Arm (×5) | Zeal | +4% Area damage |
| Side | Endurance | +8 life on kill |
| Side | Stride | +3% movement speed |
| Notable | Shield Wall | +10% block chance; blocks heal you for 1% of your health |
| Notable | Redoubt | Vanilla's Redoubt at rank 5 |
| Notable | Reckoning | Vanilla's Reckoning at rank 2: a 20% chance after being hit of an extra swing |
| Notable | Righteous Fury | 20% more Holy damage while three or more enemies are within 8 yd |
| **Keystone** | **Martyr** | 25% of the damage you take strikes every enemy within 10 yd as Holy damage; healing you take is halved |
| **Keystone** | **Unyielding** | You can't be stunned or slowed; you move 20% slower |

**Between the regions:** "Swiftness" (+3% movement speed) near the start; bridges of two
"+3 to all attributes" nodes round a notable: Holy Weapons (Crusader–Lightbringer, +3% Holy
damage per 10 Strength), Aegis (Lightbringer–Templar, +8 Stamina, Intellect and Spirit), Shield
and Hammer (Templar–Crusader, 10% more Melee damage with a one-hander and a shield).

**What's not in the web:** the first tree's skill modifiers (Sweeping Seal, Chain of Judgement…)
and its capstones (Avenger, Martyr's Ward, Dawnbringer, Purifying Light) move to the skill trees
in phase 2. Their server hooks stay. The talent-granted spells (Consecration, Seal of Command,
Blessing of Kings at 20; Blessing of Sanctuary, Divine Favor, Sanctity Aura at 30; Holy Shock,
Holy Shield, Repentance at 40) come free at the level their talent row needs, as "Spells without
ranks" plans; Consecration comes at 6.

### Example builds at 60

- **Hammerdin (Hammers, Judgement, Consecration).** Hammer of Justice
  thrown with Blessed Hammer, Consecration walking with you, Judgement chaining. Web leans
  Lightbringer for Intellect (radius) and Spirit (cooldowns). Wants +Hammer of Justice affixes
  and Spirit gear: a caster-paladin in plate.
- **Crusader (Crusader, Judgement, Consecration).** Wide Swing and Sweeping Seal
  cleave every pack, Twin Seals, Avenger keeping the Seal up. Web takes Crusader with Zealot.
  Wants Strength (bigger arcs) and attack speed: Smite's Mighty Hammer early, Spinal Reaper late.
- **Templar (Auras, Consecration, Hammer of Justice, Holy Shock, Light).** Martyr's Ward and
  Martyr keystone: stand in Consecration in the middle of a pack while auras burn it down. Wants
  Stamina (life on kill) and armour.
- **Lightbringer (Holy Light, Wrath, Consecration).** Searing Radiance and Lightforged turn
  every heal into a bolt, Judging Light judges what stands near the healed, and Holy Nova bursts
  each critical heal; Overflow chains overheal. Wants Intellect and Spirit, and +Light.

## The mage, in full

### Identity

The second class. A caster with three schools that pull apart: **Fire** (Ignite, explosions, big
single hits), **Frost** (chill, freeze, shatter) and **Arcane** (Overload, mana). Every skill
tree and every web keystone leans into one of the ailments, so a build is a school and an
ailment plan, not just a bigger number. Like the paladin, it gets every trainer spell free at its
level and its talent-granted spells (Pyroblast, Blast Wave, Presence of Mind, Ice Barrier...) at
the level their talent row needs.

**How the classes share the systems (built with the mage):**
- A skill is its class's and its id (1 to 6). A class's nodes take their own numbers within each
  skill's hundred: the paladin's 1 to 19, the mage's 20 to 39, the druid's 40 to 59, the
  warlock's 60 to 79, the shaman's 80 to 99. So every capstone has its own Codex page.
- Codex pages and runes drop only for the classes of the ARPG players there. One of another
  class's, looted in a mixed group, goes to the bags to trade instead of the Codex.
- Runes fit each class's skills on their own (`FitsFor` in `ArpgSkills.cpp`); the Skills window
  lists only the class's runes. The mage takes Chains, Shattering, Expanse, Lingering, Haste,
  Leech and Sanctity; the paladin's proc runes (Grace, Verdict, Anvil, Fervour, Zeal), Command
  and Fury stay the paladin's.
- Only the paladin's swing counts as a skill (Crusader). A mage's wand or staff swing is no
  skill's.
- A skill tree's spell modifiers reach its spells by id (every rank and triggered part with the
  name), not by vanilla's family flags: classic shares those bits between unrelated spells (every
  mage bolt carries one, so "Flamestrike is instant" would have made Fireball instant too) and
  some spells have none (Exorcism, Holy Wrath, whose modifiers did nothing before). The client
  is told only of the bits no other spell carries, for its tooltips.
- The generated unique powers already suit every class. The authored uniques' kits are by spell,
  and six are the mage's already (Fireball, Frostbolt, Arcane Missiles, Polymorph).

### Thermal Shock: Frostbolt sets up, fire pays off

Every mage has it, no node needed: Frostbolt, the free filler that gives 4% mana back and makes its target Vulnerable
(`ARPG-ROADMAP-RESOURCES.md`), always chills what it hits, and fire spells deal 30% more to a
chilled enemy. Frostbolt then Fireball beats Fireball alone, in damage and in mana.

### Mage skills that can be specialised

Six skills for five slots, like the paladin's. The hooks are in `ArpgMage.cpp`.

| Skill | Covers | Branches | Role |
|---|---|---|---|
| **Fireball** | Fireball, Pyroblast | Volley, Inferno, Pyre | The bolt: more of them, Ignite, and the big charge |
| **Frostbolt** | Frostbolt, Cone of Cold | Shards, Winter, Cone | Shards, chill and freeze, the cone |
| **Arcane Missiles** | Arcane Missiles | Barrage, Flux, Focus | The channel: seeking missiles, Overload, mana |
| **Arcane Explosion** | Arcane Explosion | Nova, Fury, Mana | The close nova |
| **Storms** | Blizzard, Flamestrike | Blizzard, Storm, Flamestrike | Ground storms |
| **Fire Blast** | Fire Blast, Scorch | Blast, Scorch, Combust | Quick fire up close; sets off Ignites |

#### Fireball

| Branch | Node | Ranks | Effect |
|---|---|---|---|
| Volley | Twin Fireball [A] | 3 | 1/1/2 extra fireballs, at 45/60/60% |
| Volley | Quick Fireball [M] | 3 | -0.15 sec recovery per rank |
| Volley | Piercing Flame [A] | 3 | Flies through 1/2/4 enemies, at 60% |
| Volley | **Flame Echo** [A] | 1 | Every 3rd Fireball echoes, at 70% |
| Inferno | Searing Bolt [M] | 4 | 6% more damage per rank |
| Inferno | Kindle [Ail] | 3 | 8% per rank to Ignite |
| Inferno | Lingering Flame [M] | 3 | Your Ignites burn 15% more per rank |
| Inferno | **Pyroclasm** [H] | 1 | Critical hits explode for 60% within 8 yd |
| Pyre | Quick Pyre [M] | 3 | -0.5 sec Pyroblast recovery per rank |
| Pyre | **Hot Streak** [Syn] | 1 | A critical Fire Blast or Scorch makes the next Pyroblast instant |
| Pyre | Pyre [M] | 3 | Pyroblast's critical strikes 20% more of their bonus per rank |
| Pyre | **Cataclysm** [A] | 1 | Pyroblast bursts within 10 yd, at 60% |

#### Frostbolt

| Branch | Node | Ranks | Effect |
|---|---|---|---|
| Shards | Splintering Bolt [A] | 3 | Shatters into 2/3/4 shards within 10 yd, at 35% |
| Shards | Bitter Cold [M] | 4 | 6% more Frostbolt damage per rank |
| Shards | Glacial Chain [A] | 2 | Chains to 1/2 more within 10 yd, at 50% |
| Shards | **Echoing Frost** [A] | 1 | Every 3rd Frostbolt echoes, at 60% |
| Winter | Deep Chill [Ail] | 3 | 10% per rank to deepen the cold: Freezes the Chilled, Chills the rest |
| Winter | **Winter's Grasp** [Syn] | 1 | Frost Nova makes the next Frostbolt instant, and it shatters into 3 shards for 50% |
| Winter | Razor Ice [M] | 3 | Critical strikes 20% more of their bonus per rank |
| Winter | **Shatterstorm** [H] | 1 | Frostbolt on a frozen enemy shatters for 80% within 8 yd |
| Cone | Wide Cone [M] | 3 | +15% Cone of Cold area per rank |
| Cone | Biting Cone [M] | 3 | 10% more Cone of Cold damage per rank |
| Cone | Swift Cone [M] | 3 | -1.5 sec Cone of Cold cooldown per rank |
| Cone | **Deep Freeze** [H] | 1 | Cone of Cold freezes every enemy it hits |

#### Arcane Missiles

| Branch | Node | Ranks | Effect |
|---|---|---|---|
| Barrage | Seeking Missiles [A] | 3 | Each missile jumps to 1/1/2 more within 8 yd, at 40/55/55% |
| Barrage | Empowered Missiles [M] | 4 | 6% more damage per rank |
| Barrage | Piercing Missiles [A] | 2 | Each missile flies through 1/2, at 50% |
| Barrage | **Arcane Barrage** [A] | 1 | Each missile bursts within 4 yd, at 30% |
| Flux | Unstable Missiles [Ail] | 3 | 6% per rank to Overload |
| Flux | Sharp Mind [M] | 3 | +3% critical strike chance per rank |
| Flux | **Arcane Echoes** [Syn] | 1 | A critical Arcane Explosion makes the next Arcane Missiles free, each missile jumping to 1 more for 50% |
| Flux | **Overcharge** [H] | 1 | Your Overload bursts deal 40% more |
| Focus | Focused Mind [M] | 3 | Critical strikes 15% more of their bonus per rank |
| Focus | Lingering Barrage [M] | 2 | +1 sec channel per rank: one more missile |
| Focus | **Arcane Siphon** [H] | 1 | Each missile that hits restores 0.5% of maximum mana |

#### Arcane Explosion

| Branch | Node | Ranks | Effect |
|---|---|---|---|
| Nova | Wide Explosion [M] | 4 | +10% radius per rank |
| Nova | Arcane Force [M] | 4 | 6% more damage per rank |
| Nova | Volatile [Ail] | 3 | 10% per rank to Overload |
| Nova | **Detonation** [Ail] | 1 | Overloads every enemy it hits |
| Fury | Unstable Power [M] | 3 | +3% critical strike chance per rank |
| Fury | Arcane Fury [M] | 3 | Critical strikes 20% more of their bonus per rank |
| Fury | **Arcane Nova** [M] | 1 | 40% larger radius |
| Mana | Mana Efficiency [M] | 3 | 10% less mana cost per rank |
| Mana | **Resonance** [Syn] | 1 | A critical Arcane Missile makes the next Arcane Explosion 50% wider |
| Mana | **Mana Burst** [H] | 1 | 1% of maximum mana per enemy hit, 5% at most a cast |
| Mana | Arcane Mastery [More] | 4 | 6% more Arcane Explosion damage per rank |

#### Storms

Blizzard's damage is its ticks, so its damage nodes are damage over time ones, and it inflicts no
ailment of its own; Glacial Storm is the hook on its ticks (`OnPeriodicHit`, called from
`SpellAuras.cpp`).

| Branch | Node | Ranks | Effect |
|---|---|---|---|
| Blizzard | Wide Blizzard [M] | 3 | +12% Blizzard radius per rank |
| Blizzard | Biting Storm [M] | 4 | 8% more Blizzard damage per rank |
| Blizzard | Thrifty Storm [M] | 3 | 10% less Blizzard mana cost per rank |
| Blizzard | **Glacial Storm** [H] | 1 | Blizzard chills; each strike has 15% to freeze the chilled |
| Storm | **Firestorm** [Syn] | 1 | A critical Fire Blast or Scorch makes the next Flamestrike instant |
| Storm | Storm Surge [M] | 3 | +8% radius per rank |
| Storm | Raging Storm [M] | 3 | Blizzard's strikes and Flamestrike's burn 6% more per rank |
| Storm | **Eye of the Storm** [M] | 1 | 30% wider |
| Flamestrike | Quick Flamestrike [M] | 3 | -0.3 sec recovery per rank |
| Flamestrike | Searing Ground [M] | 3 | 15% more burn per rank |
| Flamestrike | Kindling Strike [Ail] | 3 | 10% per rank to Ignite |
| Flamestrike | **Meteor** [M] | 1 | Flamestrike is instant |

#### Fire Blast

| Branch | Node | Ranks | Effect |
|---|---|---|---|
| Blast | Swift Blast [M] | 3 | -1 sec Fire Blast cooldown per rank |
| Blast | Explosive Blast [A] | 3 | Bursts within 4/6/8 yd, at 40% |
| Blast | Incineration [M] | 3 | +5% Fire Blast critical strike chance per rank |
| Blast | **Chain Blast** [H] | 1 | A kill with Fire Blast readies it again |
| Scorch | Quick Scorch [M] | 3 | -0.2 sec Scorch recovery per rank |
| Scorch | Arcing Scorch [A] | 2 | Chains to 1/2 more within 10 yd, at 60% |
| Scorch | Scorching [Ail] | 3 | 10% per rank to Ignite |
| Scorch | **Searing Brand** [H] | 1 | Scorch brands with Fire Vulnerability (3% more Fire taken per stack, 5 at most) |
| Combust | **Kindling Blast** [Syn] | 1 | Within 4 sec of a critical Fireball, Fire Blast deals 60% more and bursts within 6 yd for 50% |
| Combust | Blazing [M] | 4 | 6% more damage per rank |
| Combust | **Detonate** [H] | 1 | Fire Blast sets off your Ignite on its target at once: the burn left strikes it and every enemy within 6 yd |

#### Mage Codex pages

| Capstone | Skill | Home boss |
|---|---|---|
| Flame Echo | Fireball | Archmage Arugal, Shadowfang Keep |
| Pyroclasm | Fireball | Pyroguard Emberseer, Upper Blackrock Spire |
| Cataclysm | Fireball | Emperor Dagran Thaurissan, Blackrock Depths |
| Echoing Frost | Frostbolt | Amnennar the Coldbringer, Razorfen Downs |
| Shatterstorm | Frostbolt | Ras Frostwhisper, Scholomance |
| Deep Freeze | Frostbolt | Darkmaster Gandling, Scholomance |
| Arcane Barrage | Arcane Missiles | Arcanist Doan, Scarlet Monastery |
| Overcharge | Arcane Missiles | Mekgineer Thermaplugg, Gnomeregan |
| Arcane Siphon | Arcane Missiles | Immol'thar, Dire Maul |
| Detonation | Arcane Explosion | Archaedas, Uldaman |
| Arcane Nova | Arcane Explosion | Prince Tortheldrin, Dire Maul |
| Mana Burst | Arcane Explosion | Balnazzar, Stratholme |
| Glacial Storm | Storms | Baron Rivendare, Stratholme |
| Eye of the Storm | Storms | Mutanus the Devourer, Wailing Caverns |
| Meteor | Storms | Warchief Rend Blackhand, Upper Blackrock Spire |
| Chain Blast | Fire Blast | Edwin VanCleef, the Deadmines |
| Searing Brand | Fire Blast | Archivist Galford, Stratholme |
| Detonate | Fire Blast | Overlord Wyrmthalak, Lower Blackrock Spire |

Frost Nova, Blink, Polymorph, the armours and the wards stay out of the trees: castable from the
spellbook, fed by the web and gear. Frost Nova's root counts as frozen for Shatter and
Shatterstorm.

### The mage passive web

The paladin's layout (`Assemble` in `ArpgTree.cpp`), with the mage's own nodes. Notables teach
the mage's talents at the rank given.

**Fire region (north-west):**

| Kind | Name | Effect |
|---|---|---|
| Arm (×5) | Kindling | +4% Fire damage |
| Notable | Improved Fireball | Talent, rank 5: -0.5 sec Fireball recovery |
| **Keystone** | **Pyromancer** | 40% more Fire damage; Frost and Arcane damage 30% less |
| Arm (×5) | Insight | +5 Intellect |
| Notable | Critical Mass | Talent, rank 3: +6% Fire critical strike chance |
| Notable | Incinerate | Talent, rank 2: +4% Fire Blast and Scorch critical strike chance |
| Arm (×5) | Embers | +4% Area damage |
| Notable | Master of Elements | Talent, rank 3: Fire and Frost critical strikes refund 30% of their mana |
| **Keystone** | **Wildfire** | An Ignited enemy's death spreads its Ignite to 3 within 8 yd; Ignites 20% less |
| Side | Expanse, Ardour | +5% spell area; +5 life on kill and +4 mana on hit |

**Arcane region (north-east):**

| Kind | Name | Effect |
|---|---|---|
| Arm (×5) | Mind | +5 Intellect |
| Notable | Arcane Mind | Talent, rank 5: +10% maximum mana |
| **Keystone** | **Brimming Mind** | Spells: 1% more per 2% of mana over half (25% at most); below half, 15% less |
| Arm (×5) | Focus | +4% Arcane damage |
| Notable | Arcane Concentration | Talent, rank 5: 10% chance on a hit of a free next spell |
| Notable | Arcane Instability | Talent, rank 3: +3% spell damage and critical strike chance |
| Arm (×5) | Meditation | +4 Spirit and +3 mana on hit |
| Notable | Arcane Meditation | Talent, rank 3: 15% of mana regeneration while casting |
| **Keystone** | **Arcane Flux** | Overload bursts 50% more and restore 2% mana each; hits 10% less |
| Side | Expanse, Tempo | +5% spell area; +4% cooldown recovery |

**Frost region (south):**

| Kind | Name | Effect |
|---|---|---|
| Arm (×5) | Rime | +4% Frost damage |
| Notable | Improved Frostbolt | Talent, rank 5: -0.5 sec Frostbolt recovery |
| **Keystone** | **Shatter** | 40% more against frozen enemies (frozen, or rooted as by Frost Nova); 15% less against the rest |
| Arm (×5) | Fortitude | +6 Stamina |
| Notable | Ice Shards | Talent, rank 5: Frost critical strikes 100% more of their bonus |
| Notable | Arctic Reach | Talent, rank 2: +20% Frost range and area |
| Arm (×5) | Ice | +4% armour and +2 Stamina |
| Notable | Frostbite | Talent, rank 3: 15% for chills to freeze |
| **Keystone** | **Permafrost** | Enemies that hit you are chilled, or frozen if chilled (each once every 3 sec); Fire damage 25% less |
| Side | Endurance, Stride | +8 life on kill; +3% movement speed |

**Between the regions:** three "Blink" nodes (+3% movement speed) near the start, and the bridges
Spellfire (Fire–Arcane: +8% Fire and Arcane damage), Frost Channeling (Arcane–Frost: talent,
rank 3, Frost spells 15% less mana) and Frostfire (Frost–Fire: +8% Frost and Fire damage).

### Example mage builds at 60

- **Pyromancer (Fireball, Fire Blast, Storms).** Twin Fireball and Pyroclasm clear packs,
  Detonate turns every Ignite into a burst, Meteor drops Flamestrike instantly. Web: Pyromancer
  and Wildfire, Critical Mass.
- **Shatter (Frostbolt, Storms, Arcane Explosion).** Frost Nova or Deep Freeze, then Frostbolt's
  Shatterstorm; Glacial Storm keeps packs chilled and freezing. Web: Shatter, Ice Shards,
  Permafrost for a mage who lets them come.
- **Arcanist (Arcane Missiles, Arcane Explosion, Fire Blast).** Detonation and Unstable Missiles
  Overload everything; Overcharge and Arcane Flux make the bursts the damage and the mana.
  Web: Arcane Flux, Arcane Mind, Brimming Mind with a deep pool.

## The druid, in full

### Identity

The third class, and the shapeshifter. Three ways to play that pull apart: **Balance** (Wrath,
Starfire, Moonfire, Hurricane: Nature that Poisons, Arcane that Overloads), **Feral** (the cat's
Bleeds and finishers, the bear's Maul and Swipe) and **Restoration** (heals that hurt, Thorns,
roots). As with the mage, every skill tree leans into an ailment: Poison for Balance, Bleed for
the cat, Overload for Starfire and Moonfire.

**What an ARPG druid gets beyond the stock class:**
- **Bear Form and Swipe at level 10**, free (Bear Form is a quest in vanilla, Swipe a level 16
  spell): a druid without area damage can't play an ARPG, as the paladin's Consecration at 6.
- **Hurricane at level 20**, free (40 in vanilla), for the same reason on the caster's side.
- **Combo points are the druid's, not the target's.** Points carry from one enemy to the next and
  outlive the enemy they were built on, so a finisher always has its points whichever enemy is
  under the cursor (cmangos `Unit::AddComboPoints`, `Spell::CheckCast`). The HUD shows them as
  five pips above the power orb. A spread copy of a Rake or a Rip (Spreading Rake, Bloodletting,
  the uniques) adds no points and spends none, and a copied Rip strikes with the points held.
- Talent-granted spells (Insect Swarm, Moonkin Form, Swiftmend…) come free at their row's
  level, as for every class.

### Druid skills that can be specialised

Six skills for five slots. The hooks are in `ArpgDruid.cpp`.

| Skill | Covers | Branches | Role |
|---|---|---|---|
| **Wrath** | Wrath, Starfire | Volley, Starfall, Grace | The bolts: split, chain, Starfire bursts |
| **Moonfire** | Moonfire, Insect Swarm | Lunar, Swarm, Moonglow | Damage over time that spreads |
| **Hurricane** | Hurricane | Gale, Squall, Hail | The storm |
| **Bear** | Maul, Swipe, Demoralizing Roar | Maul, Swipe, Guardian | The bear: cleave and endure |
| **Cat** | Claw, Shred, Rake, Rip, Ferocious Bite | Claws, Bleed, Finisher | The cat: Bleeds and finishers |
| **Rejuvenation** | Rejuvenation, Regrowth, Healing Touch, Entangling Roots | Bloom, Wild, Roots | Heals that hurt, and roots |

#### Wrath

| Branch | Node | Ranks | Effect |
|---|---|---|---|
| Volley | Splitting Wrath [A] | 3 | 1/1/2 extra bolts, at 45/60/60% |
| Volley | Quick Wrath [M] | 3 | -0.15 sec Wrath recovery per rank |
| Volley | Thorned Wrath [Ail] | 3 | 8% per rank to Poison |
| Volley | **Wrath of Cenarius** [A] | 1 | Wrath chains to 3 more within 10 yd, at 60% |
| Starfall | Lunar Fury [M] | 4 | 6% more Starfire damage per rank |
| Starfall | Quick Starfire [M] | 3 | -0.2 sec Starfire recovery per rank |
| Starfall | Moonstruck [Ail] | 3 | 10% per rank to Overload |
| Starfall | **Starfall** [A] | 1 | Starfire bursts within 10 yd, at 60% |
| Grace | Nature's Grace [M] | 3 | +3% critical strike chance per rank |
| Grace | Moonfury [M] | 3 | Critical strikes 20% more of their bonus per rank |
| Grace | **Lunar Eclipse** [Syn] | 1 | A critical Wrath makes the next Starfire instant, bursting within 6 yd for 40% |
| Grace | **Eclipse** [H] | 1 | Critical Wraths and Starfires restore 2% of maximum mana and make the next cast 0.5 sec quicker |

#### Moonfire

Moonfire and Insect Swarm do most of their damage as ticks, so the Swarm and Moonglow capstones
are hooks on their ticks (`OnDruidTick`, called from `SpellAuras.cpp`).

| Branch | Node | Ranks | Effect |
|---|---|---|---|
| Lunar | Spreading Moon [F] | 3 | Moonfire also lands on 1/2/3 more within 8 yd |
| Lunar | Lunar Burn [M] | 3 | 15% more Moonfire burn per rank |
| Lunar | **Moonkissed** [Syn] | 1 | A critical Wrath or Starfire makes the next Moonfire also land on 3 more within 8 yd |
| Lunar | **Full Moon** [F] | 1 | Moonfire spreads to every enemy within 10 yd when its target dies |
| Lunar | Moonlit Wrath [More] | 3 | 8% more Moonfire damage per rank (after Full Moon) |
| Swarm | Plague [F] | 2 | Insect Swarm also lands on 1/2 more within 8 yd |
| Swarm | Locusts [Ail] | 3 | Moonfire's hits 10% per rank to Poison |
| Swarm | Swarm Fury [M] | 3 | 10% more Insect Swarm damage per rank |
| Swarm | **Hive Mind** [H] | 1 | Every Insect Swarm tick Poisons its target |
| Moonglow | Moonglow [M] | 3 | 10% less mana cost per rank |
| Moonglow | Long Night [M] | 2 | +3 sec duration per rank |
| Moonglow | **Lunar Tide** [H] | 1 | Every Moonfire and Insect Swarm tick restores 0.5% of maximum mana (2% a second at most) |

#### Hurricane

| Branch | Node | Ranks | Effect |
|---|---|---|---|
| Gale | Wide Hurricane [M] | 3 | +12% radius per rank |
| Gale | Howling Gale [M] | 4 | 8% more damage per rank |
| Gale | **Gathering Storm** [Syn] | 1 | A critical Moonfire or Starfire makes the next Hurricane free |
| Gale | **Tempest** [H] | 1 | Each strike has 10% to hurl a Wrath at the enemy struck, for three times the strike (2 a second at most) |
| Hail | Grasping Winds [M] | 2 | +2 sec duration per rank |
| Hail | Biting Hail [H] | 3 | Each strike has 5% per rank to chill |
| Hail | Eye of the Wild [H] | 3 | Each enemy struck heals you 0.3% of maximum health per rank (3% a second at most) |
| Hail | **Thunderhead** [M] | 1 | Hurricane costs half the mana |
| Squall | Stinging Hail [Ail] | 3 | Each strike 6% per rank to Poison |
| Squall | Gale Force [More] | 4 | 6% more Hurricane damage per rank |
| Squall | Thrifty Gale [M] | 3 | 10% less mana cost per rank |

#### Bear

| Branch | Node | Ranks | Effect |
|---|---|---|---|
| Maul | Brutal Maul [M] | 4 | 8% more Maul damage per rank |
| Maul | Mauling Arc [G] | 3 | Maul strikes every enemy in front, the extras at 20/35/50% |
| Maul | Savage Maul [Ail] | 3 | 10% per rank to Bleed |
| Maul | **Earthshaker** [H] | 1 | Maul sends a shockwave 10 yd ahead, at 60% |
| Swipe | **Roaring Swipe** [Syn] | 1 | Demoralizing Roar makes the next Swipe deal 60% more |
| Swipe | Wide Swipe [M] | 2 | +15% Swipe area per rank |
| Swipe | **Ravaging Swipe** [M] | 1 | Swipe strikes up to 10 enemies (3 in vanilla) |
| Swipe | Ursine Might [More] | 4 | 6% more Bear damage per rank |
| Guardian | Bristling Fur [H] | 3 | In Bear Form, each blow taken gives 2 rage per rank (once a second) |
| Guardian | Survival Instincts [H] | 3 | Bear hits heal you for 2% of their damage per rank |
| Guardian | **Thorned Hide** [H] | 1 | In Bear Form, enemies that hit you take 30% of your attack power as Nature (once a second each) |

#### Cat

| Branch | Node | Ranks | Effect |
|---|---|---|---|
| Claws | Sharpened Claws [M] | 4 | 6% more Cat damage per rank |
| Claws | Raking Claws [G] | 3 | Claw strikes every enemy in front, the extras at 20/35/50% |
| Claws | Light Paws [M] | 2 | Cat abilities cost 5 less energy per rank |
| Claws | **Stalker's Step** [K] | 1 | A kill with Shred steps you behind the nearest enemy within 10 yd |
| Bleed | Deep Wounds [M] | 3 | 15% more Rake and Rip damage per rank |
| Bleed | Rending Claws [Ail] | 3 | 8% per rank to Bleed |
| Bleed | Spreading Rake [F] | 2 | Rake also lands on 1/2 more within 8 yd |
| Bleed | **Bloodletting** [F] | 1 | Rip also lands on 4 more within 8 yd |
| Finisher | **Bloodied Bite** [Syn] | 1 | A critical Claw or Shred makes the next Ferocious Bite a sure critical strike |
| Finisher | Savagery [M] | 3 | +4% critical strike chance per rank |
| Finisher | Savage Fury [H] | 1 | Critical Claws, Shreds and Rakes give an extra combo point |
| Finisher | **Feast** [G] | 1 | Ferocious Bite strikes every enemy in front, at 60% |

#### Rejuvenation

Like the paladin's Holy Light: heals, which the tree turns on enemies too.

| Branch | Node | Ranks | Effect |
|---|---|---|---|
| Bloom | Lush Growth [M] | 4 | 6% more Regrowth and Healing Touch healing per rank |
| Bloom | Living Bloom [M] | 3 | Rejuvenation and Regrowth heal 10% more over time per rank |
| Bloom | **Overgrowth's Gift** [Syn] | 1 | Entangling Roots makes the next Regrowth or Healing Touch instant |
| Bloom | **Bloom** [H] | 1 | Your heals burst: every enemy within 8 yd takes 50% of the heal as Nature |
| Wild | Nature's Wrath [H] | 3 | Your heals strike the nearest enemy within 20 yd for 15% of the heal per rank, as Nature |
| Wild | Verdant Fury [H] | 2 | Healing yourself gives 5% more damage per rank for 5 sec |
| Wild | **Wrath of the Wild** [H] | 1 | Your heals Poison every enemy within 8 yd, a hit of 30% of the heal (once every 3 sec) |
| Roots | Living Root [F] | 2 | Entangling Roots also roots 1/2 more within 6 yd |
| Roots | Grasping Vines [M] | 3 | 20% more Entangling Roots damage per rank |
| Roots | **Overgrowth** [F] | 1 | Entangling Roots roots every enemy within 10 yd of its target |
| Wild | Thrift of the Grove [M] | 3 | 10% less mana cost per rank (after Wrath of the Wild) |
| Roots | Thorny Vines [More] | 3 | 10% more Entangling Roots damage per rank (after Overgrowth) |

#### Druid Codex pages

| Capstone | Skill | Home boss |
|---|---|---|
| Wrath of Cenarius | Wrath | Alzzin the Wildshaper, Dire Maul |
| Starfall | Wrath | Shade of Eranikus, the Temple of Atal'Hakkar |
| Eclipse | Wrath | Princess Theradras, Maraudon |
| Full Moon | Moonfire | Twilight Lord Kelris, Blackfathom Deeps |
| Hive Mind | Moonfire | Noxxion, Maraudon |
| Lunar Tide | Moonfire | Aku'mai, Blackfathom Deeps |
| Tempest | Hurricane | Mutanus the Devourer, Wailing Caverns |
| Thunderhead | Hurricane | Lord Serpentis, Wailing Caverns |
| Earthshaker | Bear | Charlga Razorflank, Razorfen Kraul |
| Ravaging Swipe | Bear | Tendris Warpwood, Dire Maul |
| Thorned Hide | Bear | Celebras the Cursed, Maraudon |
| Stalker's Step | Cat | Lord Cobrahn, Wailing Caverns |
| Bloodletting | Cat | Herod, Scarlet Monastery |
| Feast | Cat | King Gordok, Dire Maul |
| Bloom | Rejuvenation | Verdan the Everliving, Wailing Caverns |
| Wrath of the Wild | Rejuvenation | Lady Anacondra, Wailing Caverns |
| Overgrowth | Rejuvenation | Lord Pythas, Wailing Caverns |

**Runes** fit the druid's skills too: Chains (Wrath, Moonfire), Shattering (Wrath, Moonfire, Bear,
Cat), Expanse (Hurricane, Bear), Lingering (Moonfire, Hurricane, Rejuvenation), Leech (all but
Rejuvenation) and Sanctity (all).

Thorns, Faerie Fire, the forms' other abilities (Tiger's Fury, Dash, Bash, Pounce...) and the
travel forms stay out of the trees: castable from the spellbook, fed by the web and gear.

### The druid passive web

The same layout as the others, with the druid's nodes. Notables teach the druid's talents.

**Balance region (north-west):**

| Kind | Name | Effect |
|---|---|---|
| Arm (×5) | Starlight | +4% Arcane damage |
| Notable | Improved Wrath | Talent, rank 5: -0.5 sec Wrath recovery |
| **Keystone** | **Celestial Focus** | 30% more Arcane and Nature spell damage; your attacks 40% less |
| Arm (×5) | Insight | +5 Intellect |
| Notable | Moonglow | Talent, rank 3: 9% less mana cost for Moonfire, Starfire, Wrath and the heals |
| Notable | Improved Moonfire | Talent, rank 5: +10% Moonfire damage and critical strike chance |
| Arm (×5) | Growth | +4% Nature damage |
| Notable | Nature's Grace | Talent: a spell critical strike makes the next cast 0.5 sec quicker |
| **Keystone** | **Nature's Bounty** | Your Poisons deal 50% more; your hits 15% less |
| Side | Expanse, Moonwell | +5% spell area; +4 Spirit and +3 mana on hit |

**Feral region (north-east):**

| Kind | Name | Effect |
|---|---|---|
| Arm (×5) | Ferocity | +4% Melee damage |
| Notable | Sharpened Claws | Talent, rank 3: +6% critical strike chance in Bear and Cat Form |
| **Keystone** | **Predator** | Your Bleeds deal 60% more; your spells 30% less |
| Arm (×5) | Hide | +6 Stamina |
| Notable | Thick Hide | Talent, rank 5: +10% armour from items |
| Notable | Heart of the Wild | Talent, rank 5: +20% Intellect; more Stamina in Bear Form, attack power in Cat Form |
| Arm (×5) | Claws | +5 Agility |
| Notable | Predatory Strikes | Talent, rank 3: attack power in the forms, 150% of your level |
| **Keystone** | **King of the Jungle** | 4% more Melee damage per kill in the last 6 sec (10 at most); your heals 50% less |
| Side | Swiftness, Wildness | +3% movement speed; +8 life on kill |

**Restoration region (south):**

| Kind | Name | Effect |
|---|---|---|
| Arm (×5) | Blossom | +4% healing |
| Notable | Improved Rejuvenation | Talent, rank 3: 15% more Rejuvenation |
| **Keystone** | **Tree of Life** | Your heals 50% more; your damage 20% less |
| Arm (×5) | Spirit | +4 Spirit |
| Notable | Furor | Talent, rank 5: rage or energy on shifting |
| Notable | Improved Healing Touch | Talent, rank 5: -0.5 sec Healing Touch recovery |
| Arm (×5) | Bark | +4% armour and +2 Stamina |
| Notable | Improved Thorns | Talent, rank 3: 75% more Thorns damage |
| **Keystone** | **Living Thorns** | Thorns on you strikes every enemy within 8 yd each second for its damage; your heals 30% less |
| Side | Endurance, Tranquility | +8 life on kill; +4% cooldown recovery |

**Between the regions:** three "Wild" nodes (+3% movement speed) near the start, and the bridges
Natural Weapons (Balance–Feral: talent, rank 5, +10% melee damage), Leader of the Pack
(Feral–Restoration: talent, +3% critical strike chance for the party in the forms) and Gift of
Nature (Restoration–Balance: talent, rank 5, +10% healing).

### Example druid builds at 60

- **Moonkin (Wrath, Moonfire, Hurricane).** Splitting Wrath and Wrath of Cenarius chain through
  packs, Thorned Wrath and Hive Mind stack Poison, Full Moon carries Moonfire from body to body.
  Web: Celestial Focus and Nature's Bounty.
- **Feral cat (Cat, Bear, Rejuvenation).** Rake and Rip spread with Spreading Rake and
  Bloodletting, points carry between kills, Feast finishes a pack. Web: Predator and King of
  the Jungle.
- **Thornwarden (Bear, Rejuvenation, Hurricane).** Ravaging Swipe and Earthshaker for clearing,
  Thorned Hide and Living Thorns punish everything that touches you, Bloom turns each heal into
  a burst. Web: Living Thorns, Thick Hide, Tree of Life.

## The warlock, in full

Built (server `Arpg/ArpgWarlock.{h,cpp}`, `Arpg/ArpgMinions.{h,cpp}`), untested in game. The
research behind the minion design is in `ARPG-DECISIONS.md` ("Minions").

### Identity

The fourth class, and the one that gets others to do its killing. It has three ways to play:

- **Affliction:** Corruption, Curse of Agony, Siphon Life and the drains. Shadow damage over time
  that Withers and spreads from body to body.
- **Destruction:** Shadow Bolt, Searing Pain, Immolate, Conflagrate, Rain of Fire and Hellfire.
  Fire that Ignites and pays it off, plus a bolt that pierces.
- **Demonology:** a demon at your side and a horde of lesser ones around it. **A full minion build**
  is possible: the demons do most of the damage, and the warlock commands, curses and keeps them
  fed.

**What an ARPG warlock gets beyond the stock class:**
- **No reagents.** An ARPG player's own class spells need none (no Soul Shard, Infernal Stone or
  Demonic Figurine, `Player::CanNoReagentCast`; the client mirrors it).
- **Its summons cast at once,** with a flat 1.5 sec recovery: the demons, Inferno and Ritual of
  Doom (`Arpg::IsArpgSummon`).
- **Its demons without their quests,** and its big spells sooner:

  | Spell | ARPG level | Vanilla |
  |---|---|---|
  | Summon Voidwalker | 10 | a quest |
  | Rain of Fire | 10 | 20 |
  | Summon Succubus | 20 | a quest |
  | Hellfire | 20 | 30 |
  | Inferno | 20 | 50, and a stone |
  | Summon Felhunter | 30 | a quest |
  | Ritual of Doom | 40 | 60, and a party |

- **Inferno and Ritual of Doom go into the horde.**
  - Inferno's Infernal joins the horde for 60 sec and never turns on you: no Enslave Demon.
  - Ritual of Doom summons its Doomguard beside you for 60 sec: no portal, no party, no sacrifice.
  - Their cooldowns are 1 min and 3 min, not an hour.
- **Life Tap** stays as the resource trick: health into mana. **Searing Pain** is the free filler,
  giving 3% mana back on a hit, from level 1 (vanilla's 18: until then the warlock spent 30% of
  its fights under a fifth of its mana).
- **Fel Ward** (its defence, 2026-10-21): every kill, its own or a minion's, gives Barrier of 5%
  of maximum health (8% under Demon Skin or Demon Armor), so a warlock in the thick of packs
  grows a shield as it kills. With minions drawing 4 times their damage in threat, it answers the
  2026-10-19 deaths (11.8 an hour, all attrition from melee packs).
- Talent spells come free at their row's level, as for every class: Siphon Life, Conflagrate,
  Shadowburn, Fel Domination, Soul Link and the others.

### Minions: the rules

Every minion in the game follows these rules: the warlock's demons, its horde, and Book of the
Dead's skeletons (`Arpg/ArpgMinions.h`).

- **Two kinds.**
  - **Your demon:** one at a time, as in vanilla. It persists, follows you, and has a skill
    tree of its own.
  - **The horde:** timed minions: imps from kills (20 sec), the Infernal, the Doomguard, risen
    skeletons.
    - Its size is 5 at base, more with gear's "+N to the horde's size" and Imp Swarm, and 15 at most.
    - When it's full, the oldest goes.
- **Minions hit with your numbers.**
  - A minion's hit is tagged **Minion** beside its own tags.
  - Your increased and more bonuses apply wherever the tags match: "+% Fire damage" helps an Imp,
    "+% Minion damage" helps them all, and Intellect helps their spells.
  - The enemy's states and ailments count as they do for your own hits.
  - Its ailments, Daze and synergy triggers are yours, so the synergy layer works through it.
- **Each demon carries its own state:**

  | Demon | Its state |
  |---|---|
  | Imp | Ignites (15% a hit) |
  | Succubus | Withers (15% a hit) |
  | Felhunter | Overloads (15% a hit) |
  | Voidwalker | builds Daze with its physical blows |

- **Horde imps** throw a Firebolt every 2 sec.
- **They hold the enemy's eye:** a minion's hit is worth 4 times its damage in threat, so a pack
  turns on the demons and not on the warlock behind them.
- **Your demon comes back with you:** dying with a demon out, it returns as you rise at your
  checkpoint (Fel Domination brings it back after its own death in a fight).
- **Their own stats,** from a new affix pool (frozen like the others, so old items keep theirs):
  - +% Minion damage;
  - +% Minion life;
  - +% Minion attack and cast speed;
  - +N to the horde's size (weapons and amulets);
  - minions' life leech.

  The web's Demonology side carries the same stats.
- **They fight where you do.** There's no pet bar and no target. Minions attack:
  1. the enemy you last struck or cast at, for 5 sec;
  2. otherwise, the nearest enemy within 15 yards of you.

  They let go of anything more than 25 yards from you, and teleport back to you past 40.
- **Commands keep you busy** (the Demon skill's tree):
  - **Torment:** your curses send every minion at the cursed enemy, dealing it 20% more for 4 sec.
  - **Sacrifice:** Life Tap detonates the oldest horde minion for Fire damage of a quarter of its
    life within 8 yards.
  - **Soul Link:** your demon takes a share of the damage you take.
- **On the HUD:** the demon's health as a bar with its name over the health orb, and "Horde N/M"
  under it (`SMSG_ARPG_MINIONS`, 0x354).
- **In the statistics:** each minion's damage is its own line ("Imp (minion): Firebolt"), and the
  session line gives `minionPct`, the minions' share of the damage.

### Warlock skills that can be specialised

Six skills for five slots.

| Skill | Covers | Branches | Role |
|---|---|---|---|
| **Shadow Bolt** | Shadow Bolt, Searing Pain, Shadowburn | Volley, Nightfall, Ruin | The bolt: split, pierce, crits |
| **Corruption** | Corruption, Curse of Agony, Siphon Life | Plague, Agony, Siphon | Shadow over time that spreads |
| **Immolate** | Immolate, Conflagrate, Rain of Fire, Hellfire | Pyre, Cataclysm, Hellfire | Fire: Ignite, its payoff, the ground |
| **Drain** | Drain Life, Drain Soul, Death Coil, Life Tap | Siphon, Reaping, Pact | Sustain and execute |
| **Demon** | the summons and every demon ability | Brute, Seduction, Command | The one at your side |
| **Legion** | Inferno, Ritual of Doom | Imps, Infernal, Doom | The horde |

#### Shadow Bolt

| Branch | Node | Ranks | Effect |
|---|---|---|---|
| Volley | Splitting Bolt [A] | 3 | 1/1/2 extra bolts, at 45/60/60% |
| Volley | Piercing Shadow [A] | 3 | Flies through 1/2/4 enemies, at 60% |
| Volley | Withering Bolt [Ail] | 3 | 8% per rank to Wither |
| Volley | **Shadowstorm** [A] | 1 | Bursts within 6 yd of its target, at 50% |
| Nightfall | Nightfall [Syn] | 3 | Corruption and Agony ticks: 2% per rank to loose a free Shadow Bolt |
| Nightfall | Quick Bolt [M] | 3 | -0.15 sec Shadow Bolt recovery per rank |
| Nightfall | Shadow Trance [Syn] | 1 | Shadow Bolt consumes Withered for 100% of the hit and the stacks |
| Ruin | Devastation [M] | 3 | +3% critical strike chance per rank |
| Ruin | Ruin [M] | 3 | Critical strikes deal 20% more of their bonus per rank |
| Ruin | Searing Pain [M] | 3 | 10% more Searing Pain damage per rank |
| Ruin | **Shadowburn** [H] | 1 | A kill with Shadowburn readies it again |

#### Corruption

| Branch | Node | Ranks | Effect |
|---|---|---|---|
| Plague | Spreading Corruption [F] | 3 | Also lands on 1/2/3 more within 8 yd |
| Plague | Rot [M] | 3 | 15% more Corruption damage per rank |
| Plague | **Pandemic** [F] | 1 | Spreads to every enemy within 10 yd when its target dies |
| Agony | Lingering Curse [F] | 2 | Curse of Agony also lands on 1/2 more within 8 yd |
| Agony | Tormented Soul [Syn] | 3 | Agony's ticks: 10% per rank to Wither |
| Agony | **Haunting Agony** [H] | 1 | Curse of Agony deals 100% more to Withered enemies |
| Siphon | Siphoning [M] | 3 | 15% more Siphon Life damage (and healing) per rank |
| Siphon | Grim Reach [M] | 2 | +3 sec duration on all three per rank |
| Siphon | **Soul Feast** [H] | 1 | Every damage over time tick heals 0.3% of maximum health, 3% a second at most |
| Plague | Blight [More] | 4 | 6% more Corruption skill damage per rank (after Pandemic) |
| Agony | Anguish [More] | 3 | 10% more Curse of Agony damage per rank (after Haunting Agony) |
| Siphon | Dark Thrift [M] | 2 | 15% less mana cost per rank (after Soul Feast) |

#### Immolate

| Branch | Node | Ranks | Effect |
|---|---|---|---|
| Pyre | Spreading Flame [F] | 2 | Immolate also lands on 1/2 more within 8 yd |
| Pyre | Emberstorm [M] | 4 | 6% more damage per rank |
| Pyre | Conflagration [Syn] | 1 | Conflagrate consumes Ignite to burst within 8 yd for the hit and the burn left |
| Cataclysm | Wide Rain [M] | 3 | +12% Rain of Fire radius per rank |
| Cataclysm | Burning Rain [H] | 3 | Each Rain of Fire wave: 10% per rank to Ignite |
| Cataclysm | **Firestorm** [H] | 1 | Rain of Fire strikes Ignited enemies twice |
| Hellfire | Hellfire Ward [M] | 3 | 25% less self-damage from Hellfire per rank |
| Hellfire | Infernal Heat [Syn] | 3 | Each Hellfire tick: your minions deal 5% more per rank for 4 sec |
| Hellfire | **Backlash** [H] | 1 | 20% less damage taken while you channel Hellfire |
| Pyre | Fel Kindling [M] | 3 | 10% less mana cost per rank (after Conflagration) |
| Cataclysm | Downpour [More] | 3 | 10% more Rain of Fire damage per rank (after Firestorm) |

#### Drain

| Branch | Node | Ranks | Effect |
|---|---|---|---|
| Siphon | Grim Drain [M] | 4 | 10% more Drain Life damage per rank |
| Siphon | Chained Drain [F] | 2 | Drain Life also drains 1/2 more within 10 yd |
| Siphon | **Life Link** [H] | 1 | Drain Life's ticks heal your minions for 50% of their damage |
| Reaping | Soul Reaper [M] | 3 | Drain Soul 30% more per rank below 25% health |
| Reaping | **Reaping** [H] | 1 | A kill under your Drain Soul raises an imp and restores 5% of your mana |
| Pact | Dark Pact [M] | 3 | Life Tap gives 1% more of your maximum mana per rank |
| Pact | Death's Grasp [M] | 2 | -10 sec Death Coil cooldown per rank |
| Pact | **Demonic Pact** [H] | 1 | Life Tap also heals you 5% of your maximum health, taken from your demon |
| Siphon | Essence Drain [More] | 4 | 6% more Drain skill damage per rank (after Life Link) |
| Reaping | Soul Hunger [More] | 3 | 10% more Drain Soul damage per rank (after Reaping) |
| Pact | Dark Bargain [M] | 3 | 10% less Drain skill mana cost per rank (after Demonic Pact) |

#### Demon

| Branch | Node | Ranks | Effect |
|---|---|---|---|
| Brute | Fel Vitality [M] | 4 | Your demon has 10% more life per rank |
| Brute | Demonic Embrace [M] | 3 | Your demon deals 8% more per rank |
| Brute | Cleaving Demon [G] | 3 | Your demon's melee strikes the others beside its target, at 30/45/60% |
| Brute | **Fel Domination** [H] | 1 | When your demon dies it is summoned again at once (once every 30 sec) |
| Seduction | Lash Arc [G] | 2 | Lash of Pain and Firebolt strike 1/2 more within 6 yd, at 50% |
| Seduction | Demonic Mastery [M] | 3 | While your demon lives, you deal 4% more per rank |
| Seduction | **Grand Pact** [M] | 1 | Your demon has 40% more life and deals 25% more |
| Command | Torment [Syn] | 1 | Your curses command every minion at the cursed enemy, 20% more to it for 4 sec |
| Command | Sacrifice [Syn] | 1 | Life Tap detonates your oldest horde minion |
| Command | Soul Link [M] | 3 | 10% per rank of the damage you take goes to your demon |
| Command | **Demonic Sacrifice** [K] | 1 | With no demon at your side, you deal 30% more: the build without minions |

#### Legion

| Branch | Node | Ranks | Effect |
|---|---|---|---|
| Imps | Imp Gate [H] | 3 | Kills have 25% per rank to raise an imp for 20 sec |
| Imps | Imp Swarm [M] | 2 | The horde holds 2 more per rank |
| Imps | Fiery Swarm [Ail] | 3 | Horde imps' Firebolts: 10% per rank to Ignite |
| Imps | **Burning Legion** [H] | 1 | A horde minion's kill raises another imp |
| Infernal | Infernal Fall [H] | 3 | The landing stuns and strikes within 8/10/12 yd |
| Infernal | Immolation Aura [M] | 3 | Infernals deal 20% more per rank |
| Infernal | **Lord of Flame** [H] | 1 | Inferno calls down two Infernals |
| Doom | Ritual Haste [M] | 2 | -60 sec Ritual of Doom cooldown per rank |
| Doom | Doombringer [M] | 3 | Your Doomguard deals 15% more per rank |
| Doom | **Pit Lord** [H] | 1 | Your Doomguard stays 10 min |

Capstones' Codex pages: Arugal, Thaurissan, Amnennar, Jandice Barov, Gandling, Incendius,
Emberseer, Balnazzar, Rivendare, Tortheldrin, Taragaman, Immol'thar, the Shade of Eranikus,
Archaedas, Wyrmthalak and Rend (`ArpgSkills.cpp`, `PAGES`).

### The warlock passive web

The same shape as the others. The Affliction, Destruction and Demonology talents are its notables,
with two keystones on each side.

| Keystone | Side | Effect |
|---|---|---|
| **Plague Lord** | Affliction | Damage over time 40% more; direct hits 20% less |
| **Soul Siphon** | Affliction | Drains heal 2% more per damage over time of yours on the target (10 at most); direct hits 15% less |
| **Hellstorm** | Destruction | Rain of Fire and Hellfire stun on a crit (once every 4 sec an enemy); damage over time 20% less |
| **Ruinous Power** | Destruction | Shadow Bolt and Searing Pain 30% more; minions 30% less |
| **Legion** | Demonology | Every kill raises an imp for 20 sec; your own spells 15% less |
| **Demonic Knowledge** | Demonology | Minions 30% more damage and life; you 20% less |

Notables teach the talents:
- Affliction: Improved Corruption, Shadow Mastery, Nightfall, Improved Drain Life;
- Destruction: Emberstorm, Improved Shadow Bolt, Devastation, Bane;
- Demonology: Master Summoner, Fel Intellect, Unholy Power, Demonic Embrace;
- the bridges: Cataclysm, Fel Stamina, Grim Reach.

The Demonology side's smalls carry +% Minion damage, +% Minion life, and +% Minion attack and cast
speed.

### The warlock's synergy (`ARPG-SYNERGY.md`)

- *Feast of Souls:* Death Coil consumes Withered for an extra hit of 100% and the stacks.
- *Contagion:* a Withered enemy's death Withers the 2 nearest.
- *Fel Bond:* a minion's kill of a Withered enemy heals you 2% of your maximum health.
- The nodes Shadow Trance (Shadow Bolt cashes Withered) and Conflagration (Conflagrate bursts
  Ignite).
- Some pairings these make:
  - the Succubus Withers and Shadow Bolt cashes it;
  - the Imp Ignites and Conflagrate bursts it;
  - the Voidwalker builds Daze for a Daze unique.

### Example warlock builds at 60

- **Demonologist, the minion build (Demon, Legion, Corruption, Drain, Immolate):**
  - The imps and the Infernal do most of the work. Imp Gate, Legion and Burning Legion keep the
    horde full.
  - Torment on a curse points it, and Corruption keeps everything Withered for the Succubus's
    payoffs.
  - Life Link heals the horde.
  - Web: Legion, Demonic Knowledge. Gear: minion damage and horde size.
  - **Target:** about 70% of the damage from minions (`minionPct` in the statistics).
- **Plaguebringer (Corruption, Drain, Shadow Bolt):** Spreading Corruption, Pandemic and
  Contagion carry the rot. Tormented Soul Withers, Shadow Trance and Feast of Souls cash it, and
  Soul Feast and the drains sustain. Web: Plague Lord, Soul Siphon.
- **Destroyer (Shadow Bolt, Immolate, Demon for Demonic Sacrifice):** Splitting and Piercing Bolt
  clear, Rain of Fire and Firestorm hold the ground, and Conflagration bursts the Ignites. No
  demon. Web: Hellstorm, Ruinous Power.

### How it is built

- **`Arpg/ArpgMinions.{h,cpp}`, for every class:**
  - the horde per player;
  - targeting and the leash;
  - horde imps' Firebolts;
  - the pet's life and speed modifiers;
  - the HUD packet.
- **Minion hits:** `Arpg::DamageModForTags` gives them the owner's bonuses (`MinionDamageMod` for
  their own more). `Arpg::OnHitLanded` gives the owner their ailments, Daze and triggers.
- **`Arpg/ArpgWarlock.{h,cpp}`, the hooks:**
  - hits, ticks (periodic damage and leech) and kills (the owner's, and a minion's own from
    `Unit::Kill`);
  - casts at spell go;
  - Ritual of Doom's effect veto (`Spell::HandleEffect`) and Inferno's Infernal
    (`spell_inferno_warlock_summon`);
  - Soul Link, Hellfire Ward and Backlash in `Unit::DealDamage`;
  - Fel Domination.
- The skills, the web, the minion affixes, the costs and the class's cooldown rules live with the
  other classes'.
- **Built after:** the four warlock sets (above, "Sets") and minion damage numbers (quieter than
  your own: smaller, dimmer, no crit pop). **Not yet:** warlock uniques' rows beyond Book of the
  Dead.

## The shaman, in full

Built (2026-10-22): `Arpg/ArpgShaman.{h,cpp}` on the server, the trees in `ArpgSkills.cpp`, the
web in `ArpgTree.cpp`. The fifth and last class.

### Identity

The elements' caller, with three ways to play that pull apart:

- **Stormcaller:** Lightning Bolt and Chain Lightning. Lightning jumps from body to body and
  leaves enemies **Shocked**.
- **Earthwarden:** totems that fight. Searing, Magma and Fire Nova Totems are the class's second
  minion build: stationary minions, where the warlock's follow you (Walking Totems makes them
  follow too).
- **Enhancer:** a weapon and its imbues. Windfury's extra strikes, Stormstrike, and Flametongue
  or Frostbrand turning the swing into Fire or Frost. It uses the melee rules ("Melee": pace,
  cleave, Fortify, life on hit).

### A new state: Shocked

Lightning has no ailment of its own: Nature's is Poison, which suits the druid's swarms but not a
storm. So the shaman has one (`Arpg/ArpgStates.h`, the 16th state and the last bit of the `u16`
mask, after Vulnerable):

- **Shocked** comes from the shaman's Lightning hits: Lightning Bolt, Chain Lightning, Stormstrike,
  and Lightning Shield's discharge with Charged Shield. It is a class rule, not the Nature tag, so
  a druid's Nature still Poisons.
- A Shocked enemy takes 5% more damage from everyone per stack, up to 4 stacks (8 under
  Overcharge), for 4 sec from the last (3 under Overcharge).
- **Consumed** by Earth Shock for an extra hit of 25% of its hit per stack.
- The hover bar shows it, and the enemy wears Lightning Shield's orbs.

### The class's own rules

- *Shock:* Lightning hits Shock (above).
- *Earth Shock* consumes Shocked.
- *Static Discharge:* a Shocked enemy's death Shocks the 2 nearest.
- *Elemental Crossing:* Frost Shock on an Ignited enemy, or Flame Shock on a Chilled one,
  strikes again for 50%: the mage's Thermal Shock, the shaman's way.
- *The free filler:* Lightning Bolt costs nothing, gives 4% mana back on a hit and makes its
  target Vulnerable, as the other classes' fillers; Chain Lightning (16%) is the spender.

### What an ARPG shaman gets beyond the stock class

- **No totem items.** The Earth, Fire, Water and Air Totems are quest items in vanilla. An ARPG
  shaman's own class spells need none, as no class needs reagents (`Player::CanNoReagentCast`;
  `Spell::CheckItems` on the server, `reagent_totem_refusal` and the button greying on the client).
- **Area spells sooner:**

  | Spell | ARPG level | Vanilla |
  |---|---|---|
  | Fire Nova Totem | 10 | 12 |
  | Chain Lightning | 20 | 32 |
  | Magma Totem | 20 | 26 |

- **Totems are minions** (`Arpg/ArpgMinions.h`; `MinionOwner` finds them by their owner):
  - their hits carry the Minion and Totem tags, and take your bonuses by tag;
  - their ailments are yours;
  - "+% Minion damage" and the minion affixes count for them;
  - they don't move or leash (Walking Totems aside), one of each element at a time, as in vanilla.
- Talent spells come free at their row's level, as for every class: Stormstrike, Elemental
  Mastery, Mana Tide Totem.
- Totem spells are scored as the totem's, a minion's, through the core's owner redirect
  (`Arpg::TotemCastScope`); totems take minion life too.
- **Not built:** totems placed where you aim (they go up at your feet), Tremor Totem breaking
  stuns, and Totemic Recall.

### Shaman skills that can be specialised

Six skills for five slots. Every tree holds 24-28 ranks against a skill's cap of 20.

| Skill | Covers | Branches | Role |
|---|---|---|---|
| **Lightning** | Lightning Bolt, Chain Lightning, Lightning Shield | Storm, Overload, Conduit | The jumps and the Shock |
| **Shocks** | Earth Shock, Flame Shock, Frost Shock | Earth, Flame, Frost | The payoffs, one per element |
| **Fire Totems** | Searing Totem, Magma Totem, Fire Nova Totem | Searing, Magma, Nova | The totems that fight |
| **Earthen Totems** | Stoneclaw, Earthbind, Strength of Earth, Windfury, Grace of Air, Tremor, Stoneskin | Stone, Wind, Ward | Totems that hold and empower |
| **Strike** | the swing, Stormstrike, the weapon imbues | Windfury, Imbue, Storm | The Enhancer |
| **Healing Wave** | Healing Wave, Lesser Healing Wave, Chain Heal | Tide, Ancestral, Torrent | Heals that hurt, as the paladin's and the druid's |

Flametongue and Frostbrand are a choice: one or the other. Healing Wave's Riptide, Ancestral
Healing, Torrent and Healing Stream use the druid's heal hooks (Nature's Wrath, Verdant Fury,
Bloom, Wrath of the Wild).

#### Lightning

| Branch | Node | Ranks | Effect |
|---|---|---|---|
| Storm | Forked Lightning [A] | 3 | Lightning Bolt chains to 1/2/3 more enemies within 10 yards, at 50% |
| Storm | Quick Bolt [M] | 3 | -0.15 sec Lightning Bolt recovery per rank |
| Storm | Static [H] | 3 | Your Lightning hits have a 10% chance per rank to Shock twice |
| Storm | **Thunderstorm** [A] | 1 | Chain Lightning bursts onto every enemy within 6 yards of each it strikes, at 40% |
| Overload | Lightning Overload [Syn] | 3 | Your Lightning hits have a 5% chance per rank to strike again, at 50% |
| Overload | Concussive Storm [More] | 4 | 6% more Lightning damage per rank |
| Overload | **Lightning Fury** [M] | 1 | Your Lightning's critical strikes deal 100% more of their bonus |
| Conduit | Conduit [H] | 3 | Each Shock consumed restores 1% of your maximum mana per rank |
| Conduit | Charged Shield [H] | 3 | Lightning Shield's discharge Shocks its attacker, and deals 25% more per rank |
| Conduit | **Ball Lightning** [H] | 1 | Each Shocked enemy's death throws a Lightning Bolt at the nearest enemy |
| Conduit | Thrift of the Storm [M] | 3 | 10% less Chain Lightning mana cost per rank |

#### Shocks

| Branch | Node | Ranks | Effect |
|---|---|---|---|
| Earth | Earthquake [Syn] | 1 | When Earth Shock consumes a Shock, every enemy within 6 yards takes the payoff too |
| Earth | Reverberation [M] | 3 | -1 sec Shock cooldown per rank |
| Earth | **Earthen Power** [H] | 1 | Earth Shock Dazes its target |
| Earth | Tectonic Force [More] | 4 | 8% more Earth Shock damage per rank |
| Flame | Searing Flames [Ail] | 3 | Your Shock hits have a 15% chance per rank to Ignite |
| Flame | Flame Spread [A] | 2 | Flame Shock also lands on 1/2 more enemies within 8 yards |
| Flame | **Lava Burst** [H] | 1 | Flame Shock on an Ignited enemy consumes the Ignite to burst onto every enemy within 8 yards for the burn left |
| Flame | Magma Core [More] | 3 | 10% more Flame Shock damage per rank |
| Frost | Frozen Power [More] | 3 | 10% more Frost Shock damage per rank |
| Frost | Frostbite [Syn] | 3 | Frost Shock on a Chilled enemy has a 15% chance per rank to Freeze it |
| Frost | **Glacial Shock** [A] | 1 | Frost Shock bursts onto every enemy within 6 yards of its target, at 80% |
| Frost | Shock Thrift [M] | 3 | 10% less Shock mana cost per rank |

#### Fire Totems

| Branch | Node | Ranks | Effect |
|---|---|---|---|
| Searing | Searing Bolts [H] | 3 | Searing Totem's bolts strike 1/2/3 more enemies within 8 yards of their target, at 50% |
| Searing | Burning Totem [Ail] | 3 | Your Fire Totems' hits have a 10% chance per rank to Ignite |
| Searing | **Twin Totems** [H] | 1 | Searing Totem throws two bolts at once, as two totems would |
| Searing | Searing Heat [More] | 4 | 8% more Searing Totem damage per rank |
| Magma | Wide Magma [M] | 3 | +15% Magma Totem radius per rank |
| Magma | Magma Pulse [More] | 4 | 8% more Magma Totem damage per rank |
| Magma | **Molten Core** [H] | 1 | Magma Totem's pulses Ignite every enemy they strike |
| Nova | Nova Fury [More] | 3 | 10% more Fire Nova damage per rank |
| Nova | Echoing Nova [H] | 2 | Fire Nova strikes each enemy 1/2 more times, at 60% |
| Nova | **Cataclysmic Nova** [H] | 1 | Fire Nova consumes the Ignite on every enemy it strikes, for the burn left |
| Nova | Totem Thrift [M] | 3 | 10% less Fire Totem mana cost per rank |

#### Earthen Totems

| Branch | Node | Ranks | Effect |
|---|---|---|---|
| Stone | Stoneclaw [H] | 3 | Stoneclaw Totem has 25% more life per rank, and every enemy within 10 yards turns on it when it is placed |
| Stone | Earth's Grasp [Syn] | 1 | The enemies your Earthbind Totem slows are Vulnerable |
| Stone | **Totemic Ward** [H] | 1 | While a totem of yours stands, you take 10% less damage |
| Stone | Earthen Thrift [M] | 3 | 10% less Earthen totem mana cost per rank |
| Wind | Wind's Edge [H] | 3 | +4% attack speed per rank while a Windfury or Grace of Air Totem of yours stands |
| Wind | Air Mastery [H] | 3 | Your Lightning deals 4% more per rank while a Windfury or Grace of Air Totem of yours stands |
| Wind | **Storm Totem** [H] | 1 | Your Lightning hits on enemies within 10 yards of a totem of yours Shock twice |
| Ward | Totemic Reach [M] | 3 | +15% Earthen totem radius per rank |
| Ward | Earthen Vigor [H] | 2 | While a totem of yours stands, you regain 0.5% of your maximum health a second per rank |
| Ward | **Ancestral Totems** [M] | 1 | Your Earthen totems last twice as long |

#### Strike

| Branch | Node | Ranks | Effect |
|---|---|---|---|
| Windfury | Windfury [H] | 3 | Your Windfury Weapon strikes 10% more often per rank |
| Windfury | Tempest Swing [H] | 3 | Windfury's strikes also hit every enemy in front of you, at 30/45/60% |
| Windfury | **Unleashed Rage** [H] | 1 | Each Windfury strike gives you 5% more Melee damage for 6 sec, stacking to 5 |
| Windfury | Weapon Fury [More] | 4 | 6% more Strike damage per rank |
| Imbue | Rockbiter [H] | 3 | Your hits build 15% more Daze per rank |
| Imbue | Flametongue [Conv] | 1 | Your Strike hits count as Fire: they Ignite, not Bleed, and build no Daze |
| Imbue | Frostbrand [Conv] | 1 | Your Strike hits count as Frost: a critical one Chills (Freezes the Chilled), and build no Daze |
| Imbue | Flurry [H] | 3 | +4% attack speed per rank |
| Storm | Charged Strike [Syn] | 3 | Stormstrike Shocks its target 1/2/3 times |
| Storm | Static Shock [H] | 3 | Your swings on a Shocked enemy have a 10% chance per rank to throw a Lightning Bolt at it |
| Storm | **Stormbringer** [H] | 1 | Your critical swings Shock every enemy within 8 yards |
| Storm | Quick Storm [M] | 3 | -1 sec Stormstrike cooldown per rank |

#### Healing Wave

| Branch | Node | Ranks | Effect |
|---|---|---|---|
| Tide | Tidal Force [M] | 4 | 6% more healing per rank |
| Tide | Healing Grace [M] | 3 | -0.2 sec recovery per rank |
| Tide | Riptide [Syn] | 3 | Your heals strike the nearest enemy within 20 yards for 15% of the heal per rank, as Nature |
| Ancestral | Ancestral Healing [Syn] | 2 | Healing yourself gives you 5% more damage per rank for 5 sec |
| Ancestral | Tidal Focus [M] | 3 | 10% less mana cost per rank |
| Ancestral | **Earthliving** [H] | 1 | Your heals Shock every enemy within 8 yards |
| Torrent | Healing Way [M] | 3 | +4% critical strike chance per rank |
| Torrent | Torrent [Syn] | 1 | Your heals burst outward: every enemy within 8 yards takes 50% of the heal as Nature |
| Torrent | **Healing Stream** [H] | 1 | Your heals Poison every enemy within 8 yards, a hit of 30% of the heal (once every 3 sec) |
| Ancestral | Nature's Guidance [M] | 3 | -0.2 sec Chain Heal recovery per rank |

### The shaman passive web

The Elemental, Enhancement and Restoration talents are its notables, with two keystones on each
side. Restoration's smalls carry the minion stats, for the totems.

| Keystone | Side | Effect |
|---|---|---|
| **Thunderlord** | Elemental | Lightning Bolt always chains to 2 more within 10 yd at 50% (the roadmap's "Lightning Bolt always chains"); Lightning deals 15% less |
| **Overcharge** | Elemental | Shocked stacks to 8; it lasts 3 sec, not 4 |
| **Walking Totems** | Restoration | Your totems walk with you; they deal 15% less |
| **Totem Lord** | Restoration | Your totems deal 50% more; your own spells deal 20% less |
| **Windwalker** | Enhancement | Your Windfury Weapon strikes twice as often (20% more a swing); your spells deal 30% less |
| **Elemental Blades** | Enhancement | Your swings strike again for 25% as Fire and again as Frost; your swings deal 15% less |

Notables teach the talents:
- Elemental: Concussion, Call of Thunder, Elemental Fury, Lightning Mastery;
- Enhancement: Thundering Strikes, Enhancing Totems, Flurry, Weapon Mastery;
- Restoration: Totemic Focus, Tidal Mastery, Healing Way, Purification;
- the bridges: Convection, Ancestral Knowledge, Tidal Focus.

### Sets

| Level | Set | Armour | Bonuses | Its power |
|---|---|---|---|---|
| 20 | Stormcaller's Garb | Leather | +8 Intellect; +10% Nature damage | Lightning Bolt chains to 1 more within 10 yd, 50% |
| 30 | Totemwarden's Harness | Leather | +10% Minion damage; +15% Minion life | Flame Shock lands on 1 more within 8 yd |
| 40 | Thunderforged Links | Mail | +15 Strength; +12% Melee damage; +6% attack speed | Stormstrike strikes every enemy in front, 40% |
| 50 | Elemental Ascendance | Mail | +15 Intellect; +12% Nature damage; +1 to all Spell skills | Chain Lightning bursts within 8 yd of each target, 40% |

### Pairings these open (`ARPG-SYNERGY.md`)

- **The warlock's horde and the totems:** the same minion affixes serve both.
- **Daze:** Rockbiter's Daze and Earthen Power feed every Daze unique and the paladin's pairings.
- **Shocked with another class's damage:** every class's damage takes Shocked's more. A Shocked
  enemy is a good target for any payoff in the group, and for the shaman's own Fire Totems.

### Example shaman builds at 60

- **Stormcaller (Lightning, Shocks, Earthen Totems):**
  - Forked Lightning and Thunderstorm clear packs, and Static stacks Shock.
  - Earthquake cashes it in, Ball Lightning carries it on, Storm Totem doubles it.
  - Web: Thunderlord, Overcharge.
- **Earthwarden, the totem minion build (Fire Totems, Earthen Totems, Shocks):**
  - Twin Totems and Molten Core Magma Totems do the work, Walking Totems carry them, and Totemic
    Ward and Stoneclaw hold the pack.
  - Web: Walking Totems, Totem Lord. Gear: minion damage, as the warlock's.
  - **Target:** about 70% of the damage from totems.
- **Enhancer (Strike, Shocks, Earthen Totems):**
  - Windfury and Tempest Swing clear in front; Wind's Edge quickens it.
  - Flametongue Ignites and Lava Burst bursts the Ignites. Stormstrike and Stormbringer Shock the
    pack, and Earthquake cashes it in.
  - Web: Windwalker, Elemental Blades.

### Decisions taken in building it

- **Shocked as a new state.** The alternative, the shaman's Lightning Poisoning, reused the
  druid's ailment and blurred the two classes.
- **Restoration is support,** not a fourth way to play: heals that hurt, in one skill.
- Simplified from the design, for want of the engine: Twin Totems is a second bolt, not a second
  totem; Totem Lord replaces "two totems of each element"; Elemental Blades replaces "both imbues
  at once"; Windfury Totem's own swings are untouched (Wind's Edge gives attack speed instead).

## Making a character

Built. On an ARPG realm, Create New Character opens the class screen first: five vertical
slices, one per ARPG class, each standing a character in its tier 2 set with an epic weapon in
front of a stock character-creation scene, animated as the original glue screens are.

| Slice | Character | Set | Held | Scene |
| --- | --- | --- | --- | --- |
| Paladin | human male | Judgement Armor | Sulfuras, Hand of Ragnaros | human creation scene |
| Mage | undead male | Netherwind Regalia | Atiesh, Greatstaff of the Guardian | undead creation scene |
| Druid | night elf female | Stormrage Raiment | Staff of the Shadow Flame | night elf creation scene |
| Warlock | orc male | Nemesis Raiment | Dragonfang Blade, Fire Runed Grimoire | the Dark Portal (main menu) |
| Shaman | tauren male | The Ten Storms | Aurastone Hammer, Drillborer Disk | tauren creation scene |

- **Clicking a slice** opens the usual creation screen locked to that class, with every race
  open. Back (or Escape) from it returns to the class screen, and Back from there to the
  character list.
- **Any race as any class.** The server builds a missing race and class pair from the race's
  warrior entry (start position, base stats) and a race that has the class (spells, action bar,
  starting gear); `BuildArpgPlayerInfo` in `ObjectMgr.cpp`.
- **The gear is chosen by name** in cmangos `Arpg/ArpgShowcase.cpp`: a set by its `ItemSet.dbc`
  name, a weapon by its item name, so the database supplies the displays. A name it lacks is
  logged and the slot left bare. The client asks with `CMSG_ARPG_SHOWCASE_QUERY` (0x34D) at the
  character screen and gets `SMSG_ARPG_SHOWCASE` (0x34E): per character, class, race, sex,
  look, then 19 equipment slots as `SMSG_CHAR_ENUM` sends them.
- **How the client draws it:** five extra portrait booths (`portrait/showcase.rs`), each a glue
  scene and a dressed character on its own render layer, rendered to a fifth of the window while
  the screen is up and parked at 64x64 otherwise. The screen itself is `class_showcase.rs`.
- **Not yet:** the scenes are the stock race scenes, not the cathedral, mage tower, moonwell,
  demonic landscape and Ten Storms locations first asked for; live world locations would need a
  world loaded per slice. Framing may need tuning once seen.

## How it gets built

### Server effect primitives

Every node and affix effect is one of a small set, so a new node is a data row, not code, except
for capstones and hooks:

| Primitive | What it does | cmangos route |
|---|---|---|
| Stat | +attribute, armour, resistance | `HandleStatModifier` |
| Spell modifier | Damage, cost, cooldown, radius, range, duration, crit, cast time, jump targets on one skill | A `SpellModifier` added to the player for the skill's family mask (`SPELLMOD_*`) |
| Tag bonus | % damage or healing for spells with a tag | One hook in the spell damage and healing bonus path |
| ARPG stat | Movement speed, cooldown recovery, life on kill, mana on hit, area size | Small hooks: speed update, cooldown add, kill, hit, radius |
| Kit | Chain, arc, burst, pierce, spread, echo, raise, step | The uniques' kit (built) |
| Hook | Capstones, synergies, keystones | One function each |

### Data and storage

- **Tables in code**, as the uniques and the current tree are: tags per spell, the web (nodes,
  their position and links), each skill tree, each affix family.
- **Saved:** `character_arpg_tree` grows a column for which tree a node belongs to (the web, or a
  skill); `character_arpg_skill` holds each skill's slot and socketed rune; `character_arpg_codex`
  the pages read.
- **New items:** Codex pages, fragments and runes are new `item_template` rows, shipped as a fork
  SQL file that the server update script applies.

### Client

- **The web window is the ARPG HUD addon's** (`arpg_hud.lua`), drawn with the 1.12 UI's own
  tools: the paths are the taxi map's route lines (a line texture turned by
  `SetTexCoord`'s eight-number form, as `DrawRouteLine` does), in a scroll frame that clips them.
  Drag pans, the wheel zooms about the cursor, nodes glow in their region's colour when taken. This
  replaces the plan for a Rust canvas: the stock UI could draw it after all, and its tooltips and
  buttons come free.
- **One window, tabs** (phase 2): the passive web, then a tab per specialised skill.
- **The server sends positions and links** with the web, so the layout is server data and a
  new node needs no client release.

### Phases

1. **Tags, attributes and the web.** Built: tag bonuses, the five attributes' ARPG effects, the
   paladin web with its small nodes, notables and keystones, boss-kill points, and the web
   window. It replaced the first tree.
2. **Skill specialisation and spells without ranks.** Built: slots by level, the skill point
   pool, the first pass at spells without ranks, and the paladin's five most-wanted skill trees
   (Strike, Seals, Judgement, Consecration, Hammer of Justice), since reworked into six (see
   "Paladin skills that can be specialised"). See "What phase 2 built" below.
3. **Codex pages and runes** (needs the mob packs step for champion packs and fragments).
4. **The other seven paladin skill trees** and the ARPG item affixes, including +skill levels.
5. **Other classes**, one at a time: the mage and the druid are built ("The mage, in full",
   "The druid, in full"); the warlock is built ("The warlock, in full"), and the shaman designed ("The shaman, in full").

### What phase 2 built

- **Slots and points** as "Specialisation" says: five slots at 1/10/20/30/40, one point a level
  from 2, 20 at most in one skill. The window's Skills tab (the talent key) lists the slots
  and the skills; Specialise puts the shown skill in the chosen slot (or the first open one),
  Take Out empties its slot and gives its points back. A skill moved between slots keeps them.
- **Not yet:** a slotted skill isn't placed on a bar; spells stay on the stock action bar. The
  ARPG bar (left click, right click, 1 to 4) is a UI step of its own. Capstones are open:
  nothing seals them until Codex pages drop (phase 3).
- **Spells without ranks, first pass:** an ARPG character gets every spell its class trainers
  teach, free, at the trainer's level (rank 2 of a spell at rank 2's level); the server's
  spellbook shows only a chain's top rank, the client keeps the bar on it, and the spellbook
  hides "Rank N". Tooltips still show it. The smooth blend between ranks is still to come.
- **The five trees** are the tables above, with these differences:
  - Strike: Momentum gives +1/2/3% swing damage per hit in a row (5 at most); Whirling
    Strikes hits all around at 75%; Stagger casts Dazed (a slow) rather than a 1 sec daze.
  - Seals: Seal of Command is already free at 20, so Commanding Seal is +10% Seal of Command
    damage per rank, and Relentless sits under Command Arc. Zeal is +10% attack speed per rank
    while Seal of the Crusader is on; the Seal's own damage loss stays.
  - Hammer of Justice: it's a skillshot for ARPG players already, so Hurled Hammer adds 10 yards
    of range. Ricochet bounces free copies of the stun. Blessed Hammer throws a holy hammer at each
    of the three nearest enemies within 15 yards for half your attack power (no spiral yet).
    Holy Hammer is 30/45/60% of attack power as Holy damage.
  - Walking Consecration, Hallowed Ground and Steadfast check once a second.
- **How it's built:** `Arpg/ArpgSkills.{h,cpp}` holds the skills and trees as data; a node is a
  spell modifier (sent to the client like a talent's, so cooldowns and ranges agree), a uniques
  kit row, or a keystone rank a hook reads (`Arpg/ArpgCharacter.cpp`). Saved in
  `character_arpg_skill` and `character_arpg_skill_node`; wire in `ARPG.md`.

## Open questions for Jeff

- **The attribute rates and caps** in "Attributes": a fair first pass?
- **Codex page homes:** the bosses above are chosen for theme. Swap any that feel wrong.
