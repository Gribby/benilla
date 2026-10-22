# Random areas, Diablo II's way

An exploration (2026-10-22), not yet built: what Diablo II does, what our two engines allow, and
a plan in phases. Read with `ARPG-PROGRESSION.md` (dungeons, tiers, nightmares).

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

## For Jeff

- **Where they sit:** a separate "Rift" you enter from a portal or a dungeon menu (D3's Rifts,
  D4's dungeons), or areas on the act's road between zones (D2's way)?
- **Which zones' look first?** The Barrens' plains are the easiest (flat, open); Duskwood or
  Felwood look better with tree walls.
