# ARPG uniques: named items that change how spells work

Partly built: the first slice is in (see "Built so far"). Jeff's call (see `ARPG-ITEMISATION.md`,
"Decisions so far"): vanilla's named blues and purples keep their stats and gain one hand-picked interaction with a specific spell or
attack, Diablo-unique style. Generic greens and blues get random affixes later; these items are
the chase.

Item ids, levels and sources below come from classic-db (`ClassicDB_1_12_1_z2815`).

## Colour

Jeff's call: an item's name keeps its vanilla quality colour, so its tier reads at a glance (a
level 23 blue never looks like a raid purple, and nothing looks like Thunderfury's orange). Being
a unique is shown by a second colour, the client's seventh quality colour, "Artifact" pale gold
(`e6cc80`), which no 1.12 item wears:

- **Tooltip:** the mechanic line, "Unique: …", in pale gold under the item's own lines.
- **Ground:** a gold border round the label, and a thin gold core inside a taller beam.

| Kind | Name colour | Extra |
|---|---|---|
| Junk to legendary | Vanilla grey/white/green/blue/purple/orange | None |
| Random affixes (later) | Its own | Affix lines in its own colour, as vanilla's "of the Bear" |
| Uniques | Its own (blue or purple) | Pale gold mechanic line, label border, beam core |
| Tier set pieces | Purple | Possibly a set colour later |

If a gold accent turns out too quiet in play, the beam carries more: thicker or pulsing.

## Rules for every unique

- **One mechanic per item**, written as one tooltip line in pale gold under the item's own stats. The
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

### Paladin set (Jeff plays one)

Built with the rest. Paladin play is swings with a Seal up plus Judgement, so the uniques hang
off those, from the first dungeons on.

| Item | Id | Drops from | Kit | Mechanic | Level |
|---|---|---|---|---|---|
| Smite's Mighty Hammer | 7230 | Mr. Smite, Deadmines | G | Seal of Righteousness strikes every enemy in front of you; the extras take 50%. | 18 |
| Taskmaster Axe | 5194 | Sneed, Deadmines | C | Judgement chains to 2 more enemies within 10 yd for 60%. | 18 |
| Kresh's Back (shield) | 13245 | Kresh, Wailing Caverns | F | Hammer of Justice also stuns 2 more enemies within 8 yd. | 15 |
| Mograine's Might | 7723 | Scarlet Commander Mograine, Scarlet Monastery | E | Judgement also strikes every enemy within 8 yd of the target for 50%. | ~37 |
| Hand of Righteousness | 7721 | High Inquisitor Whitemane, Scarlet Monastery | E | Seal of Righteousness strikes burst onto enemies within 5 yd for 35%. | 39 |
| Hand of Edward the Odd | 2243 | World drop (57+) | C | Holy Shock chains to 2 more enemies within 10 yd for 60%. | 57 |
| Hammer of the Grand Crusader | 18717 | Balnazzar, Stratholme | B | Hammer of Wrath pierces every enemy in its path at 70%. | 58 |
| Spinal Reaper | 17104 | Ragnaros, Molten Core | G | Seal of Command strikes every enemy in front of you for 60%. | 60 |

Holy Shock and Seal of Command are talents. The server checks every row against its data when
the first ARPG player says hello and logs a wrong item or spell id as an error.

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

- **Data:** a table in code, `Arpg::Uniques()` in `src/game/Arpg/ArpgUniques.cpp`: `item`, `kit`,
  `spell` (rank 1 of the spell; 0 for a swing or auto shot), `n`, `value` (angle, reach, radius
  or chance, by kit), `pct`, `text` (the tooltip line). A database table was the first idea, but a
  table in code needs no SQL install, can't be wiped by a database reinstall, and the server is
  rebuilt for every mechanic anyway. A new item is one line there.
- **Which mechanics a player has:** read from what they wear when a mechanic could fire (19
  slots against a short table), so equipping needs no bookkeeping. Two items touching the same
  spell with the same kit: the one with more projectiles or fragments applies.
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

- **Added projectiles (kit A)** are real casts of the same spell (`Spell::SetArpgSecondary`):
  free, no cooldown, flying their own line beside the main one and never picking the unit the
  main shot took. They carry the spell's full effects (Frostbolt's slow), but their damage is
  `pct` of the normal hit, they never crit and they set off no procs. They show as ordinary
  missiles of that spell.
- **Fragments (kit D) and chain jumps (kit C)** are direct damage: `pct` of what the main hit
  dealt (after mitigation), through the ordinary spell damage path (armour, resistances, absorbs
  and the combat log work), with no other effects. They fly: the server sends a cosmetic
  `SMSG_SPELL_GO` of the spell from the unit they leave to the one they reach, so the client
  draws the missile (a fireball from the struck mob to the one behind it), and the hit lands when
  that missile arrives (`Spell.dbc` speed). A chain picks its next jump when it lands.
- **Bursts (kit E) and arcs (kit G)** land at once, with no missile: a burst hits everything near
  the target, an arc everything else in the swing's 120° and reach.
- **Which spell a row covers:** any rank of it, and any spell named after it (each Arcane
  Missiles missile, each Judgement), so a row names one spell for all its ranks and parts.

### Client (benilla)

- **Tooltip line:** the server sends the whole table after every hello
  (`SMSG_ARPG_ITEM_MECHANICS`, a few hundred bytes). The client keeps it
  (`player/arpg/uniques.rs`) and the item tooltip feed adds the pale gold "Unique: …" line to
  any matching item (`ItemTemplateView::arpg_unique`, drawn in `tooltip_item/render.rs`).
- **Ground loot:** a unique's label gets a gold border and its beam a thin, taller gold core
  (`player/arpg/loot.rs`), so it stands out even as a blue.

### Balance guard rails

- Single target: no unique adds more than about 15% damage on one enemy (Echo and the burst kits
  are the ones to watch).
- Packs: up to about double the damage spread over the pack is fine and is the point.
- Raid bosses with adds (Majordomo, Ragnaros' sons, Onyxia's whelps) get easier with uniques.
  That's acceptable, since raids are being retuned for solo play anyway (`ARPG-ITEMISATION.md`
  step 5).

## Built so far, and next

Built:
- The table, the hello-time `SMSG_ARPG_ITEM_MECHANICS` (0x33E: `u8` count, then `u32` item and
  a C string per row), the tooltip line and the ground accent.
- Kit A on **Emberstone Staff** (5201, Fireball +1) and **Quillshooter** (10567, Arcane Shot +2).
- Kit C on **Freezing Shard** (10572, wand bolts chain twice), **Illusionary Rod** (7713, each
  Arcane Missiles missile jumps once), **Whitemane's Chapeau** (7720, Smite chains twice) and
  **Ramstein's Lightning Bolts** (13515, Lightning Bolt chains twice).
- Kit D on **Staff of Dominance** (18842, Fireball 5 fragments), **Rod of the Sleepwalker** (1155,
  Wrath 3 motes) and **Staff of Jordan** (873, Frostbolt 4 shards; the slow on shards waits for
  a later pass, so its line doesn't promise it).
- Kit E on **Cookie's Stirring Rod** (5198, wand splash) and **Mograine's Might** (7723,
  Judgement splash).
- Kit G on **Night Reaver** (1318, Heroic Strike) and **Felstriker** (12590, Eviscerate).
- Kit B on **Witching Stave** (1484, Shadow Bolt pierces 1), **Bow of Searing Arrows** (2825,
  Auto Shot pierces all) and **Hammer of the Grand Crusader** (18717, Hammer of Wrath pierces
  all). Pierce hits fly on from the struck unit like fragments.
- Kit F on **Venomstrike** (6469, Serpent Sting), **Living Root** (6631, Entangling Roots),
  **Hypnotic Blade** (7714, Polymorph): free copies of the spell land on the nearest enemies
  when it lands. **Lok'amir il Romathis** (19360): Shadow Word: Pain copies onto every enemy
  near its target when that dies.
- Kit H on **Meteor Shard** (6220, Sinister Strike shard along your facing, no missile yet).
- Kit I on **Azuresong Mageblade** (17103, every 3rd Frostbolt echoes half a second later).
- Kit J on **Book of the Dead** (13353): a 20% chance on any kill that a Skeleton (creature 6412,
  set to your level and faction) rises at the corpse and fights for 20 sec.
- Kit K on **Perdition's Blade** (18816): a Sinister Strike kill teleports you behind the
  nearest enemy within 10 yd.

30 items (24 of the first 27, plus the paladin set). Test with `.additem <id>` on a GM account, or farm them.

Not built yet:
1. **Ravager**: Whirlwind's pull.
2. **Bonereaver's Edge**: a swing hitting 3+ enemies sends a shockwave. ARPG swings strike one
   enemy today, so this waits for swing cleave, or its line changes to a plain shockwave.
3. **Striker's Mark**: Multi-Shot as a fan. Multi-Shot is a chain-target spell, not a skillshot,
   so kit A doesn't catch it; it needs Multi-Shot itself made a fan of skillshots.
4. The shard slow on Staff of Jordan, and a missile for the Meteor Shard shard.

The Drop Test Loot tool rolls random items. A "drop this item id" field on the same page would
make testing a unique quicker than the GM command; it's a small addition.

## Open questions for Jeff

- Should a few **new** uniques exist alongside these: items that don't exist in vanilla,
  as world drops from champion packs?
- Should two uniques stack if they touch the same spell (two Fireball items)? For now only the
  best applies.
