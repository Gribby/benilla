# ARPG project handoff

This document lets a new Claude Code session pick up the ARPG project without the history.
Read it first, then `ARPG.md` (the client fork's overview) and the design docs it names.

## What the project is

Jeff (GitHub **Gribby**) is turning World of Warcraft 1.12.1 into a Diablo / Path of Exile / Last
Epoch style ARPG on a self-hosted server. It is two forks, always worked on together:

| | Repository | Branch | Jeff's checkout |
|---|---|---|---|
| Client | `Gribby/benilla`: a from-scratch Rust/Bevy 1.12.1 client | `arpg` | `D:\benilla` |
| Server | `Gribby/mangos-classic`: cmangos Classic | `arpg` | `D:\Wow\mangos-arpg` |

- **Two worlds:** the ARPG server is a separate plain cmangos world (no playerbots) with a fresh
  characters database. It runs alongside Jeff's existing playerbots server.
- **Windows build:** Jeff builds on Windows with MSVC (Visual Studio 18, toolset 14.51) and
  mostly plays a **paladin**.
- **Server update:** `update-arpg-server.bat` in the server repo pulls, re-runs CMake, builds
  mangosd, copies it to `RUN_DIR` and starts it. Add `full` for a clean rebuild or `nopull` to
  skip the pull. `RUN_DIR` comes from the git-ignored `arpg-local.bat`. The client is rebuilt
  with cargo.
- **Switches:** everything ARPG is behind them. On the client `WOW_ARPG=1`; on the server
  `Arpg.Enable = 1`, plus `Arpg.DevTools = 1` for test loot and test packs. A player becomes
  ARPG only after the client's hello (`Arpg::Active(player)`).

## Jeff's standing direction

- He wants a full, enjoyable, Warcraft-flavoured ARPG, not a half-measure.
  - "Less back and forth": make the reasonable call, say what you chose, and keep going.
  - "I don't want half thought through systems": design a whole system, not a stub.
  - Ask only for decisions that are genuinely his and costly to get wrong (one short
    multiple-choice question).
- **Combat:** no tab-targeting and no selection. The character faces the cursor, swings hit
  whatever they connect with or whiff, and spells are skillshots.
- **No hit stat:** ARPG players, their pets and totems never miss (`Arpg::NeverMisses`). Dodge,
  parry, block and resistances still apply.
- **Levelling:** quests are optional; bounties and kills carry it, at about 1.3–1.5× vanilla's
  pace. No quest glows or markers. No daily quests.
- **Endgame and difficulty:**
  - Raids are the endgame, gated by items from endgame dungeons.
  - Bosses are soloable, with damage caps and no auto-attack one-shots.
  - Dungeons stay special: a challenge with a reward.
- **Respecs:** free, both the passive web and the skill trees.
- **Loot:**
  - It drops on the ground with no group loot rules and no binding.
  - Rarity rolls:

    | Rarity | Affixes |
    |---|---|
    | White | 0 |
    | Green | 1–2 |
    | Blue | 3–4 |
    | Purple | 5–6 |

  - **Uniques** are gold, with a border in their affix tier's colour. Every vanilla named blue
    and purple is a unique.
    - Each has its vanilla stats as implicits, a unique power, and a fixed set of affixes. The
      affixes are fixed by item, the same on every copy; only their values roll.
    - Rolled drops are never uniques.
  - Raid and high-level mobs drop greens, blues and purples whose affixes scale with the drop's
    item level.
- **Mob packs:** 3–5 mobs, but a dense vanilla camp counts as a pack already.
- **Deferred:** a weekly raid objective, and the camera's per-room cutaway for multi-storey
  instances (gameplay before camera polish).

## How the halves talk

- **Client to server:** a single opcode, `CMSG_ARPG_ACTION` (0x33C, protocol version 2), whose
  `u8` kind selects the action. Kinds 0–25 are in `src/game/Arpg/ArpgCombat.h` (`ActionKind`).
  - The handler is `src/game/Arpg/ArpgHandler.cpp`.
  - The client writer is `crates/benilla-protocol/src/world/writer/arpg.rs`.
  - Recent kinds: 24 craft (`u8` op, `u8` bag (255 equipped, 0 backpack, 1–4 bags), `u8` slot
    from 1, `u16` affix) and 25 abandon a bounty (`u8` slot).
- **Server to client:** opcodes 0x33D–0x34C. Readers live in
  `crates/benilla-protocol/src/messages/arpg.rs`, wired through `parse.rs`, `packet.rs`,
  `events.rs` and `events/decode.rs`.

  | Opcode | Message | Opcode | Message |
  |---|---|---|---|
  | 0x33D | LOOT | 0x345 | GLOBE |
  | 0x33E | ITEM_MECHANICS | 0x346 | PROC |
  | 0x33F | TREE | 0x347 | SHEET |
  | 0x340 | SKILLS | 0x348 | IMPACT |
  | 0x341 | CHAMPIONS | 0x349 | EVENT |
  | 0x342 | STATUS | 0x34A | BOUNTIES |
  | 0x343 | ITEM_AFFIXES | 0x34B | PURSE |
  | 0x344 | TELEGRAPH | 0x34C | COMPARE |

  `NUM_MSG_TYPES` is 0x34D. A new opcode means three changes: bump it in
  `src/game/Server/Opcodes.h`, add a row to `Opcodes.cpp`, and add the client reader.
- **The client's UI** is a Lua 5.0 addon the client installs: `crates/benilla-app/src/player/arpg_hud.lua`.
  - The HUD, the passive web and Skills window, the tracker and the Forge all live there.
  - **Rust to Lua:** Rust hands data to it by evaluating chunks such as `ArpgSheet_Update({...})`.
    These are built with `lua_str` in `player/arpg/tree.rs`.
  - **Lua to Rust:** Lua answers through session-only CVars, which Rust observes
    (`On<crate::cvars::CvarChanged>`):
    - `arpgTreeAction`: numeric, `kind*100000 + node*100 + nonce`.
    - `arpgCraft`: text, `op:bag:slot:affix:nonce`.
    - `arpgDevLootDrop`.
  - **CVar gotcha:** a CVar whose default parses as a number is numeric, and the registry
    silently refuses non-numeric writes. Text CVars need a text default, which is why `arpgCraft`
    defaults to "". `cvars/mod.rs` has a test listing the string-valued rows; add new ones there.

## Server systems (`src/game/Arpg/`)

| File | What it owns |
|---|---|
| `ArpgCombat` | The ARPG switch (`Active`), action kinds, swings, line skillshots, `EnemiesNear`, `ReportProc`/`ProcHeal`/`ProcMana`, `NeverMisses` |
| `ArpgHandler.cpp` | The action handler (map-thread work goes through `GetMessager().AddMessage`) |
| `ArpgTree` | The passive web; `WebTotals` (every stat the character has) and `RefreshTotals` |
| `ArpgSkills` | Skill specialisation slots and trees (paladin and mage), runes and sockets (per class), held "things" (`Held`/`AddHeld`, table `character_arpg_held`) |
| `ArpgCharacter` | Attributes and their ARPG effects, rest regen, keystones, the character sheet, leech, thorns, kill hooks, `UpdateCharacter` (called each player update) |
| `ArpgUniques` | Authored unique mechanics (kits: extra projectiles, fragments, chain, burst, arc, pierce, echo, spread...), how they stack, impacts, `StrikeFoe` |
| `ArpgProcs` | Cross-skill procs (runes Grace, Verdict, Anvil, Fervour, Zeal; six authored uniques) and **generated unique powers** (trigger, effect, damage type and rider, scaled by item level; every named unique without an authored power gets its own) |
| `ArpgAffixes` | Rarity rolls, about 60 affixes scaling with item level and tier, seeds, crafted affix lists (`character_arpg_item`), crafting operations, salvage, `GiveRolledItem` |
| `ArpgCraft` | Orbs (items 92001–92007, kept in a purse), the Forge's operations, `SMSG_ARPG_PURSE` |
| `ArpgDamage` | The power scale (ARPG-sized damage, creature health to match), the critical multiplier bonus, added damage hits, the seven ailments |
| `ArpgCompare` | What carried items would do to the slotted skills' damage, `SMSG_ARPG_COMPARE` |
| `ArpgBases` | The **base bank**: about 1,100 generated bases at item levels 64–92 (entries 94000 up), with fitted armour/damage and vanilla-green implicits; raid drop swaps and extra raid drops |
| `ArpgLoot` | Ground loot, test loot (`DropDevLoot`), `PickRandomItem` |
| `ArpgPacks` | Packs, champions and rares with affixes, camp-density limits, `MakeLesser` |
| `ArpgDungeons` | Group scaling and damage caps, difficulty tiers (Normal → Torment III), Wardens, final-boss Caches, `LootTier` |
| `ArpgThreats` | Telegraphed attacks |
| `ArpgActions` | Dodge roll, flask, health globes, town portal |
| `ArpgCodex` | Codex pages and fragments, rune items, Molten Core's key (the Core Sigil) |
| `ArpgEvents` | Zone events (Purge, Hunt, Siege) and zone-bound bounties (`character_arpg_zone_bounty`) |
| `ArpgTags` | Spell tags (Holy, Area, Projectile, Seal, Channel...) |
| `ArpgMage` | The mage's keystone hooks (on hit, periodic tick, kill, blow taken); its web is in `ArpgTree`, its trees in `ArpgSkills` |

**Hooks** into core code are marked `// ARPG:` and are found in:
- `Entities/Unit.cpp`: kill flow, loot preparation, damage taken, miss chances;
- `Spells/Spell.cpp`: cast, finish, hit, heal and landed;
- `Loot/LootMgr.cpp`: pickups into the Codex or purse;
- `World/World.cpp`: generated `item_template` rows, written before item templates load;
- `Entities/Player.cpp`, `Entities/Creature.cpp`, `SpellAuras.cpp`.

**Generated `item_template` entries** (the server writes them at start):

| Entries | What |
|---|---|
| 90001 | Codex fragment |
| 90010+ | Runes |
| 90100+ | Codex pages |
| 91001–91010 | Core Sigil pieces |
| 92001–92007 | Orbs |
| 93000–93999 | The first base bank, no implicits; kept for items players hold, never dropped |
| 94000+ | The base bank |

**Tables the server creates:**
- `character_arpg_item`: seed, tier, rarity, crafted affixes, forged count;
- `character_arpg_held`;
- `character_arpg_zone_bounty`;
- tree, skills and Codex tables;
- the dungeon tier table.

## Client systems (`crates/benilla-app/src/player/`)

- `arpg.rs`: the view, camera, steering, the HUD addon install (`HUD_LUA`, `HUD_BINDINGS`) and
  its tests.
- `arpg/`, one module per system:
  - `loot.rs` ground loot;
  - `uniques.rs` mechanic lines;
  - `tree.rs` web/skills plus the action CVar;
  - `sheet.rs` procs/sheet;
  - `strikes.rs` impacts/gore;
  - `threats.rs` telegraph marks;
  - `globes.rs`;
  - `champions.rs`;
  - `actions.rs` flask/roll status;
  - `impact.rs`;
  - `juice.rs`;
  - `fx.rs`;
  - `feel.rs`: proc labels and their kits, kills bursting in the killing blow's damage type, the
    kill streak, the bloom, the player's wounds in red;
  - `world.rs`: event tracker and beam, bounties, purse, the `arpgCraft` CVar.
- `arpg_hud.lua`: everything drawn with the UI (the kill streak and the screen's red edges at
  low health and on big blows are `ArpgHud_Streak` / `ArpgHud_Flash` and the edges block).
- `combat_text/arpg.rs`: the ARPG look of the floating numbers (damage type colours, big
  outlined crits, small merged ticks, labels, thousands separators), under `ArpgNumbers`.
- `benilla-world/src/ffx_glow.rs`: `FfxBloom`, a thresholded bloom on the stock full-screen
  glow, which the ARPG view turns on.
- Other touched areas: `ui_items/feed.rs` (tooltips and rarity colours), `benilla-ui` tooltip
  render, `net.rs` / `net/io.rs` (`ClientCommand`s), `cvars/table.rs` (ARPG CVars).

## Design docs (client repo, `docs/`)

- `ARPG-CHARACTER.md`: skills, runes, Codex, passive web, items, affixes, crafting,
  cross-skill procs, generated unique powers, the base bank, no hit.
- `ARPG-PROGRESSION.md`: packs, camps, champions, dungeons and tiers, survival, threats, raids,
  zone events and bounties, build order.
- `ARPG-UNIQUES.md`, `ARPG-ITEMISATION.md`, `ARPG-SKILL-TREES.md`.

**Keep these current with every change:** they are the spec Jeff reads.

## How to work

### In a cloud sandbox

- **The server build** is not the full tree; the cmangos tree was built once with a PCH build
  directory.
  - Copy changed files into the sandbox copy and compile the touched objects
    (`make -k Arpg/X.o ...` in `build-pch/src/game`).
  - Run `cmake .` in the build directory when files are added: sources are globbed.
  - If a header in the precompiled header changes, delete `CMakeFiles/game.dir/cmake_pch.hxx.gch`
    and rebuild it.
- **Symbol check**, since the full link isn't run:
  ```sh
  nm -C --defined-only Arpg/*.o | grep -o "Arpg::[A-Za-z_]*(.*)" | sort -u > def
  nm -C -u Arpg/*.o Entities/*.o Spells/*.o Loot/*.o World/*.o Server/*.o Groups/*.o | grep -o "Arpg::[A-Za-z_]*(.*)" | sort -u > need
  comm -23 need def
  ```
  The output should be empty. A changed signature (for example, an added default parameter)
  needs every caller's object rebuilt.
- **Client checks:**
  ```sh
  RUSTUP_TOOLCHAIN=stable CARGO_PROFILE_DEV_DEBUG=0 CARGO_INCREMENTAL=0 cargo fmt --all
  cargo clippy -j1 -p benilla-protocol -p benilla-ui -p benilla-app --all-targets -- -D warnings
  cargo test -j1 -p benilla-app --lib -- arpg the_forge cvars::
  cargo test -j1 -p benilla-protocol arpg
  cargo test -j1 -p benilla-ui --lib tooltip
  ```
  - A `benilla-app` test build takes about 15 minutes: run it detached and poll.
  - Never run cargo and a large `make` at the same time (memory).
- **SQL** that generated rows depend on can be checked against `sql/base/mangos.sql` on a local
  MariaDB (`apt-get install mariadb-server`, start `mysqld_safe`).

### Pitfalls that have bitten

- **MSVC macros:** never name an identifier `near`, `far` or `small`. Windows headers define
  them, so Jeff's build breaks while the Linux build passes. Grep new code before committing.
- **Lua 5.0** in `arpg_hud.lua`:
  - no `#` length operator and no `%` operator: use `table.getn` and `math.mod`;
  - handlers use `this` and `arg1`;
  - no `SetEnabled`: use `Enable()` / `Disable()`;
  - test the addon in `UiScript` (the HUD tests in `player/arpg.rs` stub missing fonts and
    templates).
- **Numeric CVar defaults** refuse text writes (see above).
- **Python file edits:** never write `open(p,'w').write(open(p).read()...)`; it truncates the
  file before reading it. Read first, then write.
- **Item tooltips:** unique item lines come in one table keyed by item; an item with both a
  mechanic and a proc sends two rows, which the client joins with `\n`.
- **Threads:** maps update on several threads. Event state uses one recursive mutex;
  bounty, proc and seed state have their own locks. Never take the events lock inside the bounty
  lock.

### Review and commits

- Before a big commit, run an independent read-only review agent over both diffs. Ask it to
  check packet layouts byte for byte, threading, Lua 5.0 and MSVC pitfalls; it has caught real
  bugs every time.
- Commit and push to `arpg` on both repos, ending messages with the session's attribution
  trailer. The tree should be clean at the end of a turn.
- Tell Jeff in a line which side changed: server means `update-arpg-server.bat` (`full` when
  files were added); client means pull and rebuild.

## State at handoff

### Built and played by Jeff

- Camera, movement, swings and skillshots;
- loot, rarities and uniques;
- passive web and skill trees;
- packs and champions;
- dungeon tiers;
- roll, flask and globes;
- telegraphs;
- town portal;
- zone events and bounties;
- the Forge.

### Fixed today after Jeff's testing

- Attack telegraphs used a "can't cast" reticle; they now use plain colour with the
  selection-ring rim.
- The paladin's extra swing hits showed warrior Cleave's visual; they now show a holy (Smite)
  hit.
- The gryphons above the health and mana orbs are removed.
- Bounties are now per zone. There is no hand-in: a finished bounty pays at once.
- Uniques had no unique power; every one now has one, and fixed affixes.
- The test-loot item level reaches 92 (raid item levels).
- A base bank for raids, now with implicits.
- Dense camps had become 30-mob packs.
- Hunts were unkillable at level 22; event summons are now lesser followers.
- The Forge's requests were dropped by the numeric CVar.
- No hit.
- MSVC `near` build errors.

### Untested in game

- **The damage rework** (`ARPG-CHARACTER.md`, "Damage"): ARPG-sized numbers and creature health,
  increased and more, critical multiplier, added damage, ailments, the new affixes, gear skill
  levels, item comparisons, the 20-yard pack pull. Tuning knobs: `Arpg.Power.PerLevel` (0.1) and
  `Arpg.Power.HealthGrowth` (0.5); the ailment shares and the raid growth are constants in
  `ArpgDamage.cpp` and `ArpgDungeons.cpp`.
- The latest server changes: implicits, no hit, Hunt tuning, camp density.
- The base bank's equip-spell lookup (spell damage and healing on Sorcery/Healing bases), which
  depends on Jeff's spell data. The server log should say about 1,100 bases written.
- Raid drop swaps.
- Cross-skill procs and unique powers in real fights.

### Open questions and next steps

**Watch list:**
- Proc balance: Grace, Hand of Edward the Odd, Pyre, Thunder Edge.
- Hunt rares' affixes at low level: consider excluding Stone Skin and Vampiric below 30.
- The paladin's six skills (Crusader, Holy Light, Judgement, Consecration, Hammers, Wrath; five
  rows deep, with ailment nodes): every paladin is respecced once on login (tree revision 2).
- The mage (web, six skills, 16 keystone hooks): untested in game. Check that its notables'
  talents are taught (the server log names a missing one), Blizzard's tick hook, Meteor's instant
  Flamestrike, and Permafrost's chills.
- The feel pass: damage numbers by damage type, crits, merged ticks, wounds in red; proc
  labels over their targets (the server's `SMSG_ARPG_PROC` now carries the name, the target and
  the damage tag, `ReportProcOn`); typed kill bursts; the kill streak; the low-health edges; the
  bloom (`BLOOM_STRENGTH` / `BLOOM_THRESHOLD` in `feel.rs`); an elite kill's beat of hit-stop.
  Untested in game: tune the bloom and the label rate first.
- How the new damage feels: kill speed at 20, 40 and 60 against the old pace; whether Judged
  bursts and Bleeds read well; whether creature health growth (0.5) is right.

**Offered, waiting on Jeff:**
- Stop enemies dodging and parrying ARPG players.

**Not built yet:**
- Druid, warlock and shaman (in that order): a web, six skills and keystone hooks each, as the
  mage's ("The mage, in full" says how classes share the systems: node numbers 40-59, 60-79 and
  80-99 within each skill's hundred).
- Real item models on the ground.

**Roadmap:**
- Phase 4, the raids as weekly-lockout solo ARPG dungeons (Molten Core's attunement is built).
- A paragon track past 60.
- A weekly raid objective (deferred by Jeff).
- The camera's per-room cutaway.
