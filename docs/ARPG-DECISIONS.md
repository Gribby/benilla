# ARPG decisions and research

The record of what Jeff has decided, the research behind the design, and the tuning changes made
from play. **Read this before planning any ARPG work**, and add to it whenever Jeff decides
something or a piece of research shapes a system. Newest entries go at the end of each section,
dated. The topic docs (`ARPG-CHARACTER.md`, `ARPG-SYNERGY.md`, `ARPG-ROADMAP.md` and the rest) hold
the full designs; this file holds the *why* and the *settled*, so nothing has to be asked twice.

## Jeff's decisions

Settled; don't re-ask.

**Scope**
- 1.0 is for Jeff and friends, played solo. There's no co-op tuning and no public server.
- **Five classes, final: Paladin, Mage, Druid, Warlock, Shaman.** The first three are built. The
  warlock is next, then the shaman (`ARPG-ROADMAP.md`, M2). Their branch ideas are in
  `ARPG-SKILL-TREES.md`:
  - **Warlock:** Plaguebringer, Demonologist and Destroyer. Its keystone flavour is "every kill
    raises an imp".
  - **Shaman:** Stormcaller, Earthwarden and Enhancer. Its keystone flavour is "Lightning Bolt
    always chains".
  - Their class-screen models are set in `ARPG-CHARACTER.md`: the warlock is an orc male in
    Nemesis, the shaman a tauren male in The Ten Storms.
- **The warlock must allow a full minion build (2026-10-17).** The demons do most of the damage
  while the warlock commands, curses and feeds them (`ARPG-CHARACTER.md`, "The warlock, in
  full").
- Hardcore and seasons: probably never.
- **Levelling is in acts:** a route through the zones, each act ending in a boss. Quests are
  optional; bounties and kills carry the levelling, at about 1.3–1.5× vanilla's pace.

**Power**
- **Big stacking is welcome.** Builds may multiply into enormous damage. Don't cap power for its
  own sake; fix only what breaks the game (crashes, hangs, endless loops).
- **Power should come from synergy, not flat stats.** Flat added damage was cut from 8× vanilla's
  size to 3× for this reason.
- **No charges** (PoE-style power, frenzy or endurance charges): Jeff declined them.
- **Respecs are free,** for both the passive web and the skill trees.

**Combat**
- **No tab-targeting and no selection:**
  - the character faces the cursor;
  - swings hit whatever they connect with;
  - spells are skillshots;
  - there is no hit stat (ARPG players never miss).
- **Never stuck unable to cast:** fillers are free, and overdraw is allowed.
- **No cast times (2026-10-16).** Spells cast at once, on the move; their old cast time becomes a
  recovery (a GCD), which cooldown reduction cuts (`ARPG-CHARACTER.md`, "Casting").
- **Bosses are soloable,** with damage caps and no auto-attack one-shots.

**Loot**
- Loot drops on the ground: no group loot rules and no binding.
- **Rarity rolls:** white 0 affixes, green 1–2, blue 3–4, purple 5–6.
- **Uniques:** every vanilla named blue and purple is a unique. Each has fixed affixes and its own
  power. Rolled drops are never uniques.
- **Drop target:** about one purple and one unique every other dungeon run.
- **No class-only vanilla items (2026-10-18).** Priest robes, tier pieces and the like are open
  to every class; the server lifts each vanilla item's class restriction at load. Only the ARPG
  sets keep one class each.

**World**
- **No quest glows or markers,** anywhere.
- **Mob packs:** 3–5 mobs; a dense vanilla camp counts as a pack already.
- **Packs spawn ahead of the player,** just out of sight, and never in towns.
- **Act quest givers offer only the act's quests;** their other vanilla quests are removed.
- **Level-up fanfare:** a banner that lists the level's gains and new spells.
- Scenes play at the act's key moments (`ARPG-ACT-1.md`).

**Voice (2026-10-19):** Jeff records the scenes' lines himself (McBride and Hogger first), and the
written line follows the take: McBride's third is now "YOU! $N! Clear them out, and bring me
Garrick Padfoot!".

**Deferred:** a weekly raid objective, and the camera's per-room cutaway.

**Working agreements**
- Make the reasonable call, say what was chosen, and keep going. Ask only for decisions that are
  Jeff's and costly to get wrong.
- Design whole systems, not stubs.
- Jeff builds the server with **MSVC**, so the server code never uses `near`, `far` or `small`
  as identifiers and never calls `min`/`max` unqualified (server `CLAUDE.md`).
- After each push, tell Jeff what to run:
  - `update-arpg-server.bat` (add `full` when files were added);
  - pull and rebuild the client.
- His play statistics come to the server repo's `arpg-stats` branch. Read them before tuning.

## Research

### Synergy: Path of Exile 2 and Last Epoch (2026-10)

What makes their builds feel built rather than bought:
- **Shared enemy states.** Skills meet through what an enemy is in (Chilled, Ignited, Heavy
  Stunned, Primed), never by naming each other. One skill sets up and another pays off.
  - PoE2's **Heavy Stun** builds from physical hits as a bar.
  - PoE2's **ground surfaces** (chilled, ignited ground) are states you place.
- **Consume to pay off.** A payoff ends the state for a burst, which gives a rhythm: set up, then
  cash in.
- **Conditional modifiers** ("more damage against Chilled enemies") let an item open a pairing
  instead of adding a number.
- **Conversions** move a skill into another element's pairings (Last Epoch's skill-tree nodes
  that change a skill's damage type).
- **Triggers as data:** "when X, do Y" rows owned by a class, a node or an item.

Built as the synergy layer: `ARPG-SYNERGY.md`. Daze is our Heavy Stun. Consecration, Flamestrike,
Blizzard and Hurricane are our grounds.

### Minions (2026-10-17)

How ARPGs make a minion build a build and not a pet (PoE/PoE2, Last Epoch's Necromancer and
Beastmaster, Diablo IV's Necromancer):
- **A scaling channel of their own.** "Minion damage", "minion life", "minion attack speed" and
  horde size on gear and passives. Gear must change what the minions do, or a minion player finds
  no upgrades. We also let the owner's tag bonuses reach a minion's hits by their tags (Minion
  plus the damage type), so the elements still matter.
- **Persistent versus temporary.** A few lasting minions (PoE's spirit-reserved ones, Last
  Epoch's skeletons and golems) beside timed swarms from skills and kills (raise on kill,
  summoned bursts). The ARPG warlock has one demon plus a horde capped at 10.
- **The player stays active.** Command skills (PoE2), Dread Shade and sacrifice (Last Epoch),
  Corpse Explosion and Book of the Dead's sacrifice option (Diablo IV). Watching minions fight
  is the failure mode; giving orders, cursing and detonating is the play.
- **An opt-out.** Diablo IV's Book of the Dead lets a necromancer sacrifice minions for personal
  power. Ours is Demonic Sacrifice: no demon, and its power goes to you.
- **AI that never embarrasses.** Minions attack what the player is fighting, stay leashed close,
  and teleport back when lost. Bad pathing kills a minion build's feel faster than bad numbers.
- **Minions carry the owner's states.** A minion's ailments are its owner's, so the synergy
  layer works through them.

### Melee balance (2026-10-16)

How ARPGs keep melee worth playing (PoE/PoE2, Last Epoch, Diablo IV). Melee pays in risk, because
it stands in the pack, so it is given:
- **Area on its primary attacks.** Cleaves, slams and sweeps hit the pack at or near full damage,
  whereas casters' area comes from projectiles and explosions.
- **A defence layer of its own:**
  - PoE's Fortify (less damage taken from hits after hitting);
  - Diablo IV's Fortify and the barbarian's innate damage reduction;
  - Last Epoch's Sentinel and Forge Guard damage reduction and block.
- **Sustain from hitting:** life on hit or leech, so the fight itself heals.
- **Pace:**
  - Attacks run at roughly 1–2 a second, while vanilla's two-handers swing every 3.5 sec.
  - On-hit effects, procs and mana-on-hit only feel alive at ARPG pace.
  - Damage per second is held fixed; the hits just come smaller and more often.

Applied in the melee pass (`ARPG-CHARACTER.md`, "Melee"): twice the swings at half each, a
40% innate cleave, Fortify 15%, and life on hit of 1% per enemy struck.

### Casting without cast times (2026-10-16)

Jeff's call: no cast times; the GCD carries the pacing, and cooldown reduction shortens it.
- **A flat GCD would break balance.** Fireball's damage was tuned to its cast time, so one short
  GCD for every spell would multiply every slow spell's damage per second.
- **So each spell's recovery is its old cast time.** Throughput is unchanged; you just move.
- **Cooldown reduction and cast speed multiply** rather than add, so nothing reaches zero.
- **The client must predict the server's GCD exactly,** or spamming a spell gets "not ready"
  errors. The server therefore reckons the ARPG GCD the client's way: per family bit, no cast
  speed (`Arpg::ClientSpellMod`).

## Tuning from play

What the statistics showed, and what changed.

- **2026-10-15 (first act play).**
  - Loot was too generous for the drop target, so rates were cut.
  - Packs were too sparse on the road, so packs now spawn ahead of the player.
  - The paladin felt bad, and flat damage was a crude fix, which led to the synergy layer.
- **2026-10-16 (Razorfen Kraul).**
  - **Paladin:** about 640 dps, one swing every ~3 sec, 70-damage cleaves against 650 on the main
    target, and dead in under 7 sec to three Quilguards. This led to the melee pass.
  - **Mana:** the mage never fell below 61%, with no lowManaSec. Changes:
    - kill mana 2% → 1%;
    - combat regen 1.5% → 1% a second;
    - Fireball's cost 9% → 11%;
    - swing mana halved with the pace.
  - **Statistics:** the session line now carries the build (weapon, swing time, defences, each
    skill's nodes), to tell build problems from balance problems.
