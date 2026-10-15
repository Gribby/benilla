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

Every Alliance character starts in Northshire and every Horde one in the Valley of Trials, whatever
its race and class (`ObjectMgr::ArpgOneStartPerFaction`), so each faction has one road.

### The quests

Written into the world database at every start (entries 59000 and up, rewritten each time, so
editing the table in `ArpgActs.cpp` and restarting is how they change; `Arpg.Acts = 0` takes them
out). NPCs and creatures are
found by name; a name the database lacks drops that objective and is logged at start
(`ARPG acts: ... finds no ...`), and the act still loads.

**Alliance, Elwynn Forest**

| Quest | From, to | Asks | Leader |
|---|---|---|---|
| The Northshire Watch | Marshal McBride | Kobold Vermin, Defias Thugs | Garrick Padfoot (champion) |
| The Road to Goldshire | McBride to Marshal Dughan | report | |
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
| Act I opens (Alliance) | Taking The Northshire Watch | Marshal McBride, 16 s |
| Act I opens (Horde) | Taking The Trial of the Valley | Gornek, 16 s |
| Hogger | First coming within 40 yards of him on his quest | Hogger, 9.5 s |
| Zalazane | First coming within 40 yards of him on his quest | Zalazane, 9.5 s |

Subtitles only for now; the voice can ride the same message later.

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
| Voice | Deferred |

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
