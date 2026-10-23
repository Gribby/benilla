# Random areas, Diablo II's way

An exploration (2026-10-22): what Diablo II does, what our two engines allow, and a plan in
phases. Phase 1 is built as the **Elwynn Rift** ("Built: the Elwynn Rift", below). Read with
`ARPG-PROGRESSION.md` (dungeons, tiers, nightmares).

## What Diablo II does

D2 has three kinds of level, and its randomness comes from the second and third:

- **Preset:** a fixed map (towns, boss arenas, the Arcane Sanctuary's frame).
- **Maze (indoors):** cellars, caves, tombs. A grid of rooms; each room is a hand-made piece
  (a DS1 preset) with doors on some of its four sides. A run picks a path of rooms from the
  entrance to the exit, adds side branches and dead ends, and picks for each grid cell a piece
  whose doors match its neighbours'. The level's size and room set come from its type.
- **Wilderness (outdoors):** the Blood Moor, the Rocky Waste. A fixed-size field filled with
  random ground tiles, cliff or wall tiles on the border, a few presets dropped in (a camp, a
  cave mouth, a shrine, a waypoint), and the exits to the next areas on given edges.

On top of the layout, every run rolls its monsters (types from the level's list, packs,
champions and uniques), its chests and shrines. The seed is per game, so a layout holds while
the game lasts.

What makes it feel good is not the geometry alone: it is **finding the exit**, with the map
revealing as you go, side branches that are worth the detour, and a known shape (a level type
has a recognisable style) with an unknown layout.

## What our engines allow

Both sides are ours, which is what makes this possible at all.

**The server (cmangos)** knows a map through three kinds of extracted file:
- `.map` grids: heights, area ids and liquid, one per 533-yard ADT tile (`Maps/GridMap.h`:
  129×129 outer and 128×128 inner heights per grid, held in plain arrays);
- `vmaps`: the doodads' and WMOs' collision, for line of sight and heights indoors;
- `mmaps`: Detour navmeshes, for creature paths.

A map with no vmaps sees everything and stands on the terrain alone. With no mmaps, creatures walk
in straight lines, as vanilla did. Heights are needed: without them creatures have no ground.
`GridMap` loads from a file, but its arrays could as well be filled in memory, from pieces of
other grids. Recast is in `dep/recastnavigation`, so a navmesh could be built at run time later.

**The client (benilla)** streams `AdtTile`s from the MPQs (`terrain_stream.rs`) and meshes each
ADT's 256 `MCNK` chunks itself (`benilla-formats` `terrain.rs`, `adt_to_tile_mesh`). It also has
its own collision with the terrain and the doodads. A generated area would be a new source for
those tiles: built from a layout the server sends, not read from a file.

**The hard part is the seams.** Two terrain pieces cut from different places don't share heights
or texture edges. So:
- pieces are cut from flat-ish ground of one zone's style (a level type is one source zone:
  the Barrens, Duskwood, Tanaris, Felwood...);
- each piece is levelled to a common base height;
- heights are blended across a band at each seam;
- the borders are cliffs or dense trees, as D2's are, which hides the edges.

The server is the truth for heights. It sends the final, blended heights with the layout, so the
two sides can't disagree; the client only has to place textures and doodads.

**What we can't reuse cheaply:** WoW's dungeons are single large WMOs, not room kits, so D2's
indoor mazes have no ready pieces. Outdoor areas come first.

## The plan

### Phase 1: one outdoor area type

- **A tile library, built offline:** a tool cuts pieces of 2×2 chunks (about 67 yards square)
  from chosen source zones. Each piece keeps its heights, its texture layers and alpha, and the
  doodads standing on it. Each is tagged with its edges (open ground, road, cliff) and its flatness.
  The library is a data file both repos read.
- **A generator, on the server:** from a seed, a grid of cells (about 8×8, roughly 530 yards
  square):
  - a path from the entrance edge to the exit edge, two or three side branches with dead ends;
  - cliff or tree-wall pieces around the walkable cells;
  - presets dropped in: a shrine, a chest, a Warden's clearing at a branch's end, the exit.
- **A map id for random areas:** a spare id from Map.dbc. The server fills its terrain in memory
  per instance, and sends the client `SMSG_ARPG_LAYOUT`: the seed, the pieces with their
  rotations, and the blended heights.
- **The client** builds its tiles for that map from the layout: meshes from the sent heights,
  texture layers and doodads from the library.
- **Spawns** reuse what exists: packs, champions and rares (`ArpgPacks`), shrines (`ArpgShrines`),
  a Warden and Cache at the end (`ArpgDungeons`), tiers and nightmares.
- **Entry:** a portal in a zone, or a dungeon tier's "Rift" mode. Each instance is its own seed.
- **No vmaps or mmaps at first:** creatures walk straight, which open ground suits.

### Phase 2: depth

- Several area types (one per source zone's style).
- Line of sight through the doodads, from a vmap-like set built from the pieces.
- Navmeshes built at run time with Recast, for creatures that path around trees.
- A fog-of-war map that reveals as you go (the minimap of the generated area).

### Phase 3: indoor mazes

A room kit: either WMO groups from vanilla's caves and crypts cut into rooms with doorways, or
new simple rooms. Both are much more work than the outdoors, and only worth it once the outdoor
areas play well.

## Size of it

| Piece | Where | Rough size |
|---|---|---|
| Tile-cutting tool and library format | client repo (Rust, reads the MPQs) | large |
| Generator and in-memory terrain | server (`Arpg/ArpgRift.cpp`, `GridMap`) | large |
| `SMSG_ARPG_LAYOUT` and the client's tile source | both | medium |
| Seam blending and borders | server | medium |
| Spawns, entry, rewards | server (reuses existing systems) | small |

Phase 1 is a few focused working sessions. The risk is visual: seams and pieces that look
pasted. The tile library's choices (flat ground, one zone's style per area type) are what keep it
from looking like a quilt.

## Built: the Elwynn Rift

Jeff's call: modular pieces, randomly laid out, Elwynn-themed first. Where they sit was left open;
the choice made was a separate Rift entered from a stone (Diablo III's Rifts), so the act's roads
stay as they are.

**Entry.** A **Rift Stone** (a portal's look, named "Elwynn Rift") stands in Goldshire and in
Razor Hill, placed when an ARPG character comes near. Stepping onto it makes a new rift and
takes the character in; party members who step on it while one of theirs is inside join that
rift (the world thread binds them to its instance before the teleport). A **Rift Exit** stands at the far end, guarded by a rare, and leads back to the stone.
Dying releases to Goldshire (the instance's ghost entrance), and the ARPG respawn uses the
rift's start.

**The map.** Map 520, in no DBC: the server inserts its `Map.dbc` row and its
`instance_template` row at start (a party instance, levels 1–60, five players); the client adds
it to its map catalog. Each layout fills a block of 2×2 grids of that map; instances' blocks
differ (256 of them, a grid apart), so every instance shares the map's terrain without meeting.

**The pieces (server, `Arpg/ArpgRift.cpp`).** Built once, on the first entry, from the server's
own map 0 `.map` grids over Elwynn: every square of 2×2 chunks (about 67 yards) whose chunks are
in Elwynn but not Northshire, Goldshire or Stormwind, with no water and no holes. A piece keeps its
relief (heights less its mean) and is flat when its rise is 10 yards or less.

**A layout:**
- the open region: a 32×32-chunk field (about 1070 yards a side), a 2-chunk closed border, a
  cellular automaton (52% open, five passes of the 5-of-9 rule), its largest region kept (200
  chunks at least, else rolled again): clearings joined by paths, with loops;
- the start on its west side, the exit the farthest walk from it;
- a piece for each 2×2 chunks: a flat one where any chunk is open, any one elsewhere;
- heights: base 60, plus each piece's relief (shared edges averaged, three blend passes within 3
  nodes of a seam), plus hills of up to 42 yards (uneven along their length) rising steeply where
  the region is closed. The walls are the closed pieces' trees on steep ground, as Diablo II's
  are cliffs and tree lines;
- 20 packs of Elwynn's hostile natives (`NativesWithin`) on open chunks at least 6 chunks' walk
  from the start, at the character's level, formed as the road's packs are (champions and rares
  by chance, a shrine beside some); a rare a level up at the exit.

**The client (`benilla-assets` `rift.rs`).** `SMSG_ARPG_RIFT` brings the layout: a header (the
block, the pieces' sources) and the heights in bands of 32 rows. Once whole, the layout is
published and map 520's directory becomes `ArpgRift<id>`, so the terrain stream drops the old
rift's tiles and requests the new one's by itself. The MPQ reader answers those tiles with a
placeholder, and the ADT loader builds each one: every chunk is the real Elwynn chunk (texture
layers, alpha, baked shadow, ground clutter) moved into place, its vertices on the sent heights
(inner ones the mean of their corners, as the server's are), its normals from the whole height
field so seams light alike; the doodads whose centre stands on it move with it, raised or lowered
by the ground's change, under rift-only ids. No buildings, no water.

**Limits for now:** no vmaps or mmaps for the rift, so creatures walk straight and see through
trees; a ghost released in a rift goes to Goldshire's graveyard side (the instance's one ghost
entrance, the Horde's too), with no way back to the corpse but the spirit healer; the stones' spots are first guesses (`STONES` in `ArpgRift.cpp`); the rift's light is
whatever the client's lighting gives a map with no `Light.dbc` rows. Phase 2 (more area types,
navmeshes, a revealing map) stands as planned.

## Built: Goldshire's gates (Act I, Alliance)

Jeff's call (2026-10-23): Goldshire as the hub with a wall round it, the areas replacing the open
Elwynn, three of them in a chain. How it is built (`ARPG-ACT-1.md` has the gates and areas):
- **An area is a layout of a kind:** the free rift's kind takes pieces from all of Elwynn and
  Elwynn's hostiles at the player's level; an area's kind takes its pieces from its part of the
  forest (a box round Northshire's fields, the eastern farms, the south-western forest edge; all
  of Elwynn if the box has too few), and its creatures by name.
- **One instance holds the whole chain.** Teleporting within a map keeps the instance, so an
  area's exit makes the next layout in another block of the same instance and moves the player
  there; the client gets the new layout and restreams. Party members in the same instance can be
  in different areas.
- **The palisade** is the server's: an Alliance character before Hogger stepping past it is put
  back inside. Its posts are game objects of a vanilla fence's or palisade's look, if the world
  database has one (else it is unseen but still holds).
- **Gates** are touched like the stones; a closed one says so.

## Built: side areas and perks

Jeff's call (2026-10-23): a couple of side areas off each area, with a lasting reason to clear
them. Each main area's layout marks two branch tips: the way from start to exit is traced back
along the walking distances, and the open chunks farthest from it (eight chunks apart) take the
side areas' entrances. A side area's layout keeps the middle 16×16 chunks only (70 open at
least), 12 packs, its boss at the far end and no exit; it is made in the same instance on first
entry and kept for the trip. Perks are `Arpg/ArpgPerks.{h,cpp}`.

## Built: roofless caves

Echo Ridge Diggings, Fargodeep Pit and Jasperlode Hollow are caves, as Diablo II's: no roof, walls
the camera looks down between, and so none of WoW's interiors' clipping.
- **Their own areas, on the rift map.** A cave is a layout of its own, in its own block, like any
  area; it shares map 520 rather than a map of its own, since a second map would only add
  instance binds to cross between them (the side area's way back is a step within the instance).
- **Tunnels** (`CaveRegion`): a walker carves one-chunk passages through the middle 16×16 chunks
  from the west side, turning a third of the time and opening a 3×3 chamber now and then, till
  70 chunks are open: winding tunnels with loops where it crossed itself, chambers and dead ends.
- **Walls** rise 30 yards over about two nodes (8 yards) above a floor with a fifth of the
  pieces' relief: sheer, so the camera never sees over them.
- **The client paints them** (`benilla-assets` `rift.rs`, theme byte in `SMSG_ARPG_RIFT`): every
  chunk is two layers, a dirt floor under rock by slope (the texture names come from the source
  tiles, so they are the install's own), no grass. Props stand on the floor beside walls, by the
  layout's seed: the source tiles' mine carts, crates, barrels, rocks and bones, and one in six a
  lantern or torch, whose model carries its light. No trees.
- **Light:** the forest's, its ambient at 45% and its sun and fog at 35%.

## Built: the automap and the town portal in random areas

- **The automap** (Diablo II's): a random area has no minimap art, so the client draws its four
  tiles from the layout (`benilla-app` `minimap/rift.rs`): floor (green outdoors, brown in a
  cave), wall where the ground is steep, its edge against the floor outlined pale, and what is
  more than 45 yards from anywhere the player has stood left dark. The server sends the area's
  marks with the layout (`SMSG_ARPG_RIFT`, after the theme): the exit (gold), the way back
  (blue), side areas' entrances (purple) and a side area's boss (red), each drawn once its spot
  has been seen. Sight is kept per layout, so a town portal there and back finds the map as left.
- **The town portal** in a rift works as in a dungeon: it opens home, and going back returns to
  the same spot of the same instance. Both ends now stand while it is open (the town end a few
  yards ahead of the arrival), walking into the town end goes back, and going back closes both.
  A portal end a player made grows in from nothing over 0.8 sec with a slight overshoot as it
  first shows (`player/arpg/portal.rs`).

