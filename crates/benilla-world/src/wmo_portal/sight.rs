//! Fork-only, not 1.12.1: the ARPG view's rooms in sight. Under the indoor cutaway the camera hangs
//! over a cut roof, so the stock flood (its portal side tests and screen windows the camera's)
//! cannot say which rooms show, and drawing every room the cut cylinder meets showed the rooms
//! beyond the walls. Indoors, the rooms drawn are the ones the player sees instead: the room it
//! stands in, every room through one of its doorways, and a room through a doorway of those that
//! the player's eye sees through the first doorway. Rooms joined only by a wall stay dark, as do
//! the building's outer shell and roof.

use benilla_assets::WmoModel;

/// The rooms of `model` drawn for a player standing in `group` with its eye at `eye`, both in
/// model space (WoW axes): `true` per group. Two doorways deep.
pub fn sight_rooms(model: &WmoModel, group: usize, eye: [f32; 3]) -> Vec<bool> {
    let nav = &model.group_nav;
    let mut visible = vec![false; nav.len()];
    let Some(home) = nav.get(group) else {
        return visible;
    };
    visible[group] = true;
    // The first doorways: every portal of the player's room.
    let mut first: Vec<(usize, usize)> = Vec::new(); // (portal, room behind it)
    for r in refs(model, home) {
        let next = usize::from(r.group);
        if next < visible.len() {
            visible[next] = true;
            first.push((usize::from(r.portal), next));
        }
    }
    // The second: a portal of a room behind one, seen from the eye through that first doorway.
    for &(door, room) in &first {
        let Some(behind) = nav.get(room) else {
            continue;
        };
        for r in refs(model, behind) {
            let next = usize::from(r.group);
            if next >= visible.len() || visible[next] || next == group {
                continue;
            }
            let Some(target) = portal_centre(model, usize::from(r.portal)) else {
                continue;
            };
            if through_portal(model, door, eye, target) {
                visible[next] = true;
            }
        }
    }
    visible
}

fn refs<'a>(
    model: &'a WmoModel,
    nav: &benilla_assets::WmoGroupNav,
) -> impl Iterator<Item = &'a benilla_assets::WmoPortalRef> + 'a {
    let start = usize::from(nav.ref_start);
    let end = (start + usize::from(nav.ref_count)).min(model.portal_refs.len());
    model.portal_refs[start.min(end)..end].iter()
}

fn portal_points(model: &WmoModel, portal: usize) -> Option<&[[f32; 3]]> {
    let info = model.portal_infos.get(portal)?;
    let start = usize::from(info.start_vertex);
    let end = start + usize::from(info.count);
    (info.count >= 3 && end <= model.portal_vertices.len())
        .then(|| &model.portal_vertices[start..end])
}

fn portal_centre(model: &WmoModel, portal: usize) -> Option<[f32; 3]> {
    let points = portal_points(model, portal)?;
    let n = points.len() as f32;
    let sum = points
        .iter()
        .fold([0.0f32; 3], |a, p| [a[0] + p[0], a[1] + p[1], a[2] + p[2]]);
    Some([sum[0] / n, sum[1] / n, sum[2] / n])
}

fn sub(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

fn dot(a: [f32; 3], b: [f32; 3]) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

fn cross(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

/// Whether the segment from `eye` to `target` passes through portal `portal`'s polygon (a convex
/// one, as WMO portals are). A margin of a tenth of a yard round its edges forgives a doorframe.
fn through_portal(model: &WmoModel, portal: usize, eye: [f32; 3], target: [f32; 3]) -> bool {
    let Some(points) = portal_points(model, portal) else {
        return false;
    };
    let v0 = points[0];
    let mut normal = [0.0f32; 3];
    for w in points.windows(3) {
        let n = cross(sub(w[1], w[0]), sub(w[2], w[0]));
        if dot(n, n) > dot(normal, normal) {
            normal = n;
        }
    }
    if dot(normal, normal) < 1e-8 {
        return false;
    }
    let dir = sub(target, eye);
    let denom = dot(normal, dir);
    if denom.abs() < 1e-6 {
        return false;
    }
    let t = dot(normal, sub(v0, eye)) / denom;
    if !(0.0..=1.0).contains(&t) {
        return false;
    }
    let hit = [
        eye[0] + dir[0] * t,
        eye[1] + dir[1] * t,
        eye[2] + dir[2] * t,
    ];
    // Inside the convex polygon: the hit on the same side of every edge (with a margin).
    let len = dot(normal, normal).sqrt();
    let mut sign = 0.0f32;
    for i in 0..points.len() {
        let a = points[i];
        let b = points[(i + 1) % points.len()];
        let edge = sub(b, a);
        let side = dot(cross(edge, sub(hit, a)), normal) / len;
        let edge_len = dot(edge, edge).sqrt().max(1e-6);
        let distance = side / edge_len;
        if distance.abs() <= 0.1 {
            continue;
        }
        if sign == 0.0 {
            sign = distance.signum();
        } else if distance.signum() != sign {
            return false;
        }
    }
    true
}

#[cfg(test)]
mod tests {
    use super::*;
    use benilla_assets::{WmoGroupNav, WmoPortalInfo, WmoPortalRef};

    fn nav(ref_start: u16, ref_count: u16) -> WmoGroupNav {
        WmoGroupNav {
            flags: 0x2000,
            bbox_min: [0.0; 3],
            bbox_max: [0.0; 3],
            ref_start,
            ref_count,
            area_table_id: 0,
            fog_indices: [0; 4],
            group_liquid: benilla_formats::NO_GROUP_LIQUID,
        }
    }

    /// Four rooms in a row along x, 10 yd each, joined by doorways 2 yd wide and 3 high at
    /// x = 10, 20, 30, plus room 4 beside room 0 with no doorway (a wall between them).
    fn corridor() -> WmoModel {
        let door = |x: f32| vec![[x, -1.0, 0.0], [x, 1.0, 0.0], [x, 1.0, 3.0], [x, -1.0, 3.0]];
        let mut portal_vertices = Vec::new();
        let mut portal_infos = Vec::new();
        for (i, x) in [10.0f32, 20.0, 30.0].into_iter().enumerate() {
            portal_vertices.extend(door(x));
            portal_infos.push(WmoPortalInfo {
                start_vertex: (i * 4) as u16,
                count: 4,
                plane: [1.0, 0.0, 0.0, -x],
            });
        }
        let pref = |portal: u16, group: u16| WmoPortalRef {
            portal,
            group,
            side: 1,
        };
        WmoModel {
            portal_vertices,
            portal_infos,
            // Room 0: door 0 to room 1. Room 1: door 0 back, door 1 on. Room 2: door 1, door 2.
            // Room 3: door 2. Room 4: none.
            portal_refs: vec![
                pref(0, 1),
                pref(0, 0),
                pref(1, 2),
                pref(1, 1),
                pref(2, 3),
                pref(2, 2),
            ],
            group_nav: vec![nav(0, 1), nav(1, 2), nav(3, 2), nav(5, 1), nav(6, 0)],
            ..Default::default()
        }
    }

    #[test]
    fn the_room_and_the_rooms_through_its_doorways_show_and_a_walled_one_does_not() {
        let model = corridor();
        // Standing in room 0 in line with the doorways: rooms 0, 1, and 2 through door 0.
        let seen = sight_rooms(&model, 0, [5.0, 0.0, 1.5]);
        assert_eq!(seen, vec![true, true, true, false, false]);
    }

    #[test]
    fn a_second_doorway_out_of_the_eyes_line_stays_dark() {
        let model = corridor();
        // Standing in room 0 well off the line: door 1 is not seen through door 0.
        let seen = sight_rooms(&model, 0, [5.0, 30.0, 1.5]);
        assert_eq!(seen, vec![true, true, false, false, false]);
    }

    #[test]
    fn a_room_with_no_doorway_shows_only_itself() {
        let model = corridor();
        assert_eq!(
            sight_rooms(&model, 4, [0.0, 0.0, 1.0]),
            vec![false, false, false, false, true]
        );
    }
}
