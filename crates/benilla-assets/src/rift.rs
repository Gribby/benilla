//! ARPG rifts (cmangos `Arpg/ArpgRift.h`, `docs/ARPG-RANDOM-AREAS.md`): a random outdoor area made
//! of 2×2-chunk pieces of Elwynn Forest, on a map no DBC names ([`RIFT_MAP`]). The server sends
//! the layout (`SMSG_ARPG_RIFT`): which source square each piece is, and the final heights. The
//! app [`publish`]es it and names the map's directory after it ([`rift_dir`]), so a new layout is a
//! new directory and the terrain stream drops and re-requests every tile by itself. The MPQ
//! reader answers a covered tile's `.adt` with [`PLACEHOLDER`], and the ADT loader builds the tile
//! here ([`compose_tile`]): each chunk is the real source chunk (layers, alpha, shadow, clutter)
//! moved into place under the server's heights, with the doodads standing on it; no buildings
//! and no water.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use benilla_formats::{adt_to_tile_mesh, ChunkMesh, Doodad, TileMesh, WmoInstance, TILE_SIZE};

/// The rifts' map id: in no `Map.dbc`, defined by the server and here alike.
pub const RIFT_MAP: u32 = 520;
/// The rifts' map name, for the catalog.
pub const RIFT_NAME: &str = "Elwynn Rift";
/// What the MPQ reader serves for a rift tile: the loader builds the real tile.
pub const PLACEHOLDER: &[u8] = b"ARPGRIFT";
/// The map the pieces come from.
const SOURCE_DIR: &str = "Azeroth";
/// Doodads moved into a rift take ids above every authored `uniqueId`.
const RIFT_UID_BASE: u32 = 0xC000_0000;
/// A layout's outer heights a side: 2 tiles of 128 cells, plus one.
pub const RIFT_NODES: usize = 257;
/// Its pieces a side (2×2 chunks each).
pub const RIFT_PIECES: usize = 16;

const UNIT: f64 = TILE_SIZE as f64 / 128.0;
const MAP_CENTER: f64 = 32.0 * 1600.0 / 3.0;

/// One rift's layout, as the server sent it.
#[derive(Debug, Clone, PartialEq)]
pub struct RiftLayout {
    pub id: u32,
    /// The first server grid of its 2×2 block; server grid `gx` runs along WoW X, so the tile
    /// file is `{dir}_{gy}_{gx}`.
    pub gx0: u8,
    pub gy0: u8,
    /// The `AreaTable` id every chunk carries (Elwynn Forest).
    pub area: u16,
    /// `RIFT_PIECES²`, i-major (i along X): source server grid `gx`, `gy`, and the source
    /// square's first chunk `a` (along X), `b`.
    pub pieces: Vec<[u8; 4]>,
    /// `RIFT_NODES²` outer heights, row `i` (along X) by column `j`.
    pub heights: Vec<f32>,
    /// [`THEME_OUTDOOR`] (the source chunks moved into place) or [`THEME_CAVE`] (a roofless
    /// cave: rock walls and a dirt floor painted by slope, mine props, no trees or grass).
    pub theme: u8,
    /// Its marks, for the map: (kind, x, y), kind 1 the exit, 2 the way back, 3 a side area's
    /// entrance, 4 a side area's boss.
    pub marks: Vec<(u8, f32, f32)>,
    /// Its building sites, dressed by the tile holding each one's centre.
    pub sites: Vec<RiftSite>,
    /// A side area's look beyond its sites: [`FLAVOUR_WEBS`] and the rest, 0 none.
    pub flavour: u8,
    /// Each site's building as the server raised it (its collision): the WMO's vmaps name and
    /// its origin's height, by the site's index. Empty from older servers, and a name empty
    /// where the server raised none: then the client picks one itself.
    pub buildings: Vec<(String, f32)>,
}

/// Jasperlode Hollow: webs and egg sacs, from Duskwood's tiles.
pub const FLAVOUR_WEBS: u8 = 1;
/// Fargodeep Pit: candles, ore and gold, from Elwynn's mine mouths.
pub const FLAVOUR_CANDLES: u8 = 2;
/// Echo Ridge Diggings: scaffolds, beams and spoil.
pub const FLAVOUR_DIG: u8 = 3;
/// The Defias Hideout: burnt farms (Westfall's) and Defias camps.
pub const FLAVOUR_BURNT: u8 = 4;
/// Wolf Run: thick woods and dens.
pub const FLAVOUR_WOLVES: u8 = 5;
/// Riverpaw Warren: smaller, closer camps (the server's).
pub const FLAVOUR_WARREN: u8 = 6;

/// Server grids a flavour borrows models from, beyond its own pieces' tiles: Duskwood's for
/// webs, Westfall's for burnt farms, Elwynn's mine mouths for candles and dig works.
fn borrowed_sources(flavour: u8) -> &'static [(u8, u8)] {
    match flavour {
        FLAVOUR_WEBS => &[(51, 32), (52, 33), (51, 34), (49, 33)],
        FLAVOUR_CANDLES => &[(50, 31), (50, 32), (49, 33)],
        FLAVOUR_DIG => &[(48, 32), (48, 31), (50, 31)],
        FLAVOUR_BURNT => &[(51, 29), (52, 29), (52, 30), (53, 30)],
        _ => &[],
    }
}

/// A Defias camp: tents, bedrolls, crates and barrels.
pub const SITE_DEFIAS_CAMP: u8 = 6;
/// A wolf den: bones and rocks.
pub const SITE_DEN: u8 = 7;

/// A building site the server keeps clear and levels: [`SITE_FARMSTEAD`] and the rest.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RiftSite {
    pub kind: u8,
    pub x: f32,
    pub y: f32,
    /// The levelled ground's height.
    pub z: f32,
    /// Its facing, radians, WoW's.
    pub o: f32,
    pub radius: f32,
}

/// A farmhouse or barn in a fence ring, with hay, a cart, barrels and a scarecrow.
pub const SITE_FARMSTEAD: u8 = 1;
/// A fenced plot of crops in rows.
pub const SITE_PLOT: u8 = 2;
/// Tents round a fire, bones and a banner: a gnoll camp.
pub const SITE_CAMP: u8 = 3;
/// A watchtower and its rubble.
pub const SITE_TOWER: u8 = 4;
/// Felled logs, stumps and a sawhorse.
pub const SITE_LOGGING: u8 = 5;

/// A layout of Elwynn's own chunks.
pub const THEME_OUTDOOR: u8 = 0;
/// A roofless cave, as Diablo II's: sheer walls the camera looks down between.
pub const THEME_CAVE: u8 = 1;

impl RiftLayout {
    /// The outer height at node (`i`, `j`), clamped to the layout.
    pub fn height(&self, i: usize, j: usize) -> f32 {
        let i = i.min(RIFT_NODES - 1);
        let j = j.min(RIFT_NODES - 1);
        self.heights.get(i * RIFT_NODES + j).copied().unwrap_or(0.0)
    }

    /// The node (fractional) under a WoW (x, y).
    pub fn node_at(&self, x: f32, y: f32) -> (f64, f64) {
        let i = ((32.0 - f64::from(self.gx0)) * f64::from(TILE_SIZE) - f64::from(x)) / UNIT;
        let j = ((32.0 - f64::from(self.gy0)) * f64::from(TILE_SIZE) - f64::from(y)) / UNIT;
        (i, j)
    }

    /// The ground height at a (fractional) node, bilinear.
    fn ground_at_node(&self, i: f32, j: f32) -> f32 {
        let max = (RIFT_NODES - 1) as f32;
        let (i, j) = (i.clamp(0.0, max), j.clamp(0.0, max));
        let (i0, j0) = (i.floor() as usize, j.floor() as usize);
        let (ti, tj) = (i - i0 as f32, j - j0 as f32);
        let h = |a: usize, b: usize| self.height(a, b);
        let top = h(i0, j0) * (1.0 - tj) + h(i0, j0 + 1) * tj;
        let bottom = h(i0 + 1, j0) * (1.0 - tj) + h(i0 + 1, j0 + 1) * tj;
        top * (1.0 - ti) + bottom * ti
    }

    /// The ground height under a WoW (x, y), bilinear over the outer heights.
    pub fn ground(&self, x: f32, y: f32) -> f32 {
        let (i, j) = self.node_at(x, y);
        let max = (RIFT_NODES - 1) as f64;
        let (i, j) = (i.clamp(0.0, max), j.clamp(0.0, max));
        let (i0, j0) = (i.floor() as usize, j.floor() as usize);
        let (ti, tj) = ((i - i0 as f64) as f32, (j - j0 as f64) as f32);
        let h = |a: usize, b: usize| self.height(a, b);
        let top = h(i0, j0) * (1.0 - tj) + h(i0, j0 + 1) * tj;
        let bottom = h(i0 + 1, j0) * (1.0 - tj) + h(i0 + 1, j0 + 1) * tj;
        top * (1.0 - ti) + bottom * ti
    }

    /// Whether the tile (`tile_x`, `tile_y`) is one of the layout's four.
    pub fn covers(&self, tile_x: u32, tile_y: u32) -> bool {
        let (gx, gy) = (tile_y, tile_x);
        (u32::from(self.gx0)..u32::from(self.gx0) + 2).contains(&gx)
            && (u32::from(self.gy0)..u32::from(self.gy0) + 2).contains(&gy)
    }

    /// The outward normal at node (`i`, `j`) in WoW axes, from the heights' central differences:
    /// continuous across chunks and tiles.
    fn normal(&self, i: usize, j: usize) -> [f32; 3] {
        let (i, j) = (i.min(RIFT_NODES - 1), j.min(RIFT_NODES - 1));
        let ip = (i + 1).min(RIFT_NODES - 1);
        let im = i.saturating_sub(1);
        let jp = (j + 1).min(RIFT_NODES - 1);
        let jm = j.saturating_sub(1);
        let unit = UNIT as f32;
        // X falls as i grows, Y as j grows.
        let dh_dx = -(self.height(ip, j) - self.height(im, j)) / ((ip - im).max(1) as f32 * unit);
        let dh_dy = -(self.height(i, jp) - self.height(i, jm)) / ((jp - jm).max(1) as f32 * unit);
        let n = [-dh_dx, -dh_dy, 1.0];
        let len = (n[0] * n[0] + n[1] * n[1] + n[2] * n[2]).sqrt();
        [n[0] / len, n[1] / len, n[2] / len]
    }
}

/// The layouts known, newest last; a few are kept so a tile loading as the next arrives still
/// finds its own.
static LAYOUTS: Mutex<Vec<Arc<RiftLayout>>> = Mutex::new(Vec::new());
const KEPT: usize = 4;

/// Make a layout known to the tile loader.
pub fn publish(layout: RiftLayout) {
    let mut all = LAYOUTS
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    all.retain(|l| l.id != layout.id);
    all.push(Arc::new(layout));
    let excess = all.len().saturating_sub(KEPT);
    all.drain(..excess);
}

/// The layout `id`, if known.
pub fn layout(id: u32) -> Option<Arc<RiftLayout>> {
    LAYOUTS
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .iter()
        .find(|l| l.id == id)
        .cloned()
}

/// The layout last made known: the one the player is in.
pub fn latest() -> Option<Arc<RiftLayout>> {
    LAYOUTS
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .last()
        .cloned()
}

/// The theme of the layout last made known: the one the player is in.
pub fn current_theme() -> u8 {
    LAYOUTS
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .last()
        .map_or(THEME_OUTDOOR, |l| l.theme)
}

/// The map directory a layout's tiles stream from.
pub fn rift_dir(id: u32) -> String {
    format!("ArpgRift{id}")
}

/// A rift tile's internal path (`World/Maps/ArpgRift7/ArpgRift7_30_40.adt`, either slash, any
/// case) as (layout id, tile x, tile y).
pub fn parse_rift_path(path: &str) -> Option<(u32, u32, u32)> {
    let lower = path.replace('\\', "/").to_ascii_lowercase();
    let rest = lower.strip_prefix("world/maps/arpgrift")?;
    let (id, file) = rest.split_once('/')?;
    let id: u32 = id.parse().ok()?;
    let stem = file.strip_suffix(".adt")?;
    let stem = stem.strip_prefix(&format!("arpgrift{id}_"))?;
    let (tx, ty) = stem.split_once('_')?;
    Some((id, tx.parse().ok()?, ty.parse().ok()?))
}

/// A rift map's WDT (`World/Maps/ArpgRift7/ArpgRift7.wdt`), made for the MPQ reader: `MAIN` flags
/// the layout's four tiles alone. Without it the stream probes every tile in its window, and one
/// past the layout's block (an area's start is at its west edge) never loads, so the loading
/// screen waits forever. `None` for any other path, or a layout not known.
pub fn rift_wdt(path: &str) -> Option<Vec<u8>> {
    let lower = path.replace('\\', "/").to_ascii_lowercase();
    let rest = lower.strip_prefix("world/maps/arpgrift")?;
    let (id, file) = rest.split_once('/')?;
    let id: u32 = id.parse().ok()?;
    if file != format!("arpgrift{id}.wdt") {
        return None;
    }
    let layout = layout(id)?;
    let mut out = Vec::with_capacity(64 * 64 * 8 + 64);
    // Chunk magics are reversed on disk.
    out.extend_from_slice(b"REVM");
    out.extend_from_slice(&4u32.to_le_bytes());
    out.extend_from_slice(&18u32.to_le_bytes());
    out.extend_from_slice(b"DHPM");
    out.extend_from_slice(&32u32.to_le_bytes());
    out.extend_from_slice(&[0u8; 32]);
    out.extend_from_slice(b"NIAM");
    out.extend_from_slice(&(64u32 * 64 * 8).to_le_bytes());
    let mut grid = vec![0u8; 64 * 64 * 8];
    for tile_y in 0..64u32 {
        for tile_x in 0..64u32 {
            if layout.covers(tile_x, tile_y) {
                grid[((tile_y * 64 + tile_x) * 8) as usize] = 1;
            }
        }
    }
    out.extend_from_slice(&grid);
    Some(out)
}

/// Whether the MPQ reader should answer `path` with [`PLACEHOLDER`].
pub fn serves(path: &str) -> bool {
    parse_rift_path(path)
        .and_then(|(id, tx, ty)| layout(id).map(|l| l.covers(tx, ty)))
        .unwrap_or(false)
}

/// The lattice coordinate `n` units in from the map's centre line, as the ADT mesher snaps to.
fn lattice(n: f64) -> f32 {
    (MAP_CENTER - n * UNIT) as f32
}

/// One rift chunk to build: where it sits in the layout, and its source.
struct Wanted {
    /// The chunk's place in the layout (`la` along X), 0..32.
    la: usize,
    lb: usize,
    /// The source chunk in its tile (`a` along X).
    src_a: usize,
    src_b: usize,
}

/// Builds one rift tile from its sources, one source tile at a time: ask [`Self::next_source`]
/// for an internal path, [`Self::feed`] its bytes, and [`Self::finish`] once none is left.
pub struct RiftComposer {
    layout: Arc<RiftLayout>,
    /// The chunks each source tile gives, so each is read and parsed once.
    sources: Vec<((u8, u8), Vec<Wanted>)>,
    next: usize,
    chunks: Vec<ChunkMesh>,
    doodads: Vec<Doodad>,
    wmos: Vec<WmoInstance>,
    /// A cave's textures and props, found in its sources.
    kit: CaveKit,
    /// An outdoor area's buildings and props for its sites, found in its sources.
    dress: DressKit,
    tile: (usize, usize),
}

/// What a cave is painted and furnished with: names found in the source tiles, so they are the
/// install's own.
#[derive(Default)]
struct CaveKit {
    floor: Option<String>,
    wall: Option<String>,
    props: Vec<String>,
    lights: Vec<String>,
    /// The flavour's own: webs, candles and ore, or scaffolds and spoil.
    extra: Vec<String>,
    /// Every texture seen, for a floor or a wall the names didn't give.
    seen: Vec<String>,
}

/// Sources a cave reads for its kit; it takes no chunk of theirs.
const CAVE_SOURCES: usize = 4;

impl CaveKit {
    /// A floor and a wall from what was seen when the names gave neither.
    fn settle(&mut self) {
        if self.floor.is_none() {
            self.floor = self
                .seen
                .iter()
                .find(|t| Some(*t) != self.wall.as_ref())
                .cloned();
        }
        if self.wall.is_none() {
            self.wall = self
                .seen
                .iter()
                .find(|t| Some(*t) != self.floor.as_ref())
                .cloned();
        }
    }

    fn take(&mut self, source: &TileMesh, flavour: u8, borrowed: bool) {
        let extra: &[&str] = match flavour {
            FLAVOUR_WEBS => &["web", "cocoon", "eggsac", "egg", "spider"],
            FLAVOUR_CANDLES => &[
                "candle", "orevein", "copper", "tinvein", "coin", "chest", "treasure",
            ],
            FLAVOUR_DIG => &[
                "scaffold", "beam", "support", "pickaxe", "shovel", "barrow", "rubble", "dirt",
            ],
            _ => &[],
        };
        for d in &source.doodads {
            let lower = d.model.to_ascii_lowercase();
            if named(&lower, extra) && self.extra.len() < 12 && !self.extra.contains(&d.model) {
                self.extra.push(d.model.clone());
            }
        }
        if borrowed {
            return;
        }
        for chunk in &source.chunks {
            for name in &chunk.layer_textures {
                if self.seen.len() < 16 && !self.seen.contains(name) {
                    self.seen.push(name.clone());
                }
                let lower = name.to_ascii_lowercase();
                if self.wall.is_none() && (lower.contains("rock") || lower.contains("cliff")) {
                    self.wall = Some(name.clone());
                }
                if self.floor.is_none()
                    && (lower.contains("dirt") || lower.contains("gravel") || lower.contains("mud"))
                {
                    self.floor = Some(name.clone());
                }
            }
        }
        const PROPS: [&str; 15] = [
            "mine", "cart", "crate", "barrel", "rock", "boulder", "bone", "skull", "beam",
            "support", "rubble", "pick", "shovel", "sack", "wagon",
        ];
        const LIGHTS: [&str; 5] = ["lantern", "torch", "lamp", "brazier", "candle"];
        const GREEN: [&str; 7] = ["tree", "bush", "grass", "flower", "shrub", "fern", "leaf"];
        for d in &source.doodads {
            let lower = d.model.to_ascii_lowercase();
            if GREEN.iter().any(|g| lower.contains(g)) {
                continue;
            }
            if LIGHTS.iter().any(|l| lower.contains(l)) {
                if !self.lights.contains(&d.model) {
                    self.lights.push(d.model.clone());
                }
            } else if PROPS.iter().any(|p| lower.contains(p)) && !self.props.contains(&d.model) {
                self.props.push(d.model.clone());
            }
        }
    }
}

impl RiftComposer {
    /// The composer of tile (`tile_x`, `tile_y`); `None` when the layout doesn't cover it.
    pub fn new(layout: Arc<RiftLayout>, tile_x: u32, tile_y: u32) -> Option<Self> {
        let (gx, gy) = (tile_y as usize, tile_x as usize);
        let ox = gx.checked_sub(usize::from(layout.gx0))?;
        let oy = gy.checked_sub(usize::from(layout.gy0))?;
        if ox > 1 || oy > 1 || layout.pieces.len() < RIFT_PIECES * RIFT_PIECES {
            return None;
        }
        let mut by_source: HashMap<(u8, u8), Vec<Wanted>> = HashMap::new();
        for ca in 0..16 {
            for cb in 0..16 {
                let (la, lb) = (ox * 16 + ca, oy * 16 + cb);
                let [sgx, sgy, sa, sb] = layout.pieces[(la / 2) * RIFT_PIECES + lb / 2];
                by_source.entry((sgx, sgy)).or_default().push(Wanted {
                    la,
                    lb,
                    src_a: usize::from(sa) + la % 2,
                    src_b: usize::from(sb) + lb % 2,
                });
            }
        }
        let mut sources: Vec<_> = by_source.into_iter().collect();
        sources.sort_by_key(|(k, _)| *k);
        // A cave reads a few sources for its kit only: the same few for all its tiles (from all
        // its pieces), or each quarter would paint and furnish itself its own way.
        if layout.theme == THEME_CAVE {
            let mut all: Vec<(u8, u8)> = layout.pieces.iter().map(|p| (p[0], p[1])).collect();
            all.sort_unstable();
            all.dedup();
            sources = all
                .into_iter()
                .take(CAVE_SOURCES)
                .map(|k| (k, Vec::new()))
                .collect();
        }
        // A flavour's borrowed tiles: read for its models only.
        for &key in borrowed_sources(layout.flavour) {
            if !sources.iter().any(|(k, _)| *k == key) {
                sources.push((key, Vec::new()));
            }
        }
        Some(Self {
            layout,
            sources,
            next: 0,
            chunks: Vec::with_capacity(256),
            doodads: Vec::new(),
            wmos: Vec::new(),
            kit: CaveKit::default(),
            dress: DressKit::default(),
            tile: (ox, oy),
        })
    }

    /// The internal path of the next source tile to read, or `None` when all are fed.
    pub fn next_source(&self) -> Option<String> {
        let ((sgx, sgy), _) = self.sources.get(self.next)?;
        // Server grid (gx, gy) is the tile file `_{gy}_{gx}`.
        Some(format!(
            "World/Maps/{SOURCE_DIR}/{SOURCE_DIR}_{sgy}_{sgx}.adt"
        ))
    }

    /// The bytes of the source [`Self::next_source`] named (`None`: unreadable, its chunks are
    /// left out).
    pub fn feed(&mut self, bytes: Option<&[u8]>) {
        let Some(((sgx, sgy), wanted)) = self.sources.get(self.next) else {
            return;
        };
        self.next += 1;
        let Some(source) = bytes.and_then(|b| adt_to_tile_mesh(b).ok()) else {
            return;
        };
        // A tile borrowed for a flavour gives that flavour's models only, never its lamps or
        // fences.
        let borrowed =
            wanted.is_empty() && borrowed_sources(self.layout.flavour).contains(&(*sgx, *sgy));
        if self.layout.theme == THEME_CAVE {
            self.kit.take(&source, self.layout.flavour, borrowed);
            return;
        }
        self.dress.take(&source, self.layout.flavour, borrowed);
        let src_nw_x = (32.0 - f64::from(*sgx)) * f64::from(TILE_SIZE);
        let src_nw_y = (32.0 - f64::from(*sgy)) * f64::from(TILE_SIZE);
        for w in wanted {
            // The source chunk by its north-west corner.
            let want_x = (src_nw_x - (w.src_a * 8) as f64 * UNIT) as f32;
            let want_y = (src_nw_y - (w.src_b * 8) as f64 * UNIT) as f32;
            let Some(src) = source.chunks.iter().find(|c| {
                c.positions
                    .first()
                    .is_some_and(|p| (p[0] - want_x).abs() < 1.0 && (p[1] - want_y).abs() < 1.0)
            }) else {
                continue;
            };
            let chunk = place_chunk(&self.layout, src, w.la, w.lb);
            place_doodads(
                &self.layout,
                src,
                &chunk,
                &source.doodads,
                &mut self.doodads,
                w.la,
                w.lb,
            );
            self.chunks.push(chunk);
        }
    }

    /// The tile: its chunks and doodads, no buildings.
    pub fn finish(mut self) -> TileMesh {
        if self.layout.theme == THEME_CAVE {
            self.kit.settle();
            let (ox, oy) = self.tile;
            for ca in 0..16 {
                for cb in 0..16 {
                    let (la, lb) = (ox * 16 + ca, oy * 16 + cb);
                    self.chunks
                        .push(cave_chunk(&self.layout, &self.kit, la, lb));
                    cave_props(&self.layout, &self.kit, la, lb, &mut self.doodads);
                }
            }
        } else {
            let (ox, oy) = self.tile;
            dress_tile(
                &self.layout,
                &self.dress,
                ox,
                oy,
                &mut self.doodads,
                &mut self.wmos,
            );
        }
        // MCIN order (row by row from the north-west), which `chunk_at` assumes of a whole tile;
        // the chunks were placed grouped by source.
        self.chunks.sort_by_key(|c| c.index_y * 16 + c.index_x);
        TileMesh {
            chunks: self.chunks,
            doodads: self.doodads,
            wmos: self.wmos,
        }
    }
}

/// `src` moved to layout chunk (`la`, `lb`) under the layout's heights.
fn place_chunk(layout: &RiftLayout, src: &ChunkMesh, la: usize, lb: usize) -> ChunkMesh {
    let mut chunk = src.clone();
    let (positions, normals) = chunk_geometry(layout, la, lb);
    chunk.positions = positions;
    chunk.normals = normals;
    // Every cell drawn: the server took no piece with holes.
    chunk.holes = 0;
    chunk.indices = full_indices();
    chunk.area_id = u32::from(layout.area);
    // MCNK IndexX runs with the tile's x (WoW Y), IndexY with WoW X.
    chunk.index_x = (lb % 16) as u32;
    chunk.index_y = (la % 16) as u32;
    chunk.impassable = false;
    chunk.liquids.clear();
    chunk
}

/// A layout chunk's 145 vertices and their normals, in stride-17 order.
fn chunk_geometry(layout: &RiftLayout, la: usize, lb: usize) -> (Vec<[f32; 3]>, Vec<[f32; 3]>) {
    // The chunk corner's lattice index from the map centre line: X = centre - (gx·128 + a)·unit.
    let ni = f64::from(layout.gx0) * 128.0 + (la * 8) as f64;
    let nj = f64::from(layout.gy0) * 128.0 + (lb * 8) as f64;
    let mut positions = Vec::with_capacity(145);
    let mut normals = Vec::with_capacity(145);
    for row in 0..9usize {
        for col in 0..9usize {
            let (i, j) = (la * 8 + row, lb * 8 + col);
            positions.push([
                lattice(ni + row as f64),
                lattice(nj + col as f64),
                layout.height(i, j),
            ]);
            normals.push(layout.normal(i, j));
        }
        if row < 8 {
            for col in 0..8usize {
                let (i, j) = (la * 8 + row, lb * 8 + col);
                let h = (layout.height(i, j)
                    + layout.height(i + 1, j)
                    + layout.height(i, j + 1)
                    + layout.height(i + 1, j + 1))
                    / 4.0;
                positions.push([
                    lattice(ni + row as f64 + 0.5),
                    lattice(nj + col as f64 + 0.5),
                    h,
                ]);
                let corners = [
                    layout.normal(i, j),
                    layout.normal(i + 1, j),
                    layout.normal(i, j + 1),
                    layout.normal(i + 1, j + 1),
                ];
                let mut n = [0.0f32; 3];
                for c in corners {
                    for k in 0..3 {
                        n[k] += c[k];
                    }
                }
                let len = (n[0] * n[0] + n[1] * n[1] + n[2] * n[2]).sqrt().max(1e-6);
                normals.push([n[0] / len, n[1] / len, n[2] / len]);
            }
        }
    }
    (positions, normals)
}

/// How steep the ground is at a (fractional) node: 0 flat, 1 sheer.
fn steepness(layout: &RiftLayout, i: f32, j: f32) -> f32 {
    let (i0, j0) = (i.max(0.0).floor() as usize, j.max(0.0).floor() as usize);
    let (ti, tj) = (i - i0 as f32, j - j0 as f32);
    let s = |a: usize, b: usize| 1.0 - layout.normal(a, b)[2];
    let top = s(i0, j0) * (1.0 - tj) + s(i0, j0 + 1) * tj;
    let bottom = s(i0 + 1, j0) * (1.0 - tj) + s(i0 + 1, j0 + 1) * tj;
    top * (1.0 - ti) + bottom * ti
}

/// The rock's share of a cave's ground at a steepness: the floor's below, the wall's above.
fn wall_weight(steep: f32) -> f32 {
    ((steep - 0.12) / 0.25).clamp(0.0, 1.0)
}

/// A cave chunk: the layout's ground, the kit's floor under its rock by slope, no clutter.
fn cave_chunk(layout: &RiftLayout, kit: &CaveKit, la: usize, lb: usize) -> ChunkMesh {
    let (positions, normals) = chunk_geometry(layout, la, lb);
    let mut uvs = Vec::with_capacity(145);
    for row in 0..9u32 {
        for col in 0..9u32 {
            uvs.push([col as f32 / 8.0, row as f32 / 8.0]);
        }
        if row < 8 {
            for col in 0..8u32 {
                uvs.push([(col as f32 + 0.5) / 8.0, (row as f32 + 0.5) / 8.0]);
            }
        }
    }
    let layer_textures: Vec<String> = [kit.floor.clone(), kit.wall.clone()]
        .into_iter()
        .flatten()
        .collect();
    // Layer 1 (the rock, over the floor) by slope, a texel at a time.
    let alpha_map = (layer_textures.len() > 1).then(|| {
        let n = benilla_formats::ALPHA_MAP_SIZE as usize;
        let mut rgba = vec![0u8; n * n * 4];
        for ty in 0..n {
            for tx in 0..n {
                let i = (la * 8) as f32 + (ty as f32 + 0.5) / n as f32 * 8.0;
                let j = (lb * 8) as f32 + (tx as f32 + 0.5) / n as f32 * 8.0;
                rgba[(ty * n + tx) * 4] = (wall_weight(steepness(layout, i, j)) * 255.0) as u8;
            }
        }
        rgba
    });
    let layer_effect_ids = vec![u32::MAX; layer_textures.len()];
    ChunkMesh {
        positions,
        normals,
        uvs,
        indices: full_indices(),
        holes: 0,
        base_texture: layer_textures.first().cloned(),
        layer_textures,
        layer_effect_ids,
        alpha_map,
        shadow: None,
        pred_tex: [0; 64],
        no_effect_doodad: [true; 64],
        index_x: (lb % 16) as u32,
        index_y: (la % 16) as u32,
        area_id: u32::from(layout.area),
        impassable: false,
        liquids: Vec::new(),
    }
}

/// A placement's Y rotation (degrees) that faces its model's front along a WoW heading
/// (radians, 0 north): `placement_rotation` turns a model's +X to heading `ry + 180°`.
fn rotation_y(heading: f32) -> f32 {
    heading.to_degrees() - 180.0
}

/// What an outdoor area's sites are built from: models found in its own source tiles, so a
/// Farmlands farmstead is one of the eastern farms' houses, by name.
#[derive(Default)]
struct DressKit {
    /// Buildings (WMOs): a farm's house or barn, a watchtower.
    /// With each its origin's height over the ground it stood on in its source tile, so it sits as
    /// it was built (a buried foundation stays buried).
    houses: Vec<(String, f32)>,
    towers: Vec<(String, f32)>,
    /// Props (M2s) by use.
    fences: Vec<String>,
    hay: Vec<String>,
    crops: Vec<String>,
    farm: Vec<String>,
    camp: Vec<String>,
    fires: Vec<String>,
    logging: Vec<String>,
    obstacles: Vec<String>,
    /// Burnt farmhouses (a hideout's), with their lift; the burnt props round them.
    burnt_houses: Vec<(String, f32)>,
    burnt: Vec<String>,
    /// Every WMO the tiles hold, by its vmaps name ([`vmaps_name`]): the server names a site's
    /// building so.
    by_vmaps_name: HashMap<String, String>,
    /// A Defias camp's tents, bedrolls and crates; a den's bones; a wolf wood's trees.
    defias: Vec<String>,
    bones: Vec<String>,
    trees: Vec<String>,
}

/// Whether a lowercased name has any of the words.
fn named(lower: &str, words: &[&str]) -> bool {
    words.iter().any(|w| lower.contains(w))
}

impl DressKit {
    fn take(&mut self, source: &TileMesh, flavour: u8, borrowed: bool) {
        fn add(list: &mut Vec<String>, name: &str) {
            if list.len() < 12 && !list.iter().any(|n| n == name) {
                list.push(name.to_string());
            }
        }
        // Not a farmstead's: the towns', the abbey, the walls and the mines.
        const GRAND: [&str; 12] = [
            "abbey",
            "church",
            "cathedral",
            "stormwind",
            "keep",
            "wall",
            "bridge",
            "inn",
            "mine",
            "cave",
            "dungeon",
            "azora",
        ];
        for w in &source.wmos {
            self.by_vmaps_name
                .entry(vmaps_name(&w.model))
                .or_insert_with(|| w.model.clone());
            let lower = w.model.to_ascii_lowercase();
            if named(&lower, &GRAND) {
                continue;
            }
            let lift = source
                .chunks
                .iter()
                .find_map(|c| source_ground(c, w.position[0], w.position[1]))
                .map_or(0.0, |ground| w.position[2] - ground);
            let building = |list: &mut Vec<(String, f32)>| {
                if list.len() < 12 && !list.iter().any(|(n, _)| n == &w.model) {
                    list.push((w.model.clone(), lift));
                }
            };
            let burnt = named(&lower, &["burnt", "burned", "destroyed", "ruin"]);
            let farm = named(
                &lower,
                &["farm", "barn", "house", "shack", "cottage", "mill"],
            );
            if burnt && farm && flavour == FLAVOUR_BURNT {
                building(&mut self.burnt_houses);
            } else if borrowed {
                continue;
            } else if named(&lower, &["tower"]) {
                building(&mut self.towers);
            } else if farm && !burnt {
                building(&mut self.houses);
            }
        }
        for d in &source.doodads {
            let lower = d.model.to_ascii_lowercase();
            // A borrowed tile gives its burnt props only.
            if borrowed {
                if flavour == FLAVOUR_BURNT
                    && named(
                        &lower,
                        &[
                            "burnt", "burned", "charred", "ashes", "ashpile", "ember", "smoke",
                        ],
                    )
                {
                    add(&mut self.burnt, &d.model);
                }
                continue;
            }
            if named(&lower, &["fence"]) {
                add(&mut self.fences, &d.model);
            } else if named(&lower, &["hay"]) {
                add(&mut self.hay, &d.model);
            } else if named(
                &lower,
                &[
                    "pumpkin",
                    "wheat",
                    "cornstalk",
                    "crop",
                    "cabbage",
                    "squash",
                    "grape",
                ],
            ) {
                add(&mut self.crops, &d.model);
            } else if named(
                &lower,
                &[
                    "cart",
                    "wagon",
                    "barrel",
                    "crate",
                    "trough",
                    "well",
                    "scarecrow",
                    "sack",
                    "plow",
                ],
            ) {
                add(&mut self.farm, &d.model);
            } else if named(
                &lower,
                &["tent", "banner", "bone", "skull", "gnoll", "totem"],
            ) {
                add(&mut self.camp, &d.model);
            } else if named(&lower, &["campfire", "bonfire", "firepit", "fire"]) {
                add(&mut self.fires, &d.model);
            } else if named(
                &lower,
                &["log", "stump", "lumber", "saw", "woodpile", "axe"],
            ) {
                add(&mut self.logging, &d.model);
            }
            if flavour == FLAVOUR_BURNT
                && named(
                    &lower,
                    &[
                        "burnt", "burned", "charred", "ashes", "ashpile", "ember", "smoke",
                    ],
                )
            {
                add(&mut self.burnt, &d.model);
            }
            if named(
                &lower,
                &["tent", "bedroll", "crate", "barrel", "sack", "lantern"],
            ) && !named(&lower, &["gnoll"])
            {
                add(&mut self.defias, &d.model);
            }
            if named(&lower, &["bone", "skull", "carcass", "ribcage", "skeleton"]) {
                add(&mut self.bones, &d.model);
            }
            if flavour == FLAVOUR_WOLVES
                && named(&lower, &["tree"])
                && !named(&lower, &["stump", "log", "dead", "street"])
            {
                add(&mut self.trees, &d.model);
            }
            if named(&lower, &["rock", "boulder", "stump", "log"])
                && !named(&lower, &["tree", "cliff"])
            {
                add(&mut self.obstacles, &d.model);
            }
        }
    }
}

/// Doodad and building ids for dressing, apart from moved and cave props: chunk index, then a
/// count with bit 16 set (cave props take bit 17).
fn dress_uid(chunk_index: u32, ordinal: u32) -> u32 {
    RIFT_UID_BASE | (chunk_index << 18) | 0x1_0000 | (ordinal & 0xFFFF)
}

/// A model path as the server's vmaps name it, compared without case: the file's name, its
/// spaces underscores.
fn vmaps_name(path: &str) -> String {
    path.rsplit(['\\', '/'])
        .next()
        .unwrap_or(path)
        .to_ascii_lowercase()
        .replace(' ', "_")
}

/// One of a site's buildings, with its lift over the ground.
fn pick_building(list: &[(String, f32)], rng: &mut Rng) -> Option<(String, f32)> {
    (!list.is_empty()).then(|| list[rng.next() as usize % list.len()].clone())
}

/// A site's building and its origin's height: the one the server raised (where its collision
/// is), else (an older server, one without vmaps, a site it raised none at, or a model no tile
/// here holds) one picked from `list`, without collision.
fn site_building(
    layout: &RiftLayout,
    kit: &DressKit,
    index: usize,
    site: &RiftSite,
    list: &[(String, f32)],
    rng: &mut Rng,
) -> Option<(String, f32)> {
    let picked = pick_building(list, rng);
    if let Some((name, z)) = layout
        .buildings
        .get(index)
        .filter(|(name, _)| !name.is_empty())
    {
        if let Some(model) = kit.by_vmaps_name.get(&vmaps_name(name)) {
            return Some((model.clone(), *z));
        }
        bevy::log::warn!("rift: the server's building {name} is in none of the area's tiles");
    }
    picked.map(|(model, lift)| (model, site.z + lift))
}

/// The tile's dressing: the sites whose centre it holds, and obstacles on its open ground.
fn dress_tile(
    layout: &RiftLayout,
    kit: &DressKit,
    ox: usize,
    oy: usize,
    doodads: &mut Vec<Doodad>,
    wmos: &mut Vec<WmoInstance>,
) {
    let place = |model: &str, x: f32, y: f32, heading: f32, scale: f32, uid: u32| Doodad {
        model: model.to_string(),
        position: [x, y, layout.ground(x, y)],
        rotation: [0.0, rotation_y(heading), 0.0],
        scale,
        unique_id: uid,
    };
    for (index, site) in layout.sites.iter().enumerate() {
        let (si, sj) = layout.node_at(site.x, site.y);
        let (ti, tj) = (si - (ox * 128) as f64, sj - (oy * 128) as f64);
        if !(0.0..128.0).contains(&ti) || !(0.0..128.0).contains(&tj) {
            continue;
        }
        let chunk_index = ((si / 8.0) as u32).min(31) * 32 + ((sj / 8.0) as u32).min(31);
        let mut rng = Rng((u64::from(layout.id) << 24 ^ index as u64 ^ 0xD1B5_4A32_D192_ED03) | 1);
        let mut n = 0u32;
        let mut uid = || {
            n += 1;
            dress_uid(chunk_index, n)
        };
        let pick = |list: &Vec<String>, rng: &mut Rng| -> Option<String> {
            (!list.is_empty()).then(|| list[rng.next() as usize % list.len()].clone())
        };
        let (cx, cy, r, o) = (site.x, site.y, site.radius, site.o);
        let at = |a: f32, d: f32| (cx + (o + a).cos() * d, cy + (o + a).sin() * d);
        let put =
            |list: &Vec<String>, a: f32, d: f32, rng: &mut Rng, out: &mut Vec<Doodad>, uid: u32| {
                if let Some(model) = pick(list, rng) {
                    let (x, y) = at(a, d);
                    out.push(place(
                        &model,
                        x,
                        y,
                        o + a + rng.unit() * 0.6 - 0.3,
                        0.9 + rng.unit() * 0.2,
                        uid,
                    ));
                }
            };
        // A fence ring at the edge, a gap facing the site's way in.
        let fence_ring =
            |rng: &mut Rng, out: &mut Vec<Doodad>, uid: &mut dyn FnMut() -> u32, radius: f32| {
                let Some(model) = pick(&kit.fences, rng) else {
                    return;
                };
                let steps = ((std::f32::consts::TAU * radius) / 3.5) as usize;
                for k in 0..steps {
                    let a = std::f32::consts::TAU * k as f32 / steps as f32;
                    if a < 0.5 || a > std::f32::consts::TAU - 0.5 {
                        continue; // the gate
                    }
                    let (x, y) = at(a, radius);
                    out.push(place(
                        &model,
                        x,
                        y,
                        o + a + std::f32::consts::FRAC_PI_2,
                        1.0,
                        uid(),
                    ));
                }
            };
        match site.kind {
            SITE_FARMSTEAD => {
                let houses = if layout.flavour == FLAVOUR_BURNT && !kit.burnt_houses.is_empty() {
                    &kit.burnt_houses
                } else {
                    &kit.houses
                };
                if let Some((model, z)) = site_building(layout, kit, index, site, houses, &mut rng)
                {
                    wmos.push(WmoInstance {
                        model,
                        position: [cx, cy, z],
                        rotation: [0.0, rotation_y(o), 0.0],
                        unique_id: uid(),
                        doodad_set: 0,
                        name_set: 0,
                    });
                }
                fence_ring(&mut rng, doodads, &mut uid, r * 0.95);
                for k in 0..2 {
                    put(
                        &kit.hay,
                        2.2 + k as f32 * 0.5,
                        r * 0.7,
                        &mut rng,
                        doodads,
                        uid(),
                    );
                }
                for k in 0..3 {
                    put(
                        &kit.farm,
                        3.6 + k as f32 * 0.6,
                        r * 0.75,
                        &mut rng,
                        doodads,
                        uid(),
                    );
                }
            }
            SITE_PLOT => {
                fence_ring(&mut rng, doodads, &mut uid, r);
                if let Some(model) = pick(&kit.crops, &mut rng) {
                    // Rows across the plot.
                    let (fx, fy) = (o.cos(), o.sin());
                    let (sx, sy) = (-fy, fx);
                    let mut row = -r * 0.7;
                    while row <= r * 0.7 {
                        let mut along = -r * 0.7;
                        while along <= r * 0.7 {
                            if row * row + along * along < r * r * 0.6 {
                                let (x, y) =
                                    (cx + fx * along + sx * row, cy + fy * along + sy * row);
                                doodads.push(place(&model, x, y, o, 0.9 + rng.unit() * 0.2, uid()));
                            }
                            along += 2.5;
                        }
                        row += 3.0;
                    }
                }
                put(&kit.farm, 0.0, r + 2.0, &mut rng, doodads, uid());
            }
            SITE_CAMP => {
                put(&kit.fires, 0.0, 0.0, &mut rng, doodads, uid());
                for k in 0..3 {
                    let a = std::f32::consts::TAU * k as f32 / 3.0;
                    put(&kit.camp, a, r * 0.6, &mut rng, doodads, uid());
                }
                for k in 0..3 {
                    put(
                        &kit.camp,
                        1.0 + k as f32 * 2.1,
                        r * 0.9,
                        &mut rng,
                        doodads,
                        uid(),
                    );
                }
            }
            SITE_TOWER => {
                if let Some((model, z)) =
                    site_building(layout, kit, index, site, &kit.towers, &mut rng)
                {
                    wmos.push(WmoInstance {
                        model,
                        position: [cx, cy, z],
                        rotation: [0.0, rotation_y(o), 0.0],
                        unique_id: uid(),
                        doodad_set: 0,
                        name_set: 0,
                    });
                }
                for k in 0..4 {
                    put(
                        &kit.obstacles,
                        k as f32 * 1.6,
                        r * 0.9,
                        &mut rng,
                        doodads,
                        uid(),
                    );
                }
            }
            SITE_DEFIAS_CAMP => {
                put(&kit.fires, 0.0, 0.0, &mut rng, doodads, uid());
                for k in 0..6 {
                    let a = std::f32::consts::TAU * k as f32 / 6.0;
                    put(
                        &kit.defias,
                        a,
                        r * (0.5 + 0.08 * k as f32),
                        &mut rng,
                        doodads,
                        uid(),
                    );
                }
            }
            SITE_DEN => {
                for k in 0..5 {
                    put(
                        &kit.bones,
                        k as f32 * 1.3,
                        r * (0.2 + 0.15 * k as f32),
                        &mut rng,
                        doodads,
                        uid(),
                    );
                }
                for k in 0..3 {
                    put(
                        &kit.obstacles,
                        0.6 + k as f32 * 2.0,
                        r * 0.9,
                        &mut rng,
                        doodads,
                        uid(),
                    );
                }
            }
            SITE_LOGGING => {
                for k in 0..7 {
                    put(
                        &kit.logging,
                        k as f32 * 0.9,
                        r * (0.3 + 0.1 * k as f32),
                        &mut rng,
                        doodads,
                        uid(),
                    );
                }
            }
            _ => {}
        }
        // A hideout's farms burnt: charred props about the farmstead.
        if site.kind == SITE_FARMSTEAD && layout.flavour == FLAVOUR_BURNT {
            for k in 0..4 {
                put(
                    &kit.burnt,
                    0.8 + k as f32 * 1.4,
                    r * 0.6,
                    &mut rng,
                    doodads,
                    uid(),
                );
            }
        }
    }
    // A wolf wood: trees thick at the walls' feet.
    if layout.flavour == FLAVOUR_WOLVES && !kit.trees.is_empty() {
        for ca in 0..16 {
            for cb in 0..16 {
                let (la, lb) = (ox * 16 + ca, oy * 16 + cb);
                let chunk_index = (la * 32 + lb) as u32;
                let mut rng = Rng((u64::from(layout.id) << 30
                    ^ u64::from(chunk_index)
                    ^ 0x9E37_79B9_7F4A_7C15)
                    | 1);
                for k in 0..4u32 {
                    let (i, j) = (
                        (la * 8) as f32 + rng.unit() * 8.0,
                        (lb * 8) as f32 + rng.unit() * 8.0,
                    );
                    let here = layout.ground_at_node(i, j);
                    let walled = [(-3.0, 0.0), (3.0, 0.0), (0.0, -3.0), (0.0, 3.0)]
                        .iter()
                        .any(|(di, dj)| layout.ground_at_node(i + di, j + dj) > here + 8.0);
                    if !walled || steepness(layout, i, j) > 0.2 {
                        continue;
                    }
                    let x = lattice(f64::from(layout.gx0) * 128.0 + f64::from(i));
                    let y = lattice(f64::from(layout.gy0) * 128.0 + f64::from(j));
                    if layout
                        .marks
                        .iter()
                        .any(|&(_, mx, my)| (mx - x).hypot(my - y) < 14.0)
                        || layout
                            .sites
                            .iter()
                            .any(|s| (s.x - x).hypot(s.y - y) < s.radius + 4.0)
                    {
                        continue;
                    }
                    let model = &kit.trees[rng.next() as usize % kit.trees.len()];
                    doodads.push(place(
                        model,
                        x,
                        y,
                        rng.unit() * std::f32::consts::TAU,
                        0.8 + rng.unit() * 0.5,
                        dress_uid(chunk_index, 0xE000 + k),
                    ));
                }
            }
        }
    }
    // Obstacles: now and then on a chunk's open ground, clear of the sites.
    if kit.obstacles.is_empty() {
        return;
    }
    for ca in 0..16 {
        for cb in 0..16 {
            let (la, lb) = (ox * 16 + ca, oy * 16 + cb);
            let chunk_index = (la * 32 + lb) as u32;
            let mut rng = Rng((u64::from(layout.id) << 28
                ^ u64::from(chunk_index)
                ^ 0x2545_F491_4F6C_DD1D)
                | 1);
            if rng.unit() > 0.3 {
                continue;
            }
            let (i, j) = (
                (la * 8) as f32 + 1.0 + rng.unit() * 6.0,
                (lb * 8) as f32 + 1.0 + rng.unit() * 6.0,
            );
            if steepness(layout, i, j) > 0.15 {
                continue;
            }
            let x = lattice(f64::from(layout.gx0) * 128.0 + f64::from(i));
            let y = lattice(f64::from(layout.gy0) * 128.0 + f64::from(j));
            if layout
                .sites
                .iter()
                .any(|s| (s.x - x).hypot(s.y - y) < s.radius + 6.0)
            {
                continue;
            }
            // Nor on the arrival, the exit, the way back or a side area's door.
            if layout
                .marks
                .iter()
                .any(|&(_, mx, my)| (mx - x).hypot(my - y) < 14.0)
            {
                continue;
            }
            let model = &kit.obstacles[rng.next() as usize % kit.obstacles.len()];
            doodads.push(place(
                model,
                x,
                y,
                rng.unit() * std::f32::consts::TAU,
                0.8 + rng.unit() * 0.5,
                dress_uid(chunk_index, 0xF000),
            ));
        }
    }
}

/// A small seeded generator, the same props for every client of a layout.
struct Rng(u64);

impl Rng {
    fn next(&mut self) -> u32 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        (self.0 >> 32) as u32
    }

    fn unit(&mut self) -> f32 {
        self.next() as f32 / u32::MAX as f32
    }
}

/// A cave chunk's props: on flat floor beside a wall, a lantern or torch now and then (their
/// models carry their own light).
fn cave_props(layout: &RiftLayout, kit: &CaveKit, la: usize, lb: usize, out: &mut Vec<Doodad>) {
    if kit.props.is_empty() && kit.lights.is_empty() && kit.extra.is_empty() {
        return;
    }
    let chunk_index = (la * 32 + lb) as u32;
    let mut rng =
        Rng((u64::from(layout.id) << 20 ^ u64::from(chunk_index) ^ 0x9E37_79B9_7F4A_7C15) | 1);
    let mut ordinal = 0u32;
    for row in 0..8usize {
        for col in 0..8usize {
            let (i, j) = ((la * 8 + row) as f32 + 0.5, (lb * 8 + col) as f32 + 0.5);
            if steepness(layout, i, j) > 0.05 {
                continue;
            }
            // Beside a wall rising over it, not on the rim above one.
            let here = layout.ground_at_node(i, j);
            let walled = [(-3.0, 0.0), (3.0, 0.0), (0.0, -3.0), (0.0, 3.0)]
                .iter()
                .any(|(di, dj)| layout.ground_at_node(i + di, j + dj) > here + 10.0);
            if !walled || rng.unit() > 0.12 {
                continue;
            }
            // One in six a light, a third the flavour's own (webs, candles, scaffolds).
            let roll = rng.next();
            let pool = if !kit.lights.is_empty()
                && (roll.is_multiple_of(6) || (kit.props.is_empty() && kit.extra.is_empty()))
            {
                &kit.lights
            } else if !kit.extra.is_empty() && (roll % 3 == 1 || kit.props.is_empty()) {
                &kit.extra
            } else if !kit.props.is_empty() {
                &kit.props
            } else {
                &kit.lights
            };
            let model = pool[rng.next() as usize % pool.len()].clone();
            let x = lattice(f64::from(layout.gx0) * 128.0 + f64::from(i));
            let y = lattice(f64::from(layout.gy0) * 128.0 + f64::from(j));
            out.push(Doodad {
                model,
                position: [x, y, layout.ground(x, y)],
                rotation: [0.0, rng.unit() * 360.0, 0.0],
                scale: 0.8 + rng.unit() * 0.4,
                unique_id: RIFT_UID_BASE | (chunk_index << 18) | (0x2_0000 | (ordinal & 0x1_FFFF)),
            });
            ordinal += 1;
        }
    }
}

/// Four fan triangles per cell over all 64 cells, as `adt_to_tile_mesh` winds them.
fn full_indices() -> Vec<u32> {
    let mut indices = Vec::with_capacity(8 * 8 * 12);
    for row in 0..8u32 {
        for col in 0..8u32 {
            let tl = row * 17 + col;
            let tr = tl + 1;
            let bl = tl + 17;
            let br = bl + 1;
            let ctr = row * 17 + 9 + col;
            indices.extend_from_slice(&[ctr, tl, bl, ctr, bl, br, ctr, br, tr, ctr, tr, tl]);
        }
    }
    indices
}

/// The height of a source chunk's outer grid under a WoW (x, y), bilinear; `None` off it.
fn source_ground(src: &ChunkMesh, x: f32, y: f32) -> Option<f32> {
    let nw = *src.positions.first()?;
    let r = (nw[0] - x) / UNIT as f32;
    let c = (nw[1] - y) / UNIT as f32;
    if !(0.0..8.0).contains(&r) || !(0.0..8.0).contains(&c) {
        return None;
    }
    let (r0, c0) = (r.floor() as usize, c.floor() as usize);
    let (tr, tc) = (r - r0 as f32, c - c0 as f32);
    let h = |row: usize, col: usize| src.positions.get(row * 17 + col).map_or(nw[2], |p| p[2]);
    let top = h(r0, c0) * (1.0 - tc) + h(r0, c0 + 1) * tc;
    let bottom = h(r0 + 1, c0) * (1.0 - tc) + h(r0 + 1, c0 + 1) * tc;
    Some(top * (1.0 - tr) + bottom * tr)
}

/// The doodads standing on `src` (by their centre), moved with it onto `placed`.
fn place_doodads(
    layout: &RiftLayout,
    src: &ChunkMesh,
    placed: &ChunkMesh,
    source_doodads: &[Doodad],
    out: &mut Vec<Doodad>,
    la: usize,
    lb: usize,
) {
    let (Some(from), Some(to)) = (src.positions.first(), placed.positions.first()) else {
        return;
    };
    let (dx, dy) = (to[0] - from[0], to[1] - from[1]);
    let chunk_index = (la * 32 + lb) as u32;
    let mut ordinal = 0u32;
    let mut seen = Vec::new();
    for d in source_doodads {
        if seen.contains(&d.unique_id) {
            continue;
        }
        let Some(was) = source_ground(src, d.position[0], d.position[1]) else {
            continue;
        };
        seen.push(d.unique_id);
        let (x, y) = (d.position[0] + dx, d.position[1] + dy);
        // A site's ground is its own: no source tree in a farmhouse or a crop row.
        if layout
            .sites
            .iter()
            .any(|s| (s.x - x).hypot(s.y - y) < s.radius + 2.0)
        {
            continue;
        }
        let now = layout.ground(x, y);
        let mut moved = d.clone();
        moved.position = [x, y, d.position[2] + (now - was)];
        // Unique per placement: the same source tree may stand in several pieces.
        moved.unique_id = RIFT_UID_BASE | (chunk_index << 18) | (ordinal & 0x3_FFFF);
        ordinal += 1;
        out.push(moved);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn flat_layout(h: f32) -> RiftLayout {
        RiftLayout {
            id: 3,
            gx0: 20,
            gy0: 23,
            area: 12,
            pieces: vec![[48, 32, 0, 0]; RIFT_PIECES * RIFT_PIECES],
            heights: vec![h; RIFT_NODES * RIFT_NODES],
            theme: THEME_OUTDOOR,
            marks: Vec::new(),
            sites: Vec::new(),
            flavour: 0,
            buildings: Vec::new(),
        }
    }

    #[test]
    fn a_rift_tile_path_parses_and_a_covered_one_is_served() {
        assert_eq!(
            parse_rift_path("World/Maps/ArpgRift7/ArpgRift7_23_20.adt"),
            Some((7, 23, 20))
        );
        assert_eq!(
            parse_rift_path("world\\maps\\arpgrift7\\arpgrift7_24_21.adt"),
            Some((7, 24, 21))
        );
        assert_eq!(
            parse_rift_path("World/Maps/Azeroth/Azeroth_32_48.adt"),
            None
        );
        assert_eq!(parse_rift_path("World/Maps/ArpgRift7/ArpgRift7.wdt"), None);
        let layout = flat_layout(60.0);
        // Grid (gx 20, gy 23) is tile (23, 20).
        assert!(layout.covers(23, 20));
        assert!(layout.covers(24, 21));
        assert!(!layout.covers(20, 23));
        assert!(!layout.covers(25, 20));
    }

    #[test]
    fn a_cave_is_painted_rock_on_its_walls_and_furnished_beside_them() {
        let mut layout = flat_layout(60.0);
        layout.theme = THEME_CAVE;
        // A wall: the ground rises 30 yards over two nodes across i 40..42.
        for i in 0..RIFT_NODES {
            for j in 0..RIFT_NODES {
                layout.heights[i * RIFT_NODES + j] = 60.0 + 15.0 * (i.clamp(40, 42) - 40) as f32;
            }
        }
        let kit = CaveKit {
            floor: Some("Tileset\\Elwynn\\ElwynnDirt.blp".into()),
            wall: Some("Tileset\\Elwynn\\ElwynnRock.blp".into()),
            props: vec!["World\\Generic\\MineCart.mdx".into()],
            lights: vec!["World\\Generic\\Lantern.mdx".into()],
            extra: vec!["World\\Generic\\SpiderWeb.mdx".into()],
            seen: Vec::new(),
        };
        // Chunk 5 along X spans nodes 40..48: its first rows are the wall.
        let chunk = cave_chunk(&layout, &kit, 5, 3);
        assert_eq!(chunk.layer_textures.len(), 2);
        assert!(chunk.no_effect_doodad.iter().all(|n| *n));
        let alpha = chunk.alpha_map.as_ref().expect("two layers");
        let n = benilla_formats::ALPHA_MAP_SIZE as usize;
        assert!(alpha[(4 * n + 10) * 4] > 200, "the wall is rock");
        assert_eq!(alpha[((n - 4) * n + 10) * 4], 0, "the floor is dirt");
        // Props stand on the floor at the wall's foot (chunk 4, below it), on the ground, and
        // none on the rim above (chunk 5's far side).
        let mut props = Vec::new();
        for lb in 0..32 {
            cave_props(&layout, &kit, 4, lb, &mut props);
        }
        assert!(!props.is_empty());
        assert!(
            props.iter().any(|p| p.model.contains("Web")),
            "the flavour's own among them"
        );
        assert!(
            props.iter().any(|p| p.model.contains("Lantern")),
            "and lights"
        );
        for p in &props {
            assert!((p.position[2] - layout.ground(p.position[0], p.position[1])).abs() < 0.01);
            assert!(
                p.position[2] < 61.0,
                "on the floor below the wall: {}",
                p.position[2]
            );
        }
        let mut rim = Vec::new();
        for lb in 0..32 {
            cave_props(&layout, &kit, 5, lb, &mut rim);
        }
        assert!(
            rim.iter().all(|p| p.position[2] < 61.0),
            "nothing on the rim"
        );
        let mut again = Vec::new();
        for lb in 0..32 {
            cave_props(&layout, &kit, 4, lb, &mut again);
        }
        assert_eq!(
            props.len(),
            again.len(),
            "the same props for the same layout"
        );
        // A cave reads few sources, the same for each of its tiles.
        let layout = Arc::new(layout);
        let first = RiftComposer::new(layout.clone(), 23, 20).expect("covered");
        let second = RiftComposer::new(layout, 24, 21).expect("covered");
        assert!(first.sources.len() <= CAVE_SOURCES);
        let keys = |c: &RiftComposer| c.sources.iter().map(|(k, _)| *k).collect::<Vec<_>>();
        assert_eq!(keys(&first), keys(&second));
    }

    #[test]
    fn sites_are_dressed_once_by_the_tile_holding_them_from_the_area_s_own_models() {
        let mut layout = flat_layout(60.0);
        let at = |i: f32, j: f32| {
            (
                (32.0 - 20.0) * TILE_SIZE - i * UNIT as f32,
                (32.0 - 23.0) * TILE_SIZE - j * UNIT as f32,
            )
        };
        let (fx, fy) = at(40.0, 40.0); // tile (0, 0) of the layout
        let (px, py) = at(200.0, 60.0); // tile (1, 0)
        layout.sites = vec![
            RiftSite {
                kind: SITE_FARMSTEAD,
                x: fx,
                y: fy,
                z: 60.0,
                o: 1.0,
                radius: 22.0,
            },
            RiftSite {
                kind: SITE_PLOT,
                x: px,
                y: py,
                z: 60.0,
                o: 0.3,
                radius: 16.0,
            },
        ];
        let kit = DressKit {
            houses: vec![(
                "World\\wmo\\Azeroth\\Buildings\\HumanFarm\\Farm.wmo".into(),
                0.5,
            )],
            fences: vec!["World\\Generic\\Human\\Passive Doodads\\Fences\\Fence.mdx".into()],
            crops: vec!["World\\Azeroth\\Elwynn\\Passive Doodads\\Pumpkin.mdx".into()],
            hay: vec!["World\\Generic\\Hay.mdx".into()],
            farm: vec!["World\\Generic\\Cart.mdx".into()],
            obstacles: vec!["World\\Generic\\Rock.mdx".into()],
            ..DressKit::default()
        };
        let (mut d0, mut w0) = (Vec::new(), Vec::new());
        dress_tile(&layout, &kit, 0, 0, &mut d0, &mut w0);
        let (mut d1, mut w1) = (Vec::new(), Vec::new());
        dress_tile(&layout, &kit, 1, 0, &mut d1, &mut w1);
        // The farmstead's house is tile (0, 0)'s, on the levelled ground; the plot has none.
        assert_eq!(w0.len(), 1);
        assert!(w1.is_empty());
        assert_eq!(
            w0[0].position,
            [fx, fy, 60.5],
            "levelled ground, lifted as built"
        );
        // A fence ring round the farmstead, crops in the plot, and every id distinct.
        let fences = d0.iter().filter(|d| d.model.contains("Fence")).count();
        assert!(fences > 20, "{fences} fence pieces");
        assert!(d1.iter().filter(|d| d.model.contains("Pumpkin")).count() > 10);
        let mut ids: Vec<u32> = d0
            .iter()
            .chain(&d1)
            .map(|d| d.unique_id)
            .chain(w0.iter().map(|w| w.unique_id))
            .collect();
        let n = ids.len();
        ids.sort_unstable();
        ids.dedup();
        assert_eq!(ids.len(), n, "unique ids");
        // Obstacles keep clear of the sites.
        for d in d0.iter().filter(|d| d.model.contains("Rock")) {
            assert!((d.position[0] - fx).hypot(d.position[1] - fy) > 22.0);
        }
    }

    #[test]
    fn a_site_raises_the_building_the_server_named_where_its_collision_is() {
        assert_eq!(
            vmaps_name("World\\wmo\\Azeroth\\Buildings\\Human Farm\\Human Farm.WMO"),
            "human_farm.wmo"
        );
        let mut layout = flat_layout(60.0);
        let (fx, fy) = (
            (32.0 - 20.0) * TILE_SIZE - 40.0 * UNIT as f32,
            (32.0 - 23.0) * TILE_SIZE - 40.0 * UNIT as f32,
        );
        layout.sites = vec![RiftSite {
            kind: SITE_FARMSTEAD,
            x: fx,
            y: fy,
            z: 60.0,
            o: 1.0,
            radius: 22.0,
        }];
        let barn = "World\\wmo\\Azeroth\\Buildings\\Barn\\Elwynn Barn.wmo".to_string();
        let mut kit = DressKit {
            houses: vec![("Farm.wmo".into(), 0.5)],
            ..DressKit::default()
        };
        kit.by_vmaps_name.insert(vmaps_name(&barn), barn.clone());
        layout.buildings = vec![("Elwynn_Barn.wmo".into(), 61.25)];
        let (mut d, mut w) = (Vec::new(), Vec::new());
        dress_tile(&layout, &kit, 0, 0, &mut d, &mut w);
        assert_eq!(w.len(), 1);
        assert_eq!(
            (w[0].model.as_str(), w[0].position[2]),
            (barn.as_str(), 61.25)
        );
        // None raised there (a server without vmaps): the client's own pick, as before.
        layout.buildings = vec![(String::new(), 60.0)];
        let (mut d, mut w) = (Vec::new(), Vec::new());
        dress_tile(&layout, &kit, 0, 0, &mut d, &mut w);
        assert_eq!((w[0].model.as_str(), w[0].position[2]), ("Farm.wmo", 60.5));
    }

    #[test]
    fn a_hideout_is_burnt_and_camped_and_a_wolf_run_is_denned() {
        let mut layout = flat_layout(60.0);
        let at = |i: f32, j: f32| {
            (
                (32.0 - 20.0) * TILE_SIZE - i * UNIT as f32,
                (32.0 - 23.0) * TILE_SIZE - j * UNIT as f32,
            )
        };
        let (fx, fy) = at(40.0, 40.0);
        let (cx, cy) = at(90.0, 40.0);
        let (dx, dy) = at(40.0, 90.0);
        layout.flavour = FLAVOUR_BURNT;
        layout.sites = vec![
            RiftSite {
                kind: SITE_FARMSTEAD,
                x: fx,
                y: fy,
                z: 60.0,
                o: 0.0,
                radius: 22.0,
            },
            RiftSite {
                kind: SITE_DEFIAS_CAMP,
                x: cx,
                y: cy,
                z: 60.0,
                o: 0.0,
                radius: 13.0,
            },
            RiftSite {
                kind: SITE_DEN,
                x: dx,
                y: dy,
                z: 60.0,
                o: 0.0,
                radius: 10.0,
            },
        ];
        let kit = DressKit {
            houses: vec![("Farm.wmo".into(), 0.0)],
            burnt_houses: vec![("BurntFarm.wmo".into(), -0.5)],
            burnt: vec!["Charred.mdx".into()],
            defias: vec!["Tent.mdx".into()],
            bones: vec!["Bones.mdx".into()],
            fires: vec!["Campfire.mdx".into()],
            obstacles: vec!["Rock.mdx".into()],
            ..DressKit::default()
        };
        let (mut d, mut w) = (Vec::new(), Vec::new());
        dress_tile(&layout, &kit, 0, 0, &mut d, &mut w);
        assert_eq!(w.len(), 1);
        assert_eq!(
            w[0].model, "BurntFarm.wmo",
            "a hideout's farm is a burnt one"
        );
        assert_eq!(w[0].position[2], 59.5);
        let count = |name: &str| d.iter().filter(|p| p.model == name).count();
        assert!(count("Charred.mdx") >= 3);
        assert!(count("Tent.mdx") >= 5 && count("Campfire.mdx") == 1);
        assert!(count("Bones.mdx") >= 4);
        // Not burnt: the plain farmhouse.
        layout.flavour = 0;
        let (mut d2, mut w2) = (Vec::new(), Vec::new());
        dress_tile(&layout, &kit, 0, 0, &mut d2, &mut w2);
        assert_eq!(w2[0].model, "Farm.wmo");
        assert!(d2.iter().all(|p| p.model != "Charred.mdx"));
    }

    #[test]
    fn a_rift_wdt_flags_only_its_layout_s_tiles() {
        let mut layout = flat_layout(60.0);
        layout.id = 990_001;
        publish(layout);
        let bytes = rift_wdt("World\\Maps\\ArpgRift990001\\ArpgRift990001.wdt")
            .expect("a known layout's WDT");
        let wdt = benilla_formats::WdtReader::new(
            std::io::Cursor::new(bytes),
            benilla_formats::WowVersion::Classic,
        )
        .read()
        .expect("parses");
        // Tile x is server grid gy, tile y grid gx (flat_layout's block starts at gx 20, gy 23).
        let tiles: Vec<(usize, usize)> = (0..64)
            .flat_map(|x| (0..64).map(move |y| (x, y)))
            .filter(|&(x, y)| wdt.get_tile(x, y).is_some_and(|t| t.has_adt))
            .collect();
        assert_eq!(tiles, vec![(23, 20), (23, 21), (24, 20), (24, 21)]);
        assert!(wdt.global_wmo().is_none());
        assert!(rift_wdt("World\\Maps\\ArpgRift990002\\ArpgRift990002.wdt").is_none());
        assert!(rift_wdt("World\\Maps\\ArpgRift990001\\ArpgRift990001_23_20.adt").is_none());
    }

    #[test]
    fn a_flavour_borrows_its_tiles_once() {
        let mut layout = flat_layout(60.0);
        layout.flavour = FLAVOUR_WEBS;
        layout.theme = THEME_CAVE;
        let composer = RiftComposer::new(Arc::new(layout), 23, 20).expect("covered");
        let keys: Vec<(u8, u8)> = composer.sources.iter().map(|(k, _)| *k).collect();
        for b in borrowed_sources(FLAVOUR_WEBS) {
            assert_eq!(keys.iter().filter(|k| *k == b).count(), 1, "{b:?}");
        }
    }

    #[test]
    fn a_heading_turns_a_placed_model_to_face_it() {
        for heading in [0.0f32, 0.7, 2.0, 4.5] {
            let q = crate::coords::placement_rotation([0.0, rotation_y(heading), 0.0]);
            let w = crate::coords::bevy_to_wow(q * crate::coords::wow_to_bevy([1.0, 0.0, 0.0]));
            let got = w[1].atan2(w[0]);
            let off = (got - heading).rem_euclid(std::f32::consts::TAU);
            assert!(
                off < 1e-3 || off > std::f32::consts::TAU - 1e-3,
                "{heading} faced {got}"
            );
        }
    }

    #[test]
    fn a_composed_tile_keeps_its_chunks_in_mcin_order() {
        let layout = Arc::new(flat_layout(60.0));
        let mut composer = RiftComposer::new(layout.clone(), 23, 20).expect("covered");
        let src = blank_chunk();
        // Placed out of order, as sources group them.
        for (la, lb) in [(5, 9), (0, 0), (15, 1), (0, 15)] {
            composer.chunks.push(place_chunk(&layout, &src, la, lb));
        }
        let tile = composer.finish();
        let order: Vec<(u32, u32)> = tile.chunks.iter().map(|c| (c.index_y, c.index_x)).collect();
        assert_eq!(order, vec![(0, 0), (0, 15), (5, 9), (15, 1)]);
    }

    fn blank_chunk() -> ChunkMesh {
        ChunkMesh {
            positions: vec![[0.0, 0.0, 0.0]; 145],
            normals: Vec::new(),
            uvs: vec![[0.0, 0.0]; 145],
            indices: Vec::new(),
            holes: 0xFFFF,
            base_texture: None,
            layer_textures: Vec::new(),
            layer_effect_ids: Vec::new(),
            alpha_map: None,
            shadow: None,
            pred_tex: [0; 64],
            no_effect_doodad: [false; 64],
            index_x: 0,
            index_y: 0,
            area_id: 40,
            impassable: true,
            liquids: Vec::new(),
        }
    }

    #[test]
    fn a_placed_chunk_sits_on_the_lattice_under_the_sent_heights() {
        let mut layout = flat_layout(60.0);
        // A slope along i: each node a yard higher.
        for i in 0..RIFT_NODES {
            for j in 0..RIFT_NODES {
                layout.heights[i * RIFT_NODES + j] = 60.0 + i as f32;
            }
        }
        let src = blank_chunk();
        let chunk = place_chunk(&layout, &src, 17, 3);
        // The corner: grid 21's chunk 1 along X, grid 23's chunk 3 along Y.
        let nw = chunk.positions[0];
        let want_x = (32.0 - 21.0) * TILE_SIZE - 8.0 * UNIT as f32;
        let want_y = (32.0 - 23.0) * TILE_SIZE - 3.0 * 8.0 * UNIT as f32;
        assert!((nw[0] - want_x).abs() < 0.01, "{} vs {want_x}", nw[0]);
        assert!((nw[1] - want_y).abs() < 0.01, "{} vs {want_y}", nw[1]);
        assert_eq!(nw[2], 60.0 + 17.0 * 8.0);
        // An inner vertex is its corners' mean.
        assert_eq!(chunk.positions[9][2], 60.0 + 17.0 * 8.0 + 0.5);
        assert_eq!(chunk.indices.len(), 8 * 8 * 12);
        assert_eq!((chunk.area_id, chunk.index_x, chunk.index_y), (12, 3, 1));
        assert!(!chunk.impassable);
        // The ground agrees with the mesh at a vertex.
        assert!((layout.ground(nw[0], nw[1]) - nw[2]).abs() < 0.01);
        // Normals lean away from the rise: the ground rises southward (X falls), so +X.
        assert!(chunk.normals[0][0] > 0.0 && chunk.normals[0][2] > 0.0);
    }
}
