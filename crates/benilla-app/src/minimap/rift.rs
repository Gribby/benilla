//! Fork-only, not 1.12.1: the minimap of an ARPG rift (cmangos `Arpg/ArpgRift.h`), Diablo II's
//! automap. A rift has no minimap art, so its four tiles are drawn here from the layout's heights:
//! floor, walls outlined where they meet the floor, and the marks the server sends (the exit, the
//! way back, side areas' entrances, a side area's boss), each only once the player has been near
//! it. What has been seen stays seen for the layout (a town portal there and back keeps it).

use std::collections::HashMap;

use bevy::asset::RenderAssetUsages;
use bevy::prelude::*;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};

use benilla_assets::coords::bevy_to_wow;
use benilla_assets::rift::{latest, RiftLayout, RIFT_MAP, RIFT_NODES, THEME_CAVE};
use benilla_world::world_map::CurrentMap;

use super::MinimapTileCache;
use crate::player::Player;

/// Yards around the player that become seen.
const REVEAL_YD: f32 = 45.0;
/// A tile's side in pixels: two a cell.
const TILE_PX: usize = 256;
/// How often a changed reveal redraws the tiles, at most.
const REDRAW_SECS: f32 = 0.3;
/// The steepness (rise over run between neighbouring nodes) past which ground is wall.
const WALL_SLOPE: f32 = 0.9;
const UNIT: f32 = benilla_formats::TILE_SIZE / 128.0;

/// What has been seen of each recent layout, and when it was last drawn.
#[derive(Resource, Default)]
pub(super) struct RiftReveal {
    seen: HashMap<u32, Vec<bool>>,
    drawn: Option<u32>,
    dirty: bool,
    since_draw: f32,
}

pub(super) fn plugin(app: &mut App) {
    app.init_resource::<RiftReveal>()
        .add_systems(Update, reveal_rift_map);
}

fn reveal_rift_map(
    map: Option<Res<CurrentMap>>,
    player: Res<Player>,
    time: Res<Time>,
    mut reveal: ResMut<RiftReveal>,
    mut cache: ResMut<MinimapTileCache>,
    mut images: ResMut<Assets<Image>>,
) {
    if map.map(|m| m.0) != Some(RIFT_MAP) {
        return;
    }
    let Some(layout) = latest() else {
        return;
    };
    reveal.since_draw += time.delta_secs();
    // A few layouts' sight is kept: an area's chain and its side areas.
    if !reveal.seen.contains_key(&layout.id) {
        if reveal.seen.len() >= 8 {
            let oldest = *reveal.seen.keys().min().expect("non-empty");
            reveal.seen.remove(&oldest);
        }
        reveal
            .seen
            .insert(layout.id, vec![false; RIFT_NODES * RIFT_NODES]);
    }
    let wow = bevy_to_wow(player.pos);
    let seen = reveal.seen.get_mut(&layout.id).expect("inserted above");
    let newly = reveal_around(&layout, seen, wow[0], wow[1]);
    if newly || reveal.drawn != Some(layout.id) {
        reveal.dirty = true;
    }
    if !reveal.dirty || (reveal.drawn == Some(layout.id) && reveal.since_draw < REDRAW_SECS) {
        return;
    }
    reveal.dirty = false;
    reveal.since_draw = 0.0;
    reveal.drawn = Some(layout.id);
    let seen = &reveal.seen[&layout.id];
    for ox in 0..2u32 {
        for oy in 0..2u32 {
            // Server grid (gx, gy) is client tile (gy, gx).
            let key = (u32::from(layout.gy0) + oy, u32::from(layout.gx0) + ox);
            let data = draw_tile(&layout, seen, ox as usize, oy as usize);
            match cache.rift_tiles.get(&key).and_then(|h| images.get_mut(h)) {
                Some(image) => image.data = Some(data),
                None => {
                    let image = Image::new(
                        Extent3d {
                            width: TILE_PX as u32,
                            height: TILE_PX as u32,
                            depth_or_array_layers: 1,
                        },
                        TextureDimension::D2,
                        data,
                        // Gamma bytes, as the install's minimap tiles are (drawn `gamma_texel`).
                        TextureFormat::Rgba8Unorm,
                        RenderAssetUsages::RENDER_WORLD | RenderAssetUsages::MAIN_WORLD,
                    );
                    cache.rift_tiles.insert(key, images.add(image));
                }
            }
        }
    }
    // Another layout's tiles at other keys are left: their keys never meet this one's.
}

/// Mark the nodes within [`REVEAL_YD`] of (x, y) seen; true when any was new.
fn reveal_around(layout: &RiftLayout, seen: &mut [bool], x: f32, y: f32) -> bool {
    let (ci, cj) = layout.node_at(x, y);
    let r = f64::from(REVEAL_YD / UNIT);
    let max = (RIFT_NODES - 1) as f64;
    if ci < -r || cj < -r || ci > max + r || cj > max + r {
        return false;
    }
    let mut newly = false;
    let (i0, i1) = (
        (ci - r).floor().max(0.0) as usize,
        (ci + r).ceil().min(max) as usize,
    );
    let (j0, j1) = (
        (cj - r).floor().max(0.0) as usize,
        (cj + r).ceil().min(max) as usize,
    );
    for i in i0..=i1 {
        for j in j0..=j1 {
            let (di, dj) = (i as f64 - ci, j as f64 - cj);
            if di * di + dj * dj <= r * r && !seen[i * RIFT_NODES + j] {
                seen[i * RIFT_NODES + j] = true;
                newly = true;
            }
        }
    }
    newly
}

/// Whether the ground at node (i, j) is wall: steep to a neighbour.
fn is_wall(layout: &RiftLayout, i: usize, j: usize) -> bool {
    let h = layout.height(i, j);
    let rise = |a: usize, b: usize| (layout.height(a, b) - h).abs() / UNIT;
    let steep = rise((i + 1).min(RIFT_NODES - 1), j)
        .max(rise(i.saturating_sub(1), j))
        .max(rise(i, (j + 1).min(RIFT_NODES - 1)))
        .max(rise(i, j.saturating_sub(1)));
    steep > WALL_SLOPE
}

/// A tile's pixels (RGBA, rows south from its north edge, columns east from its west).
fn draw_tile(layout: &RiftLayout, seen: &[bool], ox: usize, oy: usize) -> Vec<u8> {
    let cave = layout.theme == THEME_CAVE;
    let floor: [u8; 3] = if cave { [104, 88, 66] } else { [86, 104, 60] };
    let wall: [u8; 3] = [40, 36, 32];
    let edge: [u8; 3] = [196, 184, 150];
    let unseen: [u8; 4] = [8, 8, 10, 255];
    // The floor's middle height, for a little shading by height.
    let base = layout.heights.iter().copied().fold(f32::INFINITY, f32::min);
    let mut px = vec![0u8; TILE_PX * TILE_PX * 4];
    let node_of = |p: usize| p / 2;
    for py in 0..TILE_PX {
        for pxx in 0..TILE_PX {
            let (i, j) = (ox * 128 + node_of(py), oy * 128 + node_of(pxx));
            let at = (py * TILE_PX + pxx) * 4;
            if !seen[i * RIFT_NODES + j] {
                px[at..at + 4].copy_from_slice(&unseen);
                continue;
            }
            let colour = if is_wall(layout, i, j) {
                // A wall meeting seen floor is outlined, as Diablo II's map draws its walls.
                let touches_floor =
                    [(1i32, 0i32), (-1, 0), (0, 1), (0, -1)]
                        .iter()
                        .any(|(di, dj)| {
                            let (a, b) = (i as i32 + di, j as i32 + dj);
                            (0..RIFT_NODES as i32).contains(&a)
                                && (0..RIFT_NODES as i32).contains(&b)
                                && seen[a as usize * RIFT_NODES + b as usize]
                                && !is_wall(layout, a as usize, b as usize)
                        });
                if touches_floor {
                    edge
                } else {
                    wall
                }
            } else {
                let shade = (1.0 + (layout.height(i, j) - base - 10.0) * 0.01).clamp(0.85, 1.15);
                floor.map(|c| (f32::from(c) * shade).min(255.0) as u8)
            };
            px[at..at + 3].copy_from_slice(&colour);
            px[at + 3] = 255;
        }
    }
    // The marks, once their spot has been seen.
    for &(kind, x, y) in &layout.marks {
        let (mi, mj) = layout.node_at(x, y);
        let (ti, tj) = (mi - (ox * 128) as f64, mj - (oy * 128) as f64);
        if !(0.0..128.0).contains(&ti) || !(0.0..128.0).contains(&tj) {
            continue;
        }
        let (ni, nj) = (mi.round() as usize, mj.round() as usize);
        if ni >= RIFT_NODES || nj >= RIFT_NODES || !seen[ni * RIFT_NODES + nj] {
            continue;
        }
        let colour: [u8; 3] = match kind {
            1 => [255, 210, 60],  // the exit
            2 => [90, 170, 255],  // the way back
            3 => [200, 110, 255], // a side area's entrance
            _ => [230, 60, 50],   // a side area's boss
        };
        let (cy, cx) = ((ti * 2.0) as i32, (tj * 2.0) as i32);
        for dy in -4..=4i32 {
            for dx in -4..=4i32 {
                if dx * dx + dy * dy > 16 {
                    continue;
                }
                let (y, x) = (cy + dy, cx + dx);
                if (0..TILE_PX as i32).contains(&y) && (0..TILE_PX as i32).contains(&x) {
                    let at = (y as usize * TILE_PX + x as usize) * 4;
                    let c = if dx * dx + dy * dy > 9 {
                        [20, 16, 12]
                    } else {
                        colour
                    };
                    px[at..at + 3].copy_from_slice(&c);
                    px[at + 3] = 255;
                }
            }
        }
    }
    px
}

#[cfg(test)]
mod tests {
    use super::*;

    fn walled_layout() -> RiftLayout {
        // Floor at 60, a wall rising 30 yards across nodes 100..102 along i.
        let mut heights = vec![60.0; RIFT_NODES * RIFT_NODES];
        for i in 0..RIFT_NODES {
            for j in 0..RIFT_NODES {
                heights[i * RIFT_NODES + j] = 60.0 + 15.0 * (i.clamp(100, 102) - 100) as f32;
            }
        }
        RiftLayout {
            id: 5,
            gx0: 20,
            gy0: 23,
            area: 12,
            pieces: vec![[48, 32, 0, 0]; 256],
            heights,
            theme: THEME_CAVE,
            marks: Vec::new(),
        }
    }

    #[test]
    fn only_the_ground_near_the_player_is_seen_and_walls_are_outlined() {
        let mut layout = walled_layout();
        let mut seen = vec![false; RIFT_NODES * RIFT_NODES];
        // Stand at node (95, 40), five nodes short of the wall.
        let x = (32.0 - 20.0) * benilla_formats::TILE_SIZE - 95.0 * UNIT;
        let y = (32.0 - 23.0) * benilla_formats::TILE_SIZE - 40.0 * UNIT;
        layout.marks = vec![(1, x, y), (3, x - 20.0 * UNIT, y - 120.0 * UNIT)];
        assert!(reveal_around(&layout, &mut seen, x, y));
        assert!(
            !reveal_around(&layout, &mut seen, x, y),
            "nothing new the second time"
        );
        assert!(seen[95 * RIFT_NODES + 40]);
        assert!(!seen[95 * RIFT_NODES + 200], "far off stays unseen");
        let px = draw_tile(&layout, &seen, 0, 0);
        let at = |i: usize, j: usize| {
            let k = ((i * 2) * TILE_PX + j * 2) * 4;
            [px[k], px[k + 1], px[k + 2]]
        };
        assert_eq!(at(95, 120), [8, 8, 10], "unseen is dark");
        // The wall's foot, beside seen floor, is outlined.
        assert_eq!(at(100, 40), [196, 184, 150]);
        // The exit mark is drawn where the player stands; the far door, unseen, is not.
        assert_eq!(at(95, 40), [255, 210, 60]);
        let far = at(115, 160);
        assert_ne!(far, [200, 110, 255]);
    }
}
