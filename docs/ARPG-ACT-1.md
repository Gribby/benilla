# Act I: Shadowfang (levels 1-25)

The first act of the 1-60 journey (`ARPG-ROADMAP.md`, M3, pulled forward as the pilot of the act
format). Chapter 1 (Elwynn Forest, Durotar) is built; the rest is the plan.

## What an act is

An act is a route through vanilla's zones, not new content: the zones, NPCs and creatures are
vanilla's, with the ARPG layer already on them (packs, champions, events, bounties, Wardens and
Caches). The act adds what an ARPG campaign has and vanilla lacks:

- **A direction.** A chain of story quests ("Act I: ...") given and taken by vanilla's own NPCs.
  The quest log is the journal: its text names the place and why, as the story would tell it. No
  markers, no arrows, no glows (Jeff's rule).
- **Packs ahead.** Out in the open world, packs of the creatures that live where the character is
  heading appear ahead of it, just out of sight, so the way is always a fight and never empty
  space.
- **Chapters**, each a band of zones around a hub town, ending in a named fight that closes it.
- **A finale**: the act's boss, whose fall ends the act and opens the next.
- **Pacing**: an act is tuned to a time. Act I's target is about 8 hours from level 1 to its
  finale, read from the play statistics (`levels`, `zoneSec`).

The act's givers (McBride, Dughan, Gornek, Gar'Thok and the rest) give only the act: their vanilla
quests are taken off them (vanilla quests still hand in to them), except a step of a chain that
starts with a giver outside the act, which keeps its place so the chain still ends somewhere. Vanilla's other quests stay, as
side quests. Those an act quest tells again (Hogger's wanted poster,
Goldtooth...) are taken off their givers so the story is not told twice, unless another quest
follows on from one (the kobold camps lead to Echo Ridge): those stay, and the start-up log names
them.

## Act I's story

The old kingdoms crack under new threats: the Defias Brotherhood in the human lands, the Burning
Blade and the quilboar in Kalimdor, the Scourge's stirrings in Lordaeron. Under all of it, in
Silverpine, Archmage Arugal's worgen curse spreads from Shadowfang Keep. Both factions' roads
lead there: the act ends with Arugal.

## Built: chapter 1

cmangos `Arpg/ArpgActs.{h,cpp}`, on with `Arpg.Acts = 1` (the default).

### One start per faction

Every Alliance character starts in Goldshire and every Horde one in the Valley of Trials, whatever
its race and class (`ObjectMgr::ArpgOneStartPerFaction`), so each faction has one road.

### Goldshire, the hub (Alliance)

The Alliance's chapter 1 is played as Diablo II's first act (cmangos `Arpg/ArpgRift.{h,cpp}`,
`ARPG-RANDOM-AREAS.md`). Goldshire is the town: a palisade (105 yards round the square) holds an
Alliance character in until the Hogger quest is done; walking out brings it back inside, and a
character found far off is brought into town. It holds no one past level 15. Four gates stand inside the palisade:

| Gate | Where | Leads to | Open |
|---|---|---|---|
| North | the Northshire Fields, levels 1–5 | Kobold Vermin and Workers, Defias Thugs, wolves; Garrick Padfoot at the end | always |
| East | the Farmlands, 5–8 | the Fargodeep and Jasperlode kobolds, the Defias bandits; Goldtooth halfway, Morgan the Collector at the end | at 5, or with the Northshire Watch done |
| South | Forest's Edge, 8–11 | Riverpaw gnolls, prowlers, bears; Hogger at the end | at 8, or with Fire on the Farms done |
| West | the road to Westfall (Sentinel Hill) | | with Hogger done |

Each area is a random layout of Elwynn's ground from its own part of the forest, made at the gate.
Its exit leads on into the next area (the Fields into the Farmlands, the Farmlands into Forest's
Edge) and Forest's Edge's back to Goldshire; each area's start has a way back to town. The areas'
creatures are the act quests' own, at the area's levels (the character's level, held to the
area's band), and its named leaders are crowned as the act crowns them; Hogger drops the finale's
guaranteed item. Marshal Dughan gives all the Alliance's quests: "The Road to Goldshire" is gone.

**Side areas.** Each area has two side areas, entered at its dead ends (as far off the way from
start to exit as the layout goes), each smaller, its packs closer, its boss at the far end:

| Off | Side area | Levels | Boss | Perk, the first time it falls |
|---|---|---|---|---|
| the Northshire Fields | Echo Ridge Diggings | 2–5 | a kobold rare | a skill point |
| the Northshire Fields | Wolf Run | 3–5 | Gruff Swiftbite | a passive point |
| the Farmlands | Fargodeep Pit | 5–8 | Goldtooth | a skill point |
| the Farmlands | The Defias Hideout | 6–8 | Morgan the Collector | 10% more gold found |
| Forest's Edge | Jasperlode Hollow | 8–10 | Mother Fang | 15 to every resistance |
| Forest's Edge | Riverpaw Warren | 9–11 | a gnoll rare | a passive point |

A side area has no exit: its way back, at its start, returns to beside its entrance in the area
it branches off (the same instance, so that area is as it was left). Its boss drops a hoard (two
or three blues), and its perk goes to every party member there who lacks it (`Arpg/ArpgPerks.h`,
kept in `character_arpg_perks`, shown on the sheet). The Mines of Elwynn, Fire on the Farms and
Jasperlode send the character into Fargodeep Pit, the Defias Hideout and Jasperlode Hollow; the
Farmlands' own exit is held by a rare.

### The quests

Written into the world database at every start (entries 59000 and up, rewritten each time, so
editing the table in `ArpgActs.cpp` and restarting is how they change; `Arpg.Acts = 0` takes them
out). NPCs and creatures are
found by name; a name the database lacks drops that objective and is logged at start
(`ARPG acts: ... finds no ...`), and the act still loads.

**Alliance, Elwynn Forest**

| Quest | From, to | Asks | Leader |
|---|---|---|---|
| The Northshire Watch | Marshal Dughan | Kobold Vermin, Defias Thugs | Garrick Padfoot (champion) |
| The Mines of Elwynn | Dughan | Kobold Tunnelers and Miners (Fargodeep) | Goldtooth (champion) |
| Fire on the Farms | Dughan | Defias Bandits, Cutpurses, Rogue Wizards | Morgan the Collector (champion) |
| Jasperlode | Dughan | Kobold Geomancers, Laborers | |
| Hogger | Dughan | Riverpaw gnolls | Hogger (the chapter's finale) |
| The Fields of Westfall | Dughan to Gryan Stoutmantle | report: opens chapter 2 | |

**Horde, Durotar**

| Quest | From, to | Asks | Leader |
|---|---|---|---|
| The Trial of the Valley | Gornek | Mottled Boars, Vile Familiars | Sarkoth (champion) |
| The Road to Razor Hill | Gornek to Gar'Thok | report | |
| Tiragarde Keep | Gar'Thok | Kul Tiras Sailors, Marines | Lieutenant Benedict (champion) |
| The Burning Blade | Gar'Thok | Burning Blade cultists | Gazz'uz (champion) |
| Zalazane | Gar'Thok | Hexed and Voodoo Trolls | Zalazane (the chapter's finale) |
| The Crossroads | Gar'Thok to Thork | report: opens chapter 2 | |

Each quest is gated on the one before; its XP and money are sized for its level.

### Packs ahead

The first build drew each quest's road with pathfinding and filled points along it; in play it did
not read as a road (Jeff), so it was replaced. Now, out in the open world (on foot or mounted, not
flying, not on a boat, not swimming, not resting in a town or inn), each time the character
travels 50 yards a pack is placed 110-160 yards ahead of it, within about 40 degrees of the way it
is heading: the creature that lives nearest that spot within three levels of the character (never
an act's leader), on the usual pack rules (champions, events). A spot is skipped if a creature
stands within 15 yards, an NPC, guard, friendly creature or player within 60 yards (a town, a
camp), or the character could not walk there (water, a cliff). At most four are out at once;
packs left 260 yards behind, out of combat, go, as do all of them at logout, and an untouched pack
goes after ten minutes. Not tied to a quest: it is how the whole open world plays.

### Leaders and finales

Each quest's named leader, at its own spawn, is a champion with a set affix and its own name
(Garrick Padfoot Extra Strong, Goldtooth Stone Skin, Gazz'uz Fire Enchanted...); it never leads or
joins a pack, so there is only ever one. The chapter's finale (Hogger, Zalazane) is a rare with a
guaranteed blue two levels above it.

### Scenes

Short cutscenes over the live world (cmangos `Arpg/ArpgScenes.{h,cpp}`, client
`player/arpg/scene.rs` on `cinematic.rs`). A scene is a camera track and subtitles around an
anchor (an NPC or creature, found by name near the character), sent as `SMSG_ARPG_SCENE` (0x352)
and played by the cinematic player: letterboxed, the UI and cursor hidden, ESC skips. The camera
keys are authored in the anchor's frame (forward toward the character, left, up, in yards), so a
scene fits wherever the anchor stands. While it plays the character is rooted, creatures leave it
alone and its actions are refused; the anchor turns to it and acts its cues (talk, point, roar).
The client closes it with `CMSG_COMPLETE_CINEMATIC`, as a cinematic, at its end or on a skip; the
server ends it anyway five seconds past its length. Each plays once per character
(`character_arpg_scene`).

| Scene | When | Around |
|---|---|---|
| Act I opens (Alliance) | Taking The Northshire Watch | Marshal McBride, 23.5 s, voiced |
| Act I opens (Horde) | Taking The Trial of the Valley | Gornek, 16 s |
| Hogger | First coming within 40 yards of him on his quest | Hogger, 9.5 s, voiced |
| Zalazane | First coming within 40 yards of him on his quest | Zalazane, 9.5 s |

**Voice.** Jeff records the lines. Each clip ships inside the client (`assets/voice/`) and the
client's `sound/scene_voice.rs` keys it by scene id and line index: a line speaks its clip as its
subtitle window opens, and the next line, the scene's end or a skip cuts it. A line's window on
the server must outlast its clip, so a new recording usually means retiming the scene in
`ArpgScenes.cpp` (its camera keys and cues with it). Recorded so far: McBride's three lines and
Hogger's; a line with no clip stays a subtitle.
The clips are normalised to −18 LUFS integrated, true peak −1.5 dB (ffmpeg's two-pass
`loudnorm`), as the raw takes come in near −44 LUFS and play too quietly beside the game's own sounds.

### Dev notes

With `Arpg.DevTools = 1`, saying `#note <text>` writes a note line to the play statistics with
where the character stands, instead of saying it: for marking a road that is too long, too empty
or too crowded while walking it.

## Not built yet

| Step | What |
|---|---|
| Hubs and waypoints | Reaching a hub saves it; the town portal goes to any reached hub |
| Run speed out of combat | Faster travel between fights |
| Travel time stat | Out-of-combat time per zone, to find the empty stretches |
| Reward choices | An item choice on each story quest |
| Chapter 2 | Westfall and the Deadmines; the Barrens, Ragefire Chasm and the Wailing Caverns |
| Chapter 3 | Redridge, Duskwood; Stonetalon, Ashenvale; both ending in Silverpine |
| Finale | Arugal in Shadowfang Keep as an act boss: telegraphs, worgen waves, an enrage; the act's Cache |
| More scenes | Chapter hand-offs (into Westfall, the Barrens), Arugal; a dev tool to record camera keys while flying |
| Voice | Gornek's and Zalazane's scene lines; the quest givers' lines if Jeff wants them |

## The plan for chapters 2 and 3

### Chapter 2: The Frontier (levels 10-20)

| Alliance | Horde |
|---|---|
| Westfall (Sentinel Hill) | The Barrens (the Crossroads) |
| Dungeon: the Deadmines | Dungeons: Ragefire Chasm, Wailing Caverns |

**Closes with** the chapter's dungeon cleared (Edwin VanCleef; Mutanus the Devourer).

### Chapter 3: The Gathering Dark (levels 20-25)

| Alliance | Horde |
|---|---|
| Redridge Mountains, Duskwood (Lakeshire, Darkshire) | Stonetalon Mountains, Ashenvale (Sun Rock, Splintertree) |

Both roads end in Silverpine Forest, at Shadowfang Keep.

### Finale: Archmage Arugal

Shadowfang Keep on its act difficulty: Arugal as an act boss, with the keep's Warden in the
courtyard. His fall ends Act I with a banner and the act's Cache (gold, a guaranteed unique, a set
piece of the character's class, a Codex page), and opens Act II.
