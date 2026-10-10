# ARPG progression: packs, dungeons, raids and quests

How a character goes from level 1 to Naxxramas on this server, alone or in a small group of ARPG
players. Read `ARPG-CHARACTER.md` first: this doc sets the fights that the character's power is
built for.

## Jeff's calls so far

- Only ARPG players play here, alone or in small groups: no stock-client or bot balance to keep.
- Packs of 3 to 5, sized to what the paladin can kill at each level.
- Pack followers are smaller, easier mobs and give less XP.
- Dungeons are special: a challenge with a reward.
- Levelling at about 1.3 to 1.5 times vanilla's pace.
- Raids need progression: Molten Core needs items from endgame dungeons.
- Bosses must be soloable: no one-shots from auto attacks, so damage scales somehow.
- No quest glows, markers or any other hand-holding.
- The damage caps below are the first pass; raid keys are permanent attunements; each dungeon's
  Warden is drafted as it's built, for Jeff to veto.

## Packs

**Size grows with the character**, because the paladin's area damage does:

| Level | Pack size | The paladin by then |
|---|---|---|
| 1–5 | 1–2 | The swing and Seal of Righteousness |
| 6–11 | 2–3 | Consecration (level 6), one skill slot |
| 12–19 | 3–4 | First tree nodes (Wide Swing, Sweeping Seal), a second slot |
| 20+ | 3–5 | Area damage, life on kill, Hallowed Ground |

**Followers** are the leader's own kind, one or two levels lower, drawn a little smaller, with
about 55% of its health and damage: a pack of five fights like about three ordinary mobs. Each
gives about 30% of the XP, so a pack of four is worth about two kills, and area damage kills it
far faster: about 1.3 to 1.5 times vanilla's pace. Quest items drop from followers as usual; their
gold and gear chances are reduced.

**How they spawn:** at runtime, not in the database. When an eligible creature spawns, it may
lead a pack: its followers are temporary summons that follow its path, share its aggro, despawn
with its corpse and come back when it respawns. Eligible: hostile, normal rank, no NPC flags, not
a pet, summon, rare or boss. Settings: `Arpg.Packs.Enable`, sizes, champion and rare chances.

**Champions and rares:**

| Tier | What | How often | Reward |
|---|---|---|---|
| Pack | Leader and followers | Most spawns | Normal |
| Champion pack | 2–3 champions sharing one or two affixes, and followers | About 1 in 8 | Bonus loot, later Codex fragments and runes |
| Rare | One named leader with 2–3 affixes, and followers | About 1 in 40 | Blue or better, later fragments and a rune chance |

Champions have about three times the health and a larger model; their names show blue, rares'
yellow. Affixes, each from stats, auras or a small hook: Extra Strong, Extra Fast, Stone Skin, Fire
Enchanted (fire on hit, a burst on death), Cold Enchanted (slows; a nova at low health), Vampiric,
Thorns, Teleporter, Healer (heals its pack), Molten (a burning trail). The client gets each
champion's tier, name and affixes and shows them on the hover bar, with a ring under its feet.

**Tuning:** a dev tool forms a pack round the nearest mob, and the server logs each pack fight's
time to kill and damage taken, so the numbers can be set from play.

**Built (step 1):** `Arpg/ArpgPacks.{h,cpp}` on the server. Packs form a second or so after a
leader is added to its map (a grid loading, a respawn); followers are temporary summons of the
leader's entry, placed behind it, following a patrolling leader or wandering near a wandering
one. Settings: `Arpg.Packs` (on), `Arpg.Packs.FollowerPower` (0.55), `Arpg.Packs.FollowerXp`
(0.3), `Arpg.Packs.FollowerLoot` (0.5). The ARPG View page's **Form Test Pack** button (with
`Arpg.DevTools = 1`) makes the nearest mob lead a pack of five; the server log reads
"ARPG packs: a pack of 5 Kobold Vermin (level 3) fell in 14.2 s; it dealt 312 damage".
Known gaps: a follower that evades walks back to where it appeared rather than to its leader.

**Built (step 2):** champions and rares, from level 8, in `Arpg/ArpgPacks.cpp`. About 1 pack in 8
brings two or three champions sharing one affix (two from level 30); about 1 in 40 brings a rare
with a name of its own and two affixes (three from level 30). Both are extra summons of the
leader's kind. A champion has 3x health, +30% damage, 3x XP, extra gold and a green or blue; a
rare 4x health, +50% damage, 5x XP, a blue and a 15% chance at a purple. Nine affixes: Extra
Strong, Extra Fast, Stone Skin, Fire Enchanted, Cold Enchanted, Vampiric, Thorns, Teleporter,
Healer; Molten waits for a ground visual. The server sends `SMSG_ARPG_CHAMPIONS` (0x341) for the
champions within 100 yards; the client (`player/arpg/champions.rs`) colours the hover bar's name
blue or yellow, lists the affixes under it, and lays a ring of that colour under each one. The
ARPG View page gains **Champion Pack** and **Rare Pack** dev buttons.

## Scaling for one to five players

WoW's encounters assume a tank, a healer and three damage dealers; here a fight has one to five
players, each of them all three at once. Three rules make that work in dungeons and raids alike.

**1. Health scales with the players inside.** A creature's health is set when the first player
engages it, by the number of players in the instance: solo about 35% of vanilla's in a dungeon,
then about +16% per player. Raids scale from a solo baseline the same way (see below).

**2. Damage is measured against the player, not in flat numbers.** In vanilla a boss swing is
sized for a tank in raid gear: Ragnaros's melee would one-shot a solo paladin whatever the
multiplier. So a hostile creature's hit in a dungeon or raid is capped at a share of the
victim's maximum health:

| Source | Cap per hit |
|---|---|
| Trash and followers | 6% |
| Champions and rares | 9% |
| Dungeon bosses | 12% |
| Raid bosses, melee | 15% |
| Avoidable boss abilities (fire on the ground, a telegraphed blast) | 35–50% |

Auto attacks wear you down, and life on kill, leech, blocks and heals hold them off; what kills
you is standing in what you should have dodged. The caps follow the player's gear on their own,
so they never need retuning per level. Difficulty tiers raise them.

**3. Mechanics that need a raid get a small-group form.** Each boss whose fight needs several
players gets a few lines of change, kept small so the fight stays recognisable. Molten Core as the
model:

| Boss | Vanilla needs | Small-group form |
|---|---|---|
| Lucifron | Decursers, adds tanked | Curses fade in Consecration or after 10 sec; one guard per player |
| Magmadar | Fear ward, a tank | The fear is shorter; Lava Bombs are the avoidable damage |
| Gehennas | Decursers | As Lucifron |
| Garr | Banishers for eight adds | One add per player, which explode on death (dodge them) |
| Baron Geddon | Living Bomb spread out | The bomb shows a ring for 4 sec: get away from the pack |
| Shazzrah | Ranged spread, decursing | Blinks to you; Arcane Explosion is the avoidable damage |
| Sulfuron Harbinger | Four priests interrupted | Two priests, plus one per extra player; any stun interrupts |
| Golemagg | Two dog tanks | The dogs share his damage cap; stand out of Earthquake |
| Majordomo | Crowd control for eight adds | Four adds plus one per extra player, killed in any order |
| Ragnaros | Tank swaps, sons phase | Wrath of Ragnaros knocks back (avoidable); sons scale with players |

Dungeon bosses mostly need only rules 1 and 2. Some get a line or two where a mechanic assumes
five players.

**Built (rules 1 and 2):** `Arpg/ArpgDungeons.{h,cpp}` on the server (`Arpg.Dungeons`, on).
Health: dungeons `Arpg.Dungeons.SoloHealth` (0.35) plus `Arpg.Dungeons.PlayerHealth` (0.16) per
extra player; raids 0.35 × 9 / the raid's size solo (about 8% for Molten Core), each extra player
adding three quarters of that; open-world elites `Arpg.Dungeons.EliteSoloHealth` (0.5) plus 0.16
per group member within 100 yards; world bosses as a 40-player raid. A creature is scaled a moment
after it spawns and again when a fight starts with a different player count or tier, keeping its
share of health. Caps, melee / spell (DoT ticks as melee): trash 6% / 12%, champions, rares and
open-world elites 9% / 15%, dungeon bosses 12% / 30%, raid and world bosses 15% / 35%; ordinary
open-world mobs are uncapped. A boss is rank 3, a ScriptDev `boss_` script, or in a dungeon a
health multiplier of 5 or more (6 from level 40): checked against the database, vanilla trash
peaks at 3 in low dungeons and 6 in Stratholme and Blackrock Spire, while bosses run 5 to 25.
Dungeon trash rolls champions (6%) and rares (1.5%) per mob from level 8. Enrage timers and the
small-group boss forms are not built yet.

**Enrage timers keep a damage check:** about 4 minutes solo for a dungeon boss, about 6 for a raid
boss, so the character's damage has to keep up with the dungeon tier.

## Dungeons

1. **Group scaling** (above).
2. **Every pack is a real pack:** vanilla's trash groups stay as they are, with champion and rare
   rolls at three times the open world's rate. Followers don't add to them.
3. **A Warden in every dungeon:** a fixed, named rare with themed affixes ("Gorehowl the
   Fleshrender", Vampiric and Extra Strong, in the Deadmines) and its own drops: the dungeon's
   Codex page chance and a themed unique.
4. **A Cache at the final boss:** loot for the character's level and the tier. The first kill
   also gives a passive point (built).
5. **Difficulty tiers:** Normal, Hard, Brutal, then Torment I, II and III. Each raises health,
   the damage caps and champions' affix count, for better loot quality, more drops and higher rune
   and Codex chances. A clear unlocks the next tier for that dungeon. The tier is chosen at the
   entrance by whoever enters first.

**Built (Wardens, Caches, tiers):** in `Arpg/ArpgDungeons.cpp`.
- *Wardens:* each vanilla dungeon has a themed one (Gorehowl the Fleshrender, Vampiric and Extra
  Strong, in the Deadmines; Forgemaster Ironmaw, Fire Enchanted and Stone Skin, in Blackrock
  Depths; nineteen in all). One trash spawn per instance, among those that load (each eligible
  one at 1 in 12 as it settles, the 25th surely, so it stands in the wing the group is in), is
  crowned a named rare; a chat line announces it and its fall. It drops
  two blues and a 30% chance at a purple. Its own unique and the Codex page come with step 5.
- *Caches:* each final boss (`Arpg::Bosses`) drops gold, and per player there a blue, a green and
  a 25% chance at a purple, announced in chat.
- *Tiers:* the ARPG View page's **Dungeon Tier** slider (`arpgDungeonTier`, sent at each world
  entry and on a change, `CMSG_ARPG_ACTION` kind 19). An instance takes its first ARPG player's
  tier, as far as that player has it open there, and says so in chat; a final boss's kill opens
  the next tier for everyone in at the kill (`character_arpg_tier`, which the server creates).

  | | Normal | Hard | Brutal | Torment I | Torment II | Torment III |
  |---|---|---|---|---|---|---|
  | Health | ×1 | ×1.4 | ×1.9 | ×2.6 | ×3.4 | ×4.4 |
  | Damage | ×1 | ×1.15 | ×1.3 | ×1.5 | ×1.7 | ×2 |
  | Caps | ×1 | ×1.2 | ×1.4 | ×1.65 | ×1.9 | ×2.2 |
  | XP | ×1 | ×1.15 | ×1.3 | ×1.5 | ×1.7 | ×2 |
  | Champion and rare chance | ×1 | ×1.25 | ×1.5 | ×1.75 | ×2 | ×2.5 |
  | Extra affixes | 0 | 0 | 1 | 1 | 2 | 2 |
  | Warden and Cache drops | | +2 item levels, +8% purple per tier; an extra blue per two tiers | | | | |

## Survival: the roll and the flask

Built (`Arpg/ArpgActions.{h,cpp}` on the server, `player/arpg/actions.rs` on the client).

- **Roll (Space, the Jump binding):** a leap of about seven yards toward the movement keys' walk,
  or the cursor with none held, every 2.5 seconds. The server sends it as a knockback, which the
  client flies and the anticheat expects; while airborne (0.45 s) every blow and hostile spell at
  the player is dodged. Not while rooted, stunned, feared, mounted, swimming or falling. A bar
  beside the power orb shows the cooldown.
- **Flask (Q, the Strafe Left binding; A still walks left):** three charges, starting full. A
  charge heals 15% of maximum health at once and 25% more over three seconds. Kills refill it
  (a follower 1 point, a mob 2, an elite 4, a champion 5, a rare 10, a boss 20; ten points a
  charge), and out of combat a charge returns every 12 seconds. Vials beside the health orb show
  the charges and the next one filling.
- The server sends `SMSG_ARPG_STATUS` (0x342) on a change: the charges, the next charge's
  progress and the roll's cooldown.

## Raids: progression gated by dungeons

Each raid has an attunement: a key made from pieces that endgame dungeons drop at a set tier.
It's per character and permanent, as vanilla's attunements are.

| Raid | Key | Pieces |
|---|---|---|
| Molten Core | Core Sigil | Ember of Thaurissan (Blackrock Depths, Emperor, Brutal), Spire Brand (Lower Blackrock Spire, Wyrmthalak, Brutal), Drakkisath's Seal (Upper Blackrock Spire, Brutal) |
| Onyxia's Lair | Drakefire Amulet (vanilla's chain) | Its quest line, with General Drakkisath on Brutal |
| Blackwing Lair | Blackhand's Command (vanilla) | Molten Core cleared, and Upper Blackrock Spire on Torment I |
| Zul'Gurub | Hakkari Totem | Zul'Farrak and Sunken Temple Wardens on Brutal |
| Ruins and Temple of Ahn'Qiraj | Scarab Sigil | Blackwing Lair cleared, and Dire Maul's three Wardens on Torment I |
| Naxxramas | Argent Writ | Ahn'Qiraj cleared, and Stratholme and Scholomance on Torment III |

The pieces are new items, shipped as a fork SQL file the server update applies. An attunement
quest, given by the vanilla quest givers where they exist (Lothos Riftwaker for Molten Core),
turns the pieces into the key. The raid's entrance checks for the key (the area trigger's
required item, or a check in the instance entry for ARPG players).

**Raid sizes:** health is set for one player (about 8% of vanilla's 40-player value for Molten
Core), then rises per extra player. Loot comes as one drop per player, and the boss's Cache scales
with the group.

**Why this order works:** levelling gets a character to Brutal Blackrock dungeons; Brutal gear and
uniques make Molten Core possible; Molten Core's gear makes Torment I possible, which opens
Blackwing Lair. Every raid needs the dungeons one step below it, so dungeons stay worth running
the whole way.

## Quests

- **Vanilla's quests as they are**, stories and all. Packs make "kill 10" quests quicker.
- **No hand-holding:** no glows, markers or objective arrows; the quest text and the map are the
  guide, as in vanilla.
- **Open-world elite quests** scale with the group, as dungeons do.
- **Attunement quests** for the raids (above).
- **Bounties (later):** a board in each capital with a few zone tasks ("kill the rare in Felwood",
  "clear Scholomance on Hard"), each paying a Cache, so level 60 zones stay worth visiting.

## Build order

1. Packs: level-scaled sizes, lesser followers, shared aggro, the dev spawn tool, the fight log.
   (Built.)
2. Champions and rares: affixes, the client's names and rings. (Built.)
3. Group scaling and damage caps, in dungeons first. (Built, but not the enrage timers.)
4. Dungeon Wardens and Caches. (Built, but not the Warden's own unique.)
5. Codex pages and runes, dropping from champions, Wardens and Caches. (Built:
   `ARPG-CHARACTER.md`, "Drops that unlock skills".)
6. Difficulty tiers. (Built.)
7. Raid attunements and Molten Core's small-group forms; the other raids one at a time.
8. Bounties, and a paragon track past 60 (`ARPG-CHARACTER.md`'s open question).

## Open questions for Jeff

None open: see "Jeff's calls so far".
