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
| 12–29 | 3–4 | First tree nodes (Wide Swing, Sweeping Seal), a second and third slot |
| 30+ | 3–5 | Area damage, life on kill, Hallowed Ground |

**A camp is a pack already.** Vanilla's camps (a gnoll camp in Redridge, a murloc shore) are
spawns standing side by side, and each one leading its own pack turned eight gnolls into thirty.
So the count follows the spawns already there, within 18 yards:
- 4 or more of normal rank: the mob leads no followers. A champion pack still comes, with two
  champions.
- 2 or 3: it leads at most one follower.
- Fewer: a lone wanderer still leads a full pack.

**Outriders: twice the packs.** Vanilla spaces its spawns for slow fights, and an ARPG clears
them several times faster, so the open world felt empty between camps (Jeff, after play: "the
amount of packs" was too low, not their size). Each pack a vanilla spawn forms also places
`Arpg.Packs.Extra` more (1 by default, so twice the packs; a fraction is a chance): a copy of the
spawn, 22–45 yards off on walkable ground, leading a pack of its own. A spot is refused within 15
yards of another creature, within a player's sight (the map's visibility distance), within 50
of a town or any NPC friendly to players, or in a
grid not yet loaded (loading one would form its own spawns' outriders, and so on across the
map). Outriders are temporary like followers: the spawn's next formation (its respawn) or its
grid unloading sends them away once they're out of combat. They give full XP and loot. Every
follower also checks on its pack every 20 seconds and leaves, out of combat, once its leader is
dead or gone: a pack left half-killed no longer stands for good.

**Followers** are the leader's own kind, one or two levels lower, drawn a little smaller, with
about 55% of its health and damage: a pack of five fights like about three ordinary mobs. Each
gives about 30% of the XP, so a pack of four is worth about two kills, and area damage kills it
far faster: about 1.3 to 1.5 times vanilla's pace. Quest items drop from followers as usual; their
gold and gear chances are reduced.

**How they spawn:** at runtime, not in the database. When an eligible creature spawns, it may
lead a pack: its followers are temporary summons that follow its path, share its aggro, despawn
with its corpse and come back when it respawns. Pulling one pulls the rest of its pack within 20
yards that can see it (a whole camp across 40 yards used to come at once). Eligible: hostile, normal rank, no NPC flags, not
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
leader's kind. A champion has 3x health, +30% damage, 3x XP, extra gold and a 40% chance at a green
or blue; a rare 4x health, +50% damage, 5x XP, a green or blue and a 3% chance at a purple.

**Loot rates** (Jeff, after play: about one purple and one unique every other dungeon run, all
sources together): rolled drops are purple 0.3% from trash, 1% from champions and 3% from rares
and bosses; a vanilla blue (a named dungeon drop) stays a unique 12% of the time and otherwise
drops as a rolled blue of its level (an authored unique, with a mechanic of its own, always drops
as itself); a Warden's purple is 12% and unique 10%, a Cache's 15% and
12%. Raid epics are untouched. Nine affixes: Extra
Strong, Extra Fast, Stone Skin, Fire Enchanted, Cold Enchanted, Vampiric, Thorns, Teleporter,
Healer; Molten waits for a ground visual. The server sends `SMSG_ARPG_CHAMPIONS` (0x341) for the
champions within 100 yards; the client (`player/arpg/champions.rs`) colours the hover bar's name
blue or yellow, lists the affixes under it, lays the game's selection ring under each one in that
colour, slowly turning, and dresses it in a 1.12 spell's aura per affix (Fire Shield for Fire
Enchanted, Frost Armor for Cold Enchanted, Stoneform for Stone Skin, Enrage for Extra Strong,
Sprint for Extra Fast, Demon Armor for Vampiric, Thorns, Mana Shield for Teleporter, Renew for
Healer; drawing only, logged at first use as "champions' X wears spell N's aura"). The
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
per group member within 100 yards; world bosses as a 40-player raid. All of it is on top of the
power scale (`ARPG-CHARACTER.md`, "Damage"): every creature's health is vanilla's × the ARPG's
damage size at its level × (1 + 0.5 × level ÷ 60), and a raid's again × (1 + (its item level − 60)
÷ 20), Molten Core ×1.3 to Naxxramas ×2.4. A creature is scaled a moment
after it spawns and again when a fight starts with a different player count or tier, keeping its
share of health. Caps, melee / spell (DoT ticks as melee): trash 6% / 12%, champions, rares and
open-world elites 9% / 15%, dungeon bosses 12% / 30%, raid and world bosses 15% / 35%; ordinary
open-world mobs are uncapped. The tier, the level gap and the difficulty raise the caps, but never
past a ceiling for anything but a boss: trash 12% / 20%, champions, rares and open-world elites
16% / 25% (2026-10-21: Razorfen Kraul's Geomancers had been landing Lightning Bolts for 40% of a
mage's health on a high tier). **The level gap** moves every creature hit on an ARPG player and its
cap: 10% less per level the player is above the creature (to 40%), 5% more per level below (to
+25%). Without it a cap that follows the player's own health made out-levelling a dungeon do
nothing: Van Cleef at 24 hit as hard, as a share of health, as at 18. A boss is rank 3, a ScriptDev `boss_` script, or in a dungeon a
health multiplier of 5 or more (6 from level 40): checked against the database, vanilla trash
peaks at 3 in low dungeons and 6 in Stratholme and Blackrock Spire, while bosses run 5 to 25.
Dungeon trash rolls champions (6%) and rares (1.5%) per mob from level 8. Enrage timers (built):
a dungeon boss still fighting after 4 minutes, a raid or world boss after 6, enrages (+50%
damage and cap, a stack more every 30 seconds, up to ten), and the players there are told. The
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
  two blues, a 12% chance at a purple and 10% at a unique. Its own unique and the Codex page come with step 5.
- *Caches:* each final boss (`Arpg::Bosses`) drops gold, and per player there a blue, a green and
  a 15% chance at a purple and 12% at a unique, announced in chat.
- *Escorts:* dungeons are spaced for five slow players, so each trash creature (never a boss, a
  scripted creature or a formation's member) leads one or two lesser copies of itself at
  `Arpg.Packs.Dungeon`'s chance (0.6). Not in Stratholme, Scholomance or Blackrock Depths, whose
  scripts count their trash's deaths; an escort whose leader falls stays to be killed. Pack and
  champion records are kept per map copy, as a dungeon's guids repeat in every copy.
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

- **Roll (Space, the Jump binding):** a dash of 7.5 yards along the ground toward the movement
  keys' walk, or the cursor with none held, every 2.5 seconds. The server sends it as a charge
  spline the client rides on the ground under it, so a small rise never cuts it short and it
  stops at a wall or a drop; for 0.45 s every blow and hostile spell at the player is dodged. Not while rooted, stunned, feared, mounted, swimming or falling. A bar
  right of the skill bar shows the cooldown.
- **Short crowd control:** a creature's stun, root, fear, confuse, charm or silence holds an ARPG
  player at most 2 seconds (`Arpg::CapControl`, `Arpg/ArpgDungeons.cpp`, applied in
  `Spell.cpp` after diminishing returns). Vanilla's 8-second Frost Nova (a Cold Enchanted
  champion's, or a caster mob's) kept the player rooted far longer than an ARPG fight allows.
- **Flask (Q, the Strafe Left binding; A still walks left):** three charges, starting full. A
  charge heals 15% of maximum health at once and 25% more over three seconds. Kills refill it
  (a follower 1 point, a mob 2, an elite 4, a champion 5, a rare 10, a boss 20; ten points a
  charge), and out of combat a charge returns every 12 seconds. Vials left of the skill bar show
  the charges and the next one filling.
- The server sends `SMSG_ARPG_STATUS` (0x342) on a change: the charges, the next charge's
  progress and the roll's cooldown.
- **Town portal (T, or the button right of the roll):** out of combat, two seconds standing still
  open a portal home (the hearth's bind point). In town (resting, or near the bind point) the
  same key goes back through it to the spot it opened from, into the same dungeon, for 30
  minutes. Moving, a blow or combat in the two seconds closes it. T is only taken if nothing else
  is bound to it; the key bindings window lists it under ARPG.
- **Death:** releasing raises the character at once at half health and mana, at the dungeon's
  entrance inside the dungeon, or at the nearest graveyard outside. No ghost walk, no corpse run;
  the corpse turns to bones. The durability loss stays.
- **Junk:** a grey item with a sell price turns into its price in gold when picked up, with a
  line in chat, so the bags never fill with vendor trash.

## Threats and the roll

Built (`Arpg/ArpgThreats.{h,cpp}` on the server, `player/arpg/threats.rs` and
`entities/missile.rs` on the client). The roll is only worth having if there is something to
roll out of, so the monsters' dangerous attacks are ones a player can see coming and avoid.

- **Bolts fly at a point.** A hostile creature's travelling spell at an ARPG player (a Frostbolt,
  a Shadow Bolt, an archer's arrow: one target, no area) no longer follows the player. It flies at
  the spot the player stood on when it left and hits only a player still on its line (within 2
  yards, body included) when it arrives. Sidestepping a bolt from range works; one from up close
  arrives too soon to. Rolling through one dodges it whatever. A bolt that misses strikes the
  ground where the player stood, and the combat text says Miss or Dodge.
- **Telegraphs.** Open-world elites, champions, rares, dungeon and raid bosses wind up a heavy
  attack every 8 to 15 seconds: they stop, roar, and the ground shows where it lands (an orange to
  crimson area with a bright edge and a fill that grows to the edge as the wind-up runs, 1.2 to
  1.7 seconds). Three shapes: a ring round the creature, a 90° cone at the player, a circle at the
  player's feet (a caster's). A boss cycles all three; any other creature keeps one.
- It lands on every ARPG player still inside for a share of their maximum health: 20% from an
  elite, 25% a champion, 30% a rare, 35% a dungeon boss, 45% a raid boss. That is past the
  per-hit damage caps, on purpose: it is always avoidable, by walking out or rolling. Absorbs and
  immunities (Divine Shield) still count.
- A stun, fear or confuse on the creature during the wind-up breaks it off (the mark greys out):
  Hammer of Justice is the answer to a telegraph. A creature being kited waits for the player to
  come back in reach.
- Dungeon trash does not telegraph (it is elite throughout, and a pull of five would be a carpet of
  marks), only its champions and rares and the bosses.
- The champions' own dangers are marked the same way: Fire Enchanted's death burst (a 6 yard
  circle on the corpse, 1.2 seconds, 15% of maximum health) and Cold Enchanted's Frost Nova at 30%
  health (a 10 yard ring round it, 1 second, then the nova). Rolling dodges both.
- The marks lie over the ground under them (each point set down on the floor or terrain), so a
  hillside or a stair shows the whole shape.
- A circle also burns with Consecration's ground model (`Spells\Consecration_Impact_Base.m2`)
  scaled to its radius, so it reads as a spell on the ground; the fill still times the wind-up.
- **Every creature has a move** (amber marks, lighter than an elite's): short wind-ups of 0.6 to
  1 second, about a tenth of the player's health, every 8 to 13 seconds, so a pack keeps the
  player moving. By what it is:
  - wolves, cats, raptors, hyenas, birds, bats: a lunge down a line, leaping at the end;
  - boars and tallstriders: a charge down a longer line;
  - bears, gorillas, crocolisks: a maul in a cone;
  - spiders: a web at the player's feet that slows; scorpids: a stinging cone that slows;
  - crabs, turtles, elementals, demons, giants: a slam ring;
  - kobolds: a candle bomb thrown at the player's feet;
  - humanoids and undead: a cleave cone, a rogue's dash down a line, and a caster backs away
    from a player who closes in, to cast again (its bolts can be sidestepped);
  - murlocs: the first one pulled brings every murloc within 22 yards.
  Dungeon trash, which has no heavy attack, has its move too; elites, champions and bosses keep
  their heavy attacks instead.
- **Health globes:** a kill sometimes drops a red orb where the creature fell (a rare or boss
  always, a champion half the time, anything else 7%, 15% while the killer is under half health);
  walking onto it heals a fifth of maximum health. It fades after a minute.
- **The weight of a blow** (client only): a crit or a heavy hit freezes both models for a few
  frames; every hit pushes the struck model back along the blow; a heavy killing blow flings the
  corpse a couple of yards on a short arc; and every kill leaves the creature's own blood splat on
  the ground (the textures 1.12 ships and never draws), fading over half a minute.
- `SMSG_ARPG_TELEGRAPH` (0x344): the wind-up (shape, grade, place, facing, radius, wind-up time)
  and its breaking off.

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

**Built (Molten Core):** the Core Sigil's three pieces drop on Brutal or harder, one per ARPG
player at the kill, and go into the character's Codex (no bag space); the third fuses them, and
the Sigil is permanent. Below Brutal the boss's kill says what it would have dropped. An ARPG player
without the Sigil can't enter Molten Core, and ARPG players need no raid group for any raid
(`Player::GetAreaTriggerLockStatus`). The other raids keep their vanilla requirements for now.

The pieces are new items, which the server writes into `item_template` itself (`Arpg/ArpgCodex.h`). An attunement
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
- **Bounties** (built, below): three at a time, carried rather than taken from a board.

## Zone events and bounties

Jeff's call: the world should feel alive, with no daily quests. Quests stay optional; bounties
and kills carry levelling.

**Zone events** (`Arpg/ArpgEvents.{h,cpp}` on the server, `player/arpg/world.rs` and the HUD's
tracker on the client). In the open world (Eastern Kingdoms and Kalimdor), a director starts one
event per zone, 45-90 seconds after an ARPG player arrives and 3-6 minutes after the last ended.
Each starts 45-130 yards from a player who is out of combat, not resting and level 5 or more.
It uses the zone's own creatures: it anchors on a plain hostile creature within 4 levels below to
1 above the player's level (no script, no quest or vendor ties, not a champion or rare) and
summons more of its kind.
- *Purge* (45%): slay 10-14 enemies within 45 yards of the spot in 3 minutes. Half as many again
  of the anchor's kind fill the camp out, as lesser followers.
- *Hunt* (30%): a named rare of the anchor's kind, with two lesser followers, roams the spot.
  Slay it in 5 minutes. The rare is no higher than your level and has one affix fewer than a
  pack's rare (one under level 30, two from 30).
- *Siege* (25%): three waves (4, 5, then 6 led by a champion) of lesser followers run at whoever
  holds the spot.

The summons fight as a pack's followers do: a level or two under the event's level, at a pack
follower's share of health and damage.
  Clearing a wave brings the next after 5 seconds; otherwise each comes 30 seconds after the last.
  4 minutes in all.

The final kill's corpse holds a cache: a green, a blue, a 20% purple, an 8% unique, 2-4 orbs and
gold. Every ARPG player who fought in the event, or stands within 120 yards, gains 3% (purge),
3.5% (hunt) or 4% (siege) of a level (gold at 60) and the event's bounty credit. Out of time, the
event fails and its summons go. The zone hears each start, wave, finish and failure in chat.

The client shows the event in a tracker under the minimap: the objective and its progress, the
clock, and the distance and compass way to the spot ("120 yd NE", or "you are there"). A tall
orange column of light stands over the spot until the event ends.
`SMSG_ARPG_EVENT` (0x349): `u8` state (1 live, 2 done, 3 failed, 4 none), `u32` id, `u8` kind,
`f32` x y z, `f32` radius, `u16` have and need, `u32` ms left, cstring title and objective.

**Bounties** belong to the zone you are in. Each zone gives three, which the tracker heads with
the zone's name:
- complete 2 of its events (10% of a level);
- slay 20-30 of its creatures (none grey; 7%);
- slay 2-4 of its champions or rares (8%), or, 35% of the time when a dungeon suits the zone's
  level, defeat that dungeon's final boss (15%).

(Their experience was cut to a third on 2026-10-22: kills alone came near the pace Jeff chose,
and the bounties doubled it; `ARPG-DECISIONS.md`, "Tuning from play".)

They are kept per zone, so going back finds them as you left them. A city, or a zone you have
outlevelled by more than 5, has none. A dungeon keeps the bounties of the zone you entered it
from. Nothing is handed in: a finished bounty pays at once, then a new one of its kind takes its
slot. It pays:
- that share of a level in XP;
- gold (level² × 20, × 40 for a dungeon);
- 1-3 orbs;
- a rolled item: green, blue or purple. When the bags are full it comes by mail.

Group members within 100 yards share kill credit. Right-click a bounty twice in the tracker to
abandon it for another (`CMSG_ARPG_ACTION` kind 25, `u8` slot). Saved in
`character_arpg_zone_bounty` (by zone), which the server creates. `SMSG_ARPG_BOUNTIES` (0x34A): `u8` count, per bounty `u8` slot, `u8`
kind, `u16` have and need, cstring title and reward.

## Nightmare dungeons

Diablo IV's nightmare dungeons, as seeded runs (cmangos `Arpg/ArpgDungeons.h`, 2026-10-21): from
**Brutal** up, each dungeon run rolls affixes when its tier is chosen, so no two runs of the same
dungeon play alike.

- **Banes** on its monsters: one on Brutal, two on Torment I, three on Torment II and III.

  | Bane | Effect |
  |---|---|
  | Bursting | Every dead foe (not critters or friends) bursts into flame a moment later (half a Fire Enchanted burst) |
  | Frenzied | Enemies attack 25% faster |
  | Stalwart | Enemies have 30% more health |
  | Vengeful | Enemies deal 20% more damage |

- **One blessing** for its players:

  | Blessing | Effect |
  |---|---|
  | Shrines | Up to twice as many shrines, found three times as often |
  | Bloodlust | Kills heal 3% of your health |
  | Fortune | The Cache holds a blue more for each player, and 15% more chance at a unique |
  | Thrift | Skills cost 25% less mana |

- **The reward:** each bane gives the run 10% more experience, and the Warden and the Cache 5%
  more chance at a purple and at a unique.
- **Shown:** the map is told the nightmare when the tier is set ("Nightmare: Bursting (the dead
  burst ...). Blessing: Bloodlust (...)."), and the affixes stand in the HUD's boon column for as
  long as the player is inside, banes in red, the blessing in green.

## Shrines

Diablo II's and IV's shrines (cmangos `Arpg/ArpgShrines.h`, 2026-10-21): a stone that gives a
short, strong boon to whoever walks onto it.

- **Where:** beside 12% of the packs placed ahead in the open world, and in a dungeon beside its
  trash (1 in 40 settling creatures, at most 2 an instance). An open-world shrine left alone goes
  after 10 minutes.
- **The touch:** stepping within 3 yards of it. It shows its name on hover but can't be clicked,
  so it needs nothing of the client; it vanishes with a flash of light, the boon goes on the HUD
  beside the health orb with its time (`ARPG-CHARACTER.md`, "Barrier and boons"), and the chat
  says what it does.
- **The kinds:**

  | Shrine | Time | Boon |
  |---|---|---|
  | Wrath | 20 sec | 50% more damage |
  | Protection | 10 sec | A Barrier of half your health, filled again each second |
  | Greed | 30 sec | Your kills' loot rolls its rarity as three tiers higher, a 30% chance of an extra item, three times the gold |
  | Celerity | 15 sec | Swiftness (50% faster movement) and 25% faster attacks |
  | Channeling | 15 sec | Your skills cost no mana |
  | Storms | 20 sec | An arcane nova around you each second in combat |

- **Built from:** generic game objects written into `gameobject_template` at start (entries
  98000 up), each wearing a vanilla shrine's or altar's model; the server log names the look each
  got ("Shrine of Wrath wears the look of ..."). Only rows named "Shrine of ..." are rewritten: an
  entry the world database uses for something else is left alone, and that kind of shrine never
  appears (the log says so).

## Build order

1. Packs: level-scaled sizes, lesser followers, shared aggro, the dev spawn tool, the fight log.
   (Built.)
2. Champions and rares: affixes, the client's names and rings. (Built.)
3. Group scaling and damage caps, in dungeons first. (Built, but not the enrage timers.)
4. Dungeon Wardens and Caches. (Built, but not the Warden's own unique.)
5. Codex pages and runes, dropping from champions, Wardens and Caches. (Built:
   `ARPG-CHARACTER.md`, "Drops that unlock skills".)
6. Difficulty tiers. (Built.)
7. Raid attunements and Molten Core's small-group forms; the other raids one at a time. (Molten
   Core's attunement built.)
8. Zone events and bounties. (Built.) A paragon track past 60 (`ARPG-CHARACTER.md`'s open
   question), and a weekly raid objective (Jeff deferred it).

## Open questions for Jeff

None open: see "Jeff's calls so far".
