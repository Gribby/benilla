# ARPG uniques: named items that change how spells work

Design, not built yet. Jeff's call (see `ARPG-ITEMISATION.md`, "Decisions so far"): vanilla's named
blues and purples keep their stats and gain one hand-picked interaction with a specific spell or
attack, Diablo-unique style. Generic greens and blues get random affixes later; these items are
the chase.

Item ids, levels and sources below come from classic-db (`ClassicDB_1_12_1_z2815`).

## Rules for every unique

- **One mechanic per item**, written as one tooltip line in orange under the item's own stats. The
  vanilla stats and procs stay exactly as they are.
- **Spell-specific beats generic.** "Fireball bursts into fragments" makes a build; "+5% damage"
  doesn't. Each mechanic names one spell (every rank) or one attack.
- **It works because of ARPG play.** Most mechanics pay off against packs (pierce, chain,
  fragments, cleave). Against one boss they add a little damage or nothing, so raid balance and
  vanilla best-in-slot lists mostly stand.
- **Secondary hits are weaker.** Anything a mechanic adds (an extra projectile, a fragment, a
  chain jump) deals a fixed share of the original hit's damage, shown in the tooltip. No added hit
  can crit on its own or proc on-hit effects, so mechanics never stack into each other.
- **ARPG players only.** The same item on a stock-client player or a playerbot does nothing new.

## The mechanic kit

Every unique is one of these engine pieces with its own numbers. Build the piece once and each new
item becomes a data row.

| Kit | Name | What it does | Params |
|---|---|---|---|
| A | Extra projectiles | A line skillshot fires N more lines, fanned around the aim | spell, N, fan angle, damage % |
| B | Pierce | A line skillshot carries on through N enemies after the first | spell, N, damage % per pierce |
| C | Chain | On hit, jumps to the nearest enemy not yet hit, N times | spell, N, jump range, damage % |
| D | Fragments | On hit, bursts into N shards that fly on past the target in a fan | spell, N, range, damage % |
| E | Burst | On hit (or on a kill), damages everything around the target | spell or attack, radius, damage % |
| F | Spread | An aura on the target (a DoT, a root, a sting) copies to enemies nearby: on apply, or when the target dies | spell, radius, count |
| G | Arc | A single-target melee ability hits every enemy in the swing arc | spell, damage % for the extras |
| H | Shockwave | A melee ability also sends a ground wave along the aim | spell or swing, length, damage % |
| I | Echo | Every Nth cast of a spell repeats for free half a second later | spell, N, damage % |
| J | Raise | Enemies you kill may rise and fight for you for a while | chance, creature, duration |
| K | Step | A kill with a spell moves you behind the nearest enemy | spell, range |

## The items

27 items: every class gets at least two, spread from Deadmines to Blackwing Lair.

### Levels 15–30

| Item | Id | Drops from | Kit | Mechanic (tooltip line) | For |
|---|---|---|---|---|---|
| Emberstone Staff | 5201 | Captain Greenskin, Deadmines | A | Fireball launches 1 extra fireball. Each deals 60% damage. | Mage |
| Cookie's Stirring Rod | 5198 | Cookie, Deadmines | E | Your wand bolts splash enemies within 4 yd of the target for 40%. | Mage, Priest, Warlock |
| Night Reaver | 1318 | Shadowfang Keep (shared boss table) | G | Heroic Strike hits every enemy in front of you; the extras take 50%. | Warrior |
| Witching Stave | 1484 | Shadowfang Keep (shared boss table) | B | Shadow Bolt pierces 1 enemy, dealing 70% to it. | Warlock |
| Meteor Shard | 6220 | Archmage Arugal, Shadowfang Keep | H | Sinister Strike sends a burning shard 12 yd along your aim for 50%. | Rogue |
| Venomstrike | 6469 | Lord Serpentis, Wailing Caverns | F | Serpent Sting spreads to 2 enemies within 8 yd when it lands. | Hunter |
| Living Root | 6631 | Verdan the Everliving, Wailing Caverns | F | Entangling Roots also roots 2 more enemies within 6 yd of the target. | Druid |
| Rod of the Sleepwalker | 1155 | Twilight Lord Kelris, Blackfathom Deeps | D | Wrath bursts into 3 motes on hit, each dealing 30% to the enemies behind. | Druid |

### Levels 30–50

| Item | Id | Drops from | Kit | Mechanic (tooltip line) | For |
|---|---|---|---|---|---|
| Quillshooter | 10567 | Razorfen Downs (shared table) | A | Arcane Shot fires 3 quills in a 30° fan. Each extra deals 50%. | Hunter |
| Freezing Shard | 10572 | Razorfen Downs (shared table) | C | Your wand bolts chain to 2 more enemies within 8 yd for 50%. | Mage, Priest, Warlock |
| Illusionary Rod | 7713 | Arcanist Doan, Scarlet Monastery | C | Each Arcane Missiles missile jumps to a second enemy for 50%. | Mage |
| Hypnotic Blade | 7714 | Arcanist Doan, Scarlet Monastery | F | Polymorph also turns 1 more enemy within 8 yd into a sheep. | Mage |
| Ravager | 7717 | Herod, Scarlet Monastery | K* | Whirlwind first pulls every enemy within 10 yd in toward you. | Warrior |
| Whitemane's Chapeau | 7720 | High Inquisitor Whitemane, Scarlet Monastery | C | Smite chains to 2 more enemies within 10 yd for 60%. | Priest |
| Mograine's Might | 7723 | Scarlet Commander Mograine, Scarlet Monastery | E | Judgement also strikes every enemy within 8 yd of the target for 50%. | Paladin |
| Staff of Jordan | 873 | World drop (levels 35–45) | D | Frostbolt shatters on hit into 4 ice shards. Each deals 30% and slows. | Mage |
| Bow of Searing Arrows | 2825 | World drop (levels 37–47) | B | Your Auto Shot arrows pierce every enemy in their path at 60%. | Hunter |

\* Ravager's pull is a small extra: a knock-in rather than one of the kit pieces. Build it after
the kit.

### Levels 50–60 dungeons

| Item | Id | Drops from | Kit | Mechanic (tooltip line) | For |
|---|---|---|---|---|---|
| Ramstein's Lightning Bolts | 13515 | Ramstein the Gorger, Stratholme | C | Lightning Bolt chains to 2 more enemies within 10 yd for 50%. | Shaman |
| Felstriker | 12590 | Warchief Rend Blackhand, Upper Blackrock Spire | G | Eviscerate strikes every enemy in front of you at 60%. | Rogue |
| Book of the Dead | 13353 | Balnazzar, Stratholme | J | Enemies you kill have a 20% chance to rise as a skeleton that fights for you for 20 sec. | Warlock, Priest |
| Hammer of the Grand Crusader | 18717 | Balnazzar, Stratholme | B | Hammer of Wrath pierces every enemy in its path at 70%. | Paladin |

### Raids

| Item | Id | Drops from | Kit | Mechanic (tooltip line) | For |
|---|---|---|---|---|---|
| Staff of Dominance | 18842 | Golemagg, Molten Core | D | Fireball bursts into 5 fragments on hit, each dealing 25% to the enemies behind. | Mage |
| Azuresong Mageblade | 17103 | Golemagg, Molten Core | I | Every 3rd Frostbolt is echoed for free at 60%. | Mage |
| Striker's Mark | 17069 | Magmadar, Molten Core | A | Multi-Shot fires 5 arrows in a 45° fan rather than picking 3 targets. | Hunter |
| Bonereaver's Edge | 17076 | Ragnaros, Molten Core | H | A swing that hits 3 or more enemies sends a shockwave 15 yd ahead for 50%. | Warrior |
| Perdition's Blade | 18816 | Ragnaros, Molten Core | K | A kill with Sinister Strike steps you behind the nearest enemy within 10 yd. | Rogue |
| Lok'amir il Romathis | 19360 | Nefarian, Blackwing Lair | F | Shadow Word: Pain spreads to every enemy within 8 yd when its target dies. | Priest, Shaman, Druid |

Legendaries stay as authored. Thunderfury already chains and Sulfuras already burns everything
around it: they were ARPG items before there was an ARPG.

## How it gets built

### Server (cmangos)

- **Data:** a fork table in the world database, `arpg_item_mechanic`, filled from a fork SQL
  file: `item`, `kit` (A to K), `spell` (rank 1 of the spell; 0 for a swing or auto shot), `n`,
  `value` (radius, angle, range or chance, by kit), `pct`, `text` (the tooltip line).
- **Which mechanics a player has:** when an ARPG player equips or unequips an item, the server
  rebuilds that player's list from what they wear. Hooks read the list, never the items.
- **Matching a spell:** any rank counts. The hook compares the cast spell's first rank
  (`SpellMgr::GetFirstSpellInChain`) with the row's `spell`.
- **Hooks (most already exist from the ARPG combat work):**

  | Hook | Kits |
  |---|---|
  | Line skillshot resolution in `ArpgCombat.cpp` | A, B |
  | Spell hit (`Spell::DoAllEffectOnTarget`, after damage) | C, D, E |
  | Swing and melee ability resolution in `ArpgCombat.cpp` | G, H |
  | Aura apply and `Unit::Kill` | F, J, K |
  | Cast finish | I |

- **Added hits:** the main hit's final damage is taken after it lands, and each added hit deals
  that × `pct` as the same school, through the ordinary spell damage path (armour, resistances,
  absorbs and combat log all work). It has no other effects of its own, except where the line
  says so ("and slows"), which applies the spell's own slow aura.
- **What the player sees:** an added projectile or fragment is a cosmetic missile of the same spell
  from the source point to its end point. On the client, a world-point missile is how ground
  spells already show; whether benilla draws a missile for a pure visual is the first thing to
  check when building A and D.

### Client (benilla)

- **Tooltip line:** the server sends the whole table once, after the hello (a new
  `SMSG_ARPG_ITEM_MECHANICS`: item id and text per row, a few kilobytes). The client keeps it per
  session and adds the orange line to any matching item's tooltip, on the ground label's hover as
  well.
- **Ground loot:** a drop with a mechanic gets an orange edge on its label and a taller beam, so a
  unique stands out even when it's a blue.

### Balance guard rails

- Single target: no unique adds more than about 15% damage on one enemy (Echo and the burst kits
  are the ones to watch).
- Packs: up to about double the damage spread over the pack is fine and is the point.
- Raid bosses with adds (Majordomo, Ragnaros' sons, Onyxia's whelps) get easier with uniques.
  That's acceptable, since raids are being retuned for solo play anyway (`ARPG-ITEMISATION.md`
  step 5).

## The first slice to build

1. The table, the per-player list, the item-mechanics packet and the tooltip line.
2. Kit A (extra projectiles) and kit D (fragments): Jeff's two examples. Prototype them on
   **Emberstone Staff** (Deadmines, easy to farm or add with `.additem 5201`) and **Staff of
   Dominance** (`.additem 18842`).
3. Kits C (chain) and G (arc). Between them they cover the most items in this list.
4. The rest in any order. Each new item is then a data row plus a playtest.

The Drop Test Loot tool rolls random items. A "drop this item id" field on the same page would
make testing a unique quicker than the GM command; it's a small addition.

## Open questions for Jeff

- Should uniques show their mechanic in their **name** too (an orange "Emberstone Staff" in place
  of the blue), or only on the tooltip line?
- Should a few **new** orange uniques exist alongside these: items that don't exist in vanilla,
  as world drops from champion packs?
- Should two uniques stack if they touch the same spell (two Fireball items), or should only the
  best apply?
