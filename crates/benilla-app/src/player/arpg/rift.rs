//! Fork-only, not 1.12.1: an ARPG rift's layout (`SMSG_ARPG_RIFT`, cmangos `Arpg/ArpgRift.h`).
//! The header and the bands of heights are put together here; once whole, the layout is
//! published to the tile loader (`benilla_assets::rift`) and the rifts' map is pointed at its
//! directory, so the terrain stream drops the old rift's tiles and builds the new one's.

use benilla_assets::rift::{publish, rift_dir, RiftLayout, RIFT_MAP, RIFT_NAME, RIFT_NODES};
use benilla_assets::MapCatalogRes;
use benilla_protocol::messages::arpg::{ArpgRift, ArpgRiftLayout};
use benilla_protocol::{SessionEvent, SessionEventKind};
use bevy::prelude::*;

use crate::net::NetHandlerApp;

/// A layout being received: its header and the rows come so far.
#[derive(Resource, Default)]
struct RiftAssembly {
    header: Option<ArpgRiftLayout>,
    values: Vec<i16>,
    got: Vec<bool>,
}

pub(super) fn register_net(app: &mut App) {
    app.init_resource::<RiftAssembly>()
        .net_handler(SessionEventKind::ArpgRift, on_rift);
}

fn on_rift(
    In(ev): In<SessionEvent>,
    mut assembly: ResMut<RiftAssembly>,
    catalog: Option<ResMut<MapCatalogRes>>,
) {
    let SessionEvent::ArpgRift { rift } = ev else {
        return;
    };
    if let Some(layout) = take_rift(&mut assembly, rift) {
        let id = layout.id;
        publish(layout);
        match catalog {
            Some(mut catalog) => catalog.0.set_map(RIFT_MAP, &rift_dir(id), RIFT_NAME, 1),
            None => warn!("rift {id}: no map catalog to point at it"),
        }
        info!("rift {id}: layout whole, streaming {}", rift_dir(id));
    }
}

/// One `SMSG_ARPG_RIFT` into the assembly; the layout once its last row is in.
fn take_rift(assembly: &mut RiftAssembly, rift: ArpgRift) -> Option<RiftLayout> {
    match rift {
        ArpgRift::Layout(header) => {
            if usize::from(header.nodes) != RIFT_NODES {
                warn!(
                    "rift {}: {} nodes a side, expected {RIFT_NODES}",
                    header.id, header.nodes
                );
                *assembly = RiftAssembly::default();
                return None;
            }
            assembly.values = vec![0; RIFT_NODES * RIFT_NODES];
            assembly.got = vec![false; RIFT_NODES];
            assembly.header = Some(header);
            None
        }
        ArpgRift::Heights(band) => {
            let header = assembly.header.as_ref()?;
            if band.id != header.id {
                return None;
            }
            let first = usize::from(band.first_row);
            let rows = usize::from(band.rows);
            if first + rows > RIFT_NODES || band.values.len() != rows * RIFT_NODES {
                return None;
            }
            assembly.values[first * RIFT_NODES..(first + rows) * RIFT_NODES]
                .copy_from_slice(&band.values);
            for got in &mut assembly.got[first..first + rows] {
                *got = true;
            }
            if !assembly.got.iter().all(|g| *g) {
                return None;
            }
            let header = assembly.header.take()?;
            let heights = assembly
                .values
                .iter()
                .map(|v| header.base + f32::from(*v) / 8.0)
                .collect();
            Some(RiftLayout {
                id: header.id,
                gx0: header.gx0,
                gy0: header.gy0,
                area: header.area,
                pieces: header.pieces,
                heights,
                theme: header.theme,
                marks: header.marks,
                sites: header
                    .sites
                    .into_iter()
                    .map(
                        |(kind, [x, y, z, o, radius])| benilla_assets::rift::RiftSite {
                            kind,
                            x,
                            y,
                            z,
                            o,
                            radius,
                        },
                    )
                    .collect(),
                flavour: header.flavour,
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use benilla_protocol::messages::arpg::ArpgRiftHeights;

    #[test]
    fn a_layout_is_whole_once_every_row_is_in() {
        let mut assembly = RiftAssembly::default();
        let header = ArpgRiftLayout {
            id: 9,
            gx0: 8,
            gy0: 11,
            nodes: RIFT_NODES as u16,
            area: 12,
            base: 60.0,
            pieces: vec![[48, 32, 0, 0]; 256],
            theme: 1,
            marks: vec![(1, -1.0, 2.0)],
            sites: Vec::new(),
            flavour: 0,
        };
        assert!(take_rift(&mut assembly, ArpgRift::Layout(header)).is_none());
        let mut whole = None;
        for first in (0..RIFT_NODES).step_by(32) {
            let rows = 32.min(RIFT_NODES - first);
            let band = ArpgRiftHeights {
                id: 9,
                first_row: first as u16,
                rows: rows as u16,
                values: vec![8; rows * RIFT_NODES],
            };
            whole = take_rift(&mut assembly, ArpgRift::Heights(band));
            assert_eq!(whole.is_some(), first + rows == RIFT_NODES);
        }
        let layout = whole.expect("whole");
        assert_eq!(
            (layout.id, layout.gx0, layout.gy0, layout.area),
            (9, 8, 11, 12)
        );
        assert_eq!(layout.height(128, 200), 61.0);
        assert_eq!(layout.theme, 1);
        // Another layout's band is not this one's.
        let stray = ArpgRiftHeights {
            id: 3,
            first_row: 0,
            rows: 1,
            values: vec![0; RIFT_NODES],
        };
        assert!(take_rift(&mut assembly, ArpgRift::Heights(stray)).is_none());
    }
}
