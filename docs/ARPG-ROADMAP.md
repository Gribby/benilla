# ARPG roadmap: the road to 1.0

How the fork goes from a broad set of first-pass systems to a polished, finished ARPG. This is
the top-level plan; each milestone's detail lives in the topic docs it names (resources and
difficulty: `ARPG-ROADMAP-RESOURCES.md`; packs, dungeons and raids: `ARPG-PROGRESSION.md`;
classes and items: `ARPG-CHARACTER.md`, `ARPG-UNIQUES.md`, `ARPG-ITEMISATION.md`).

## What "finished" means

1.0 is done when all of these are true:

- **Five classes** (Paladin, Mage, Druid, Warlock, Shaman), each with at least three builds that
  feel different and can clear the endgame.
- **A journey from 1 to 60 in acts**: a route through vanilla Azeroth's zones, each act a band
  of zones and dungeons ending in a boss, with dense packs, events and bounties, no dead
  stretches, at 1.3-1.5x vanilla's pace.
- **Every vanilla dungeon** an ARPG dungeon with its Warden and Cache, on every difficulty tier.
- **An endgame**: difficulty tiers up to Torment, the raids from Molten Core to Naxxramas as
  solo or small-group content behind attunements, and a paragon track past 60.
- **A loot chase that lasts**: affixes, uniques that change skills, sets, runes and Codex pages,
  crafting to finish an item, and a stash to keep it all.
- **It looks, sounds and feels finished**: every system explained by its own UI, no placeholder
  art or text, hits and kills that feel weighty, readable danger.
- **It runs**: no crash in a three-hour session, a one-click install and update, characters never
  lost.

1.0 is for Jeff and friends, played solo: no co-op tuning, no public-server hardening.

## Where we are

Breadth is good; depth and polish are not. Most systems exist as a first pass, and almost nothing
has been tuned against real play.

| Pillar | State | What is missing |
|---|---|---|
| Combat feel | First pass | Resource model, difficulty tuning, non-piercing projectiles, hit and kill polish from play |
| Classes | 3 of 5 | Warlock, Shaman; the three built ones untested past the mid 20s |
| Build depth | Systems built | Numbers untuned; one skill carries each build (Fireball is ~100% of a mage's damage) |
| Loot | Breadth built | Drop rates untuned; no stash, no crafting depth, sets only to 50 |
| Journey 1-60 | Vanilla content, ARPG layer on top | Played to about 26; zone pacing, density and pacing past that unknown |
| Endgame | Tiers, Molten Core's attunement | The other raids, paragon, a weekly goal |
| Presentation | HUD, feel, effects | UI pass, skill visuals, onboarding, audio, placeholder art |
| Release | Update scripts, play statistics | Crash reporting, soak tests, backups, installer |

## Milestones

Each milestone ends with play, not code: Jeff plays it, the play statistics are read, and the
exit criteria are checked before the next one starts.

### M1: the core loop is fun, levels 1-25

The first 25 levels of the three built classes are where everything is tuned, because changes
are cheap here and every later level inherits them.

- Resources: never stuck, but mana matters (`ARPG-ROADMAP-RESOURCES.md`, phase 1).
- Difficulty: monster damage by area level, dungeon multipliers, tuned to the play statistics'
  targets (deaths an hour, time at low health).
- Combat: projectiles that fly and can miss, telegraphs and champions that read at a glance, hit
  and kill feel settled.
- Builds: secondary skills and procs worth using; no skill carries more than ~60% of a build.
- Content: Ragefire Chasm, the Deadmines, Wailing Caverns, Shadowfang Keep, Blackfathom Deeps.

**Exit:** each class plays 1-25 without dead time; the statistics sit in their target bands;
Jeff wants to keep playing.

### M2: five classes

- Warlock and Shaman built to the same depth: skills and skill trees, passive web, keystones,
  uniques, sets, synergies, Codex pages.
- Each class gets three builds designed up front and checked in play.
- The class screen shows all five dressed and standing correctly.

**Exit:** all five classes reach 25 with three working builds each.

### M3: the journey to 60, in acts

- The acts: vanilla's zones grouped into five or six acts by level, each with a hub town, its
  zones in order, its dungeons, and a closing boss; the act's route shown on the map and in a
  journal (the story's next place, not quest markers).
- A zone-by-zone pass: pack density, events and bounties, so no stretch of levelling is empty.
- Every dungeon to Stratholme and Scholomance gets its Warden, Cache and tier scaling.
- Item bands: sets and uniques for each ten levels to 60; affix pools by level.
- Levelling pace measured against the 1.3-1.5x target.

**Exit:** a fresh character of each class reaches 60 in the target time; every dungeon is
cleared on Normal and Brutal.

### M4: the endgame

- Difficulty tiers through Torment, each with its rewards.
- The raids one at a time, Molten Core to Naxxramas, each with its attunement and solo tuning.
- Paragon past 60; a weekly goal; crafting that finishes an item (reroll, upgrade); a stash with
  sorting.
- Chase uniques for each class.

**Exit:** three builds per class clear Naxxramas on Torment I; a character still finds upgrades
40 hours past 60.

### M5: presentation

- UI pass: inventory, stash and comparison, the skill and web windows, tooltips, the map.
- Effects pass: each skill and rune with its own look, telegraphs, champions, loot and deaths.
- Audio pass: skills, impacts, kills, loot, UI.
- First ten minutes: the class screen, a first fight that teaches the roll and the flask through
  play, every window explaining itself (no markers or glows, as decided).
- Performance: a frame budget in big pulls, options for slower machines.

**Exit:** a new player starts without being told anything; no placeholder left; the frame budget
holds in the densest fights.

### M6: release 1.0

- Stability: crash reports collected automatically, long soak tests, server restarts that lose
  nothing, character backups.
- One installer and updater for client and server.
- A final balance pass from everyone's play statistics.
- Ready for friends: each runs the same installer and their own server, as Jeff does.

**Exit:** the "What finished means" list above is all true, and the known issues hold no blocker.

## How the work runs

- **One thing at a time, measured.** Every tuning change is read back in the play statistics.
- **Feel first.** Combat feel and difficulty come before content volume: content built on an
  untuned core gets retuned twice.
- **Every big change reviewed** before it is pushed; docs kept current.
- **Jeff plays every milestone.** His sessions are the test, and his stats are the evidence.

## Jeff's decisions

1. Five classes are final for 1.0.
2. 1.0 is for Jeff and friends, not a public release.
3. Solo: no co-op in 1.0.
4. Hardcore or seasons: probably never; not planned.
5. Levelling in acts: a route through the zones, each act ending in a boss.
6. Big stacking is welcome: builds may multiply into enormous damage (kit products, stacked "more",
   pack-squared bursts, kill resets, spread chains). Don't cap power for its own sake; fix only
   what breaks the game (crashes, hangs, a loop that never ends).
