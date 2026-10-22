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

use benilla_formats::{adt_to_tile_mesh, ChunkMesh, Doodad, TileMesh, TILE_SIZE};

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
}

impl RiftLayout {
    /// The outer height at node (`i`, `j`), clamped to the layout.
    pub fn height(&self, i: usize, j: usize) -> f32 {
        let i = i.min(RIFT_NODES - 1);
        let j = j.min(RIFT_NODES - 1);
        self.heights.get(i * RIFT_NODES + j).copied().unwrap_or(0.0)
    }

    /// The node (fractional) under a WoW (x, y).
    fn node_at(&self, x: f32, y: f32) -> (f64, f64) {
        let i = ((32.0 - f64::from(self.gx0)) * f64::from(TILE_SIZE) - f64::from(x)) / UNIT;
        let j = ((32.0 - f64::from(self.gy0)) * f64::from(TILE_SIZE) - f64::from(y)) / UNIT;
        (i, j)
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
        Some(Self {
            layout,
            sources,
            next: 0,
            chunks: Vec::with_capacity(256),
            doodads: Vec::new(),
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
    pub fn finish(self) -> TileMesh {
        TileMesh {
            chunks: self.chunks,
            doodads: self.doodads,
            wmos: Vec::new(),
        }
    }
}

/// `src` moved to layout chunk (`la`, `lb`) under the layout's heights.
fn place_chunk(layout: &RiftLayout, src: &ChunkMesh, la: usize, lb: usize) -> ChunkMesh {
    let mut chunk = src.clone();
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
    fn a_placed_chunk_sits_on_the_lattice_under_the_sent_heights() {
        let mut layout = flat_layout(60.0);
        // A slope along i: each node a yard higher.
        for i in 0..RIFT_NODES {
            for j in 0..RIFT_NODES {
                layout.heights[i * RIFT_NODES + j] = 60.0 + i as f32;
            }
        }
        let src = ChunkMesh {
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
        };
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
