# ARPG itemisation: turning vanilla's MMO gear into Diablo-style loot

Design notes, not a spec. Nothing here is built yet. Written for Jeff and for cloud sessions
picking this up; read `ARPG.md` first.

## The tension

Vanilla's itemisation and an ARPG's pull in opposite directions.

| | Vanilla (MMO) | ARPG (Diablo/PoE) |
|---|---|---|
| Items | Hand-authored, fixed stats; a Thunderfury is always the same Thunderfury | A base type plus rolled affixes; two drops of one base are rarely equal |
| Volume | Few drops, mostly vendor trash; an upgrade is an event | A shower of drops; most are junk, and sorting them is part of the game |
| Pacing | Gated by raid lockouts and group rolls | Gated by drop odds and your ability to farm faster |
| Build identity | Talents and set bonuses; items are mostly stat sticks | Items, especially uniques, change how skills work |
| Power curve | Flat-ish within a tier, step changes between tiers | A smooth climb, then an endless long tail of marginal gains |

The aim is not to replace vanilla's items. They carry the world: Thunderfury, tier sets, and the
pre-raid best-in-slot lists Jeff already builds macros for. The aim is to put an ARPG loot layer
on top, so every kill can roll something interesting, while the famous items stay chase items.

## What vanilla already gives us (more than it looks)

- **Rarity tiers are already Diablo's.** Grey, white, green, blue, purple and orange map almost
  one-to-one onto junk, normal, magic, rare, epic and unique. The ground loot colours already use
  them.
- **Random affixes already exist.** "of the Bear", "of the Eagle" and similar are random
  properties (`ItemRandomProperties.dbc`). An item rolls one at drop time, and it applies up to
  three enchantments (stat bonuses from `SpellItemEnchantment.dbc`) into the item's three random
  property enchant slots (`Item::SetItemRandomProperties`, `PROP_ENCHANTMENT_SLOT_0..2`). There
  is a **fourth** property slot (`PROP_ENCHANTMENT_SLOT_3`) that vanilla never uses: room for one
  extra affix without inventing storage.
- **Skill-modifying gear exists.** Equip effects like "increases the damage of your Frostbolt by
  X", set bonuses and on-hit procs are ordinary item spells with spell modifiers. That is a
  ready-made way to make an item change a skill.
- **Drop rates are config.** `mangosd.conf` has `Rate.Drop.Item.Poor` through `.Artifact` and
  `Rate.Drop.Money`. On the separate ARPG world these can be raised without touching Jeff's
  playerbot server, which is half the reason for running two worlds.

What vanilla does **not** have: item level scaling with the monster, more than one rolled affix
family per item, uniques that change mechanics (pierce, chain, split), and any way to show a
brand-new affix name on the client without new DBC rows.

The last point decides a lot. The client shows stats and suffix names only from its own DBC
files, so a new affix needs rows that both the server (its `dbc/` folder) and the client (a patch
MPQ, the standard private-server route) can read. benilla already reads the player's patch chain,
so a fork patch MPQ with extra rows works with no client code. It's just a data file to keep in
step on both ends.

## A path in steps, cheapest first

### 1. Volume and sorting

This alone already changes the feel a lot.
- Raise drop rates on the ARPG world. Uncommon and rare by a lot, epic a little, grey rather
  down, gold up.
- Add a **loot filter** to the ground loot: hide grey labels, or all labels below a chosen
  quality, unless Alt is held. A small client-side change (`loot.rs`'s label pass).
- **Auto-sell or salvage greys.** A vendor button that sells all greys, or a "salvage" action
  (disenchant anything green or better into materials). Volume without a sink drowns the bags.
- More bag space for the ARPG world: a starter bag set, or a bigger backpack.

### 2. Real affixes from vanilla's own tables

Cheap, no new data.
- At corpse loot generation for an ARPG looter, give every green and blue that can take one a
  random property from the vanilla families that fit its item level, even items that never had
  one. The roll goes into `LootItem::randomPropertyId`, which the ground loot packet and
  `StoreNewItem` already carry. The name and stats show on the client with no changes, because
  the DBC rows already exist.
- **Item level follows monster level** within a band: a level 30 mob's drop rolls the level-30
  tier of "of the Bear", not the level-10 one. The families come in tiers by design.
- Use the spare fourth slot for a second, independent affix (a "prefix" in Diablo terms) drawn
  from existing `SpellItemEnchantment` rows: +stamina, +attack power, +spell damage, and so on.
  The server sets it with `SetEnchantment(PROP_ENCHANTMENT_SLOT_3, …)`. One thing to check: does
  the stock tooltip show slot 3? If not, benilla's tooltip needs a line for it, a small
  client-side change.
- **To verify first:** that forcing a random property onto an item whose template has none
  saves, loads and displays correctly in cmangos and benilla.

### 3. Our own affix table

New data, still mostly server-side.
- A fork table (say `arpg_affix`): affix id, the enchant it applies, the item slots it may roll
  on, the item-level range, and a weight. The server rolls it into slot 3. New stat values or
  combinations need new `SpellItemEnchantment` rows in the shared patch MPQ.
- Affix names ("Vicious", "of Embers") need `ItemRandomProperties` rows to show in the item
  name. Alternatively, benilla shows them its own way: we own the client, so the tooltip can
  read a fork-only affix line.

### 4. Uniques that change skills

The fun part. Needs server code per mechanic.
- Most already work through item spells: "+30% Frostbolt damage", "Fireball costs 20% less",
  "Holy Light heals your target again over 6 s". Ordinary spell modifiers and procs.
- The true ARPG mechanics live in our own server code, which already owns skillshots:
  - **Pierce N:** `SelectLineTarget` keeps going past the first hit.
  - **Chain:** on hit, re-cast at the nearest enemy within X yards.
  - **Split:** the shot fans into three lines.
  - **Explode on kill:** a small area effect at the corpse.
  - **Swing cleave:** the melee arc hits everyone in it, not just the first.

  Each is a passive aura on the player, granted by an equipped item and read in `ArpgCombat.cpp`.
- Vanilla's own legendaries (Thunderfury, Sulfuras) stay as they are: they are already uniques.

### 5. Scaling for solo weekly raids

Content, not items.
- Scale the raid creatures, not the player: per-instance health and damage multipliers for the
  ARPG world, tuned so a well-geared solo character clears Molten Core in a sitting. cmangos
  supports creature multipliers in data and scripts. Boss mechanics built for 40 people (Ragnaros'
  knockback, Majordomo's adds, Onyxia's whelps) need per-boss tuning, not just numbers. That's
  the real work of the raid phase.
- Raid loot volume goes up: each boss drops several items for the one player, plus rolled
  affixes on its set pieces, so a weekly clear feels like a Diablo boss run rather than one roll
  among 40.
- Tier sets stay the chase. Affixes make each piece a little different, but the set bonus is
  why you want it.

## Talents and skills

Leave them stock at first, for three reasons:
- They are what make each class play like itself.
- Changing them touches every class's balance.
- The ARPG combat layer (arcs, skillshots, re-aim) already changes how skills feel more than
  numbers would.

Changes worth considering later, in order:
1. **Resource pacing.** Diablo spends mana fast and gets it back fast. Vanilla mana is a long,
   slow tank meant for raid fights. A higher regen rate (or out-of-combat regen) for ARPG players,
   or potions on hotkeys (Phase 3), keeps casters moving.
2. **Area skills earn their keep.** With denser packs (Phase 3), Arcane Explosion, Whirlwind,
   Consecration and Volley become the core of a build, as in Diablo. That happens without any
   change, as long as mobs come in groups.
3. **Talents that fit ARPG play better.** A few vanilla talents do little here: anything about
   threat, or about groups that don't exist solo. An ARPG talent pass could repoint those at
   mechanics from step 4 (a Fire talent that gives Fireball a split chance, say). That's a big
   design job, best left until items are settled.

## What I'd do next, and in what order

1. **Drop rates and the loot filter.** An evening's work. Most of the "it feels like Diablo"
   effect for the least effort.
2. **Vanilla random properties on every green and blue, tiered to monster level.** Server-only,
   uses existing data. Turns every drop into a roll.
3. **The fourth-slot affix.** Small server change, maybe a tooltip line in benilla.
4. **Skill-changing uniques**, starting with Pierce and Chain on skillshots: we own that code.
5. **Raid scaling**, boss by boss, starting with Molten Core.

## Decisions so far

- **Named blues and purples get unique interactions** with specific spells, rather than random
  affixes. Jeff's examples: a blue staff that adds one projectile to every skillshot; fireballs
  that burst into fragments after hitting. That is step 4 (skill-changing uniques) applied to
  vanilla's named items: each gets a hand-picked mechanic, granted as an equip aura and read in
  `ArpgCombat.cpp`.
- **Skill trees: a new ARPG tree per class**, replacing vanilla talents for ARPG players.
- **Mob packs: denser spawns led by champions** with bonus loot.
- Order of work: loot (drop rates, filter: done), then skill trees, then mob packs.

## Open questions for Jeff

- Item level beyond 60? An ARPG endgame usually wants an endless long tail. Vanilla stops at 60
  and Naxxramas. Options:
  - A level 60 "paragon" track: account-wide small bonuses per extra XP bar.
  - Affix tiers above vanilla values, earned in harder weekly raid difficulties.
- Trade and economy: is this strictly solo (bind everything to the account), or should bots and
  the auction house matter?
