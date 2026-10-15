# Act I: Shadowfang (levels 1-25), draft

The first act of the 1-60 journey (`ARPG-ROADMAP.md`, M3, pulled forward as the pilot of the act
format). A draft for Jeff's review: nothing here is built yet.

## What an act is

An act is a route through vanilla's zones, not new content: the zones, quests, dungeons and
creatures are vanilla's, with the ARPG layer already on them (packs, champions, events, bounties,
Wardens and Caches). The act adds what an ARPG campaign has and vanilla lacks:

- **A direction.** An act journal says where the story goes next, as the story would tell it:
  "Westfall's farmers are being driven off their land; Gryan Stoutmantle gathers a militia at
  Sentinel Hill, south-west of Goldshire." No markers, no arrows, no glows (Jeff's rule): the
  journal names the place and the world map outlines its zone, nothing more.
- **Chapters**, each a band of zones around a hub town, ending in a fight that closes it.
- **A finale**: the act's boss, harder than its dungeon's usual last boss, whose fall ends the
  act with a reward and opens the next.
- **Hubs and waypoints**: each chapter's hub town is a waypoint once reached; the town portal (T)
  can send the character to any waypoint of the act it has reached (Diablo II's waypoints).
- **Pacing**: an act is tuned to a time. Act I's target is about 8 hours from level 1 to its
  finale, read from the play statistics (`levels`, `zoneSec`).

Quests stay as vanilla's: the journal points at the zone's own story chains (the Defias in
Westfall, the Burning Blade in Durotar), it does not replace them.

## Act I's story

The old kingdoms crack under new threats: the Defias Brotherhood in the human lands, the Burning
Blade and the quilboar in Kalimdor, the Scourge's stirrings in Lordaeron. Under all of it, in
Silverpine, Archmage Arugal's worgen curse spreads from Shadowfang Keep. Both factions' roads
lead there: the act ends with Arugal.

## The route

Each faction has its own first two chapters and they meet in the third. A character's route is
its faction's; its start valley is its race's.

### Chapter 1: Awakening (levels 1-10)

| Alliance | Horde |
|---|---|
| Northshire, Coldridge Valley or Shadowglen, then Elwynn Forest, Dun Morogh or Teldrassil | Valley of Trials, Camp Narache or Deathknell, then Durotar, Mulgore or Tirisfal Glades |
| Hubs: Goldshire, Kharanos, Dolanaar | Hubs: Razor Hill, Bloodhoof Village, Brill |

**Closes with** the zone's named elite, picked per zone as it is built (Hogger in Elwynn Forest),
given an act fight: telegraphed moves, a champion's affix, a guaranteed blue.

### Chapter 2: The Frontier (levels 10-20)

| Alliance | Horde |
|---|---|
| Westfall, Loch Modan or Darkshore | The Barrens, or Silverpine Forest |
| Hubs: Sentinel Hill, Thelsamar, Auberdine | Hubs: the Crossroads, the Sepulcher |
| Dungeon: the Deadmines | Dungeons: Ragefire Chasm, Wailing Caverns |

**Closes with** the chapter's dungeon cleared (Edwin VanCleef; Mutanus the Devourer or
Taragaman the Hungerer).

### Chapter 3: The Gathering Dark (levels 20-25)

| Alliance | Horde |
|---|---|
| Redridge Mountains, Duskwood, Ashenvale or the Wetlands | Stonetalon Mountains, Ashenvale or Hillsbrad Foothills |
| Hubs: Lakeshire, Darkshire, Astranaar | Hubs: Sun Rock Retreat, Splintertree Post, Tarren Mill |

Both roads end in Silverpine Forest, at Shadowfang Keep.

### Finale: Archmage Arugal

Shadowfang Keep on its act difficulty: Arugal as an act boss (more health, his blink and shadow
bolts telegraphed, worgen adds that rise in waves, an enrage), with the keep's Warden in the
courtyard. His fall:

- ends Act I: a banner, the act's Cache (gold, a guaranteed unique, a set piece of the
  character's class, a Codex page);
- opens Act II in the journal, and Act I's zones on the Hard tier for those who want to replay;
- is remembered per character, as the difficulty tiers are.

## How it gets built

| Step | What | Where |
|---|---|---|
| 1. The act's data | Chapters, zones, hubs, beats and their journal text, the closing fights, per faction | cmangos `Arpg/ArpgActs.{h,cpp}`, a table in code as the uniques are |
| 2. Progress | A character's act, chapter and beats done, saved (`character_arpg_act`, made by the server); beats advance on reaching a level, entering a zone, a kill (a creature's entry) or a dungeon cleared | server |
| 3. The journal | `SMSG_ARPG_ACT` sends the act, the chapter, its beats (done or not) and the next beat's text; a journal window (J) shows them, and the world map outlines the next zone | client `player/arpg/acts.rs`, `arpg_hud.lua` |
| 4. Closing fights | The chapter elites and Arugal made act bosses: telegraphs, adds, enrage, their rewards | server, on the existing threats, champions and Cache code |
| 5. Hubs and waypoints | Reaching a hub saves it; the town portal's menu lists the act's reached hubs and goes to one | server and client, on the existing town portal |
| 6. Density and pacing | The route's zones get pack density, events and bounties first; off-route zones keep theirs. Pace read from the statistics and tuned to the 8-hour target | server config, then play |
| 7. Play | Jeff plays Act I with one character per faction; the statistics and his notes decide the tuning, then the act format is copied to Acts II to V | |

Steps 1-3 make the act playable as a route; 4-5 make it feel like an act; 6-7 tune it. Each step
is played before the next.

## Decisions for Jeff

1. **One finale for both factions** (Arugal), or each its own (VanCleef for the Alliance,
   Mutanus for the Horde)? The draft has one, as both reach Silverpine at about 25.
2. **Waypoints** through the town portal: yes, or keep travel as vanilla's (flight paths,
   walking)?
3. **The journal key**: J, or another?
4. **The act's reward**: is a guaranteed unique and a class set piece the right size?
