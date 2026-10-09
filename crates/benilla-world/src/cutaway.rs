//! Fork-only, not 1.12.1: the ARPG view's cutaway. The ARPG client's camera looks down on the
//! player from high above, so indoors and in caves the roof and the upper walls hide everything.
//! While [`Cutaway::plane`] is set, static world geometry (WMO groups, static doodads, terrain)
//! above that height and within [`Cutaway::radius`] of the player, on the ground, is not drawn,
//! except floor-like surfaces off the camera's sightline to the player ([`Cutaway::spares`]),
//! and the WMO groups inside that cylinder skip portal culling, since the camera now sees rooms
//! through the cut roof that its portals would hide. Units and players are never cut.
//!
//! Outdoors the ARPG client sets a smaller [`Cutaway::dither`] cut instead, between the player and
//! the camera: the WMO walls, roofs and static doodads (a tree crown, an awning) there draw every
//! other pixel, so the player shows through them and they still read as there. A dither reveals
//! no rooms and leaves terrain whole.
//!
//! The ARPG client writes this each frame; it stays at its default (off) in stock benilla. The
//! cut reaches the shaders through the shared light buffer's free lanes (`global_light`): row 12
//! `.w` the radius (0 off, negative for a dither), row 19 `.z` the plane, row 19 `.w` and row 5 `.z` the centre's X and
//! Z, so no binding changes.

use bevy::prelude::*;

/// A surface whose normal points at least this far up (its normal's y) is floor-like, which the
/// indoor cut spares off the sightline ([`Cutaway::spares`]); `ARPG_FLOOR_UP` in the shaders.
pub const FLOOR_UP: f32 = 0.6;
/// The sightline the indoor cut always clears, from the camera to this far below the cut plane
/// (about the player's chest), and its radius, in yards; `ARPG_SIGHT_DROP`, `ARPG_SIGHT_RADIUS`.
pub const SIGHT_DROP: f32 = 1.5;
pub const SIGHT_RADIUS: f32 = 3.5;

/// The cutaway this frame; the default draws everything.
#[derive(Resource, Default, Clone, Copy, PartialEq, Debug)]
pub struct Cutaway {
    /// Static geometry above this world height (Bevy Y) is not drawn; `None` draws everything.
    pub plane: Option<f32>,
    /// The cut's centre, the player's feet: only geometry within [`Self::radius`] of it on the
    /// ground is cut, so distant hills, trees and buildings keep their tops.
    pub center: Vec3,
    /// The cut cylinder's radius, in yards.
    pub radius: f32,
    /// A see-through dither in place of the cut: the outdoor occluder fade.
    pub dither: bool,
}

impl Cutaway {
    /// Whether a WMO group with these world bounds skips portal culling this frame: it meets the
    /// cut cylinder on the ground and reaches up past the player's feet, so the camera sees into it
    /// through the cut. A room below the floor stays culled, as the floor still hides it.
    pub fn reveals(&self, min: Vec3, max: Vec3) -> bool {
        if self.plane.is_none() || self.dither || max.y < self.center.y - 1.0 {
            return false;
        }
        let nearest = Vec2::new(
            self.center.x.clamp(min.x, max.x),
            self.center.z.clamp(min.z, max.z),
        );
        nearest.distance_squared(Vec2::new(self.center.x, self.center.z))
            <= self.radius * self.radius
    }

    /// Whether a world point lies in the cut: above the plane and inside the cylinder, as the
    /// shaders test it.
    pub fn cuts(&self, point: Vec3) -> bool {
        self.plane.is_some_and(|y| {
            point.y > y
                && Vec2::new(point.x - self.center.x, point.z - self.center.z).length_squared()
                    < self.radius * self.radius
        })
    }

    /// Whether the indoor cut spares a point it would otherwise take, as the shaders'
    /// `arpg_cut_spares` does: a floor-like surface (its normal's `up` at least [`FLOOR_UP`]) off
    /// the sightline from `eye` (the camera) to the player stays, so a ramp or a ledge higher than
    /// the player is not holed; one that would hide the player still goes, as do walls and
    /// ceilings. A dither spares nothing this way.
    pub fn spares(&self, point: Vec3, up: f32, eye: Vec3) -> bool {
        let Some(plane) = self.plane else {
            return false;
        };
        if self.dither || up < FLOOR_UP {
            return false;
        }
        let p = Vec3::new(self.center.x, plane - SIGHT_DROP, self.center.z);
        let pc = eye - p;
        let t = ((point - p).dot(pc) / pc.length_squared().max(1e-4)).clamp(0.0, 1.0);
        (point - (p + pc * t)).length_squared() >= SIGHT_RADIUS * SIGHT_RADIUS
    }

    /// The four light-buffer lanes the shaders read: `[radius, plane, centre x, centre z]`, all
    /// zero (radius 0, off) without a plane. A dither sends the radius negated.
    pub fn lanes(&self) -> [f32; 4] {
        match self.plane {
            Some(plane) if self.radius > 0.0 => {
                let radius = if self.dither {
                    -self.radius
                } else {
                    self.radius
                };
                [radius, plane, self.center.x, self.center.z]
            }
            _ => [0.0; 4],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn on() -> Cutaway {
        Cutaway {
            plane: Some(10.0),
            center: Vec3::ZERO,
            radius: 20.0,
            dither: false,
        }
    }

    #[test]
    fn a_floor_off_the_sightline_is_spared_and_one_on_it_is_not() {
        let cut = on();
        // The camera high up off to +x; the player at the origin, chest at 10 - 1.5.
        let eye = Vec3::new(16.0, 23.0, 0.0);
        // A ledge 12 high behind the player, away from the camera: kept.
        assert!(cut.spares(Vec3::new(-8.0, 12.0, 0.0), 1.0, eye));
        // The same ledge between the camera and the player, on the sightline: cut.
        let on_line = Vec3::new(8.0, 8.5 + (23.0 - 8.5) * 0.5, 0.0);
        assert!(!cut.spares(on_line, 1.0, eye));
        // A wall or a ceiling there is never spared.
        assert!(!cut.spares(Vec3::new(-8.0, 12.0, 0.0), 0.0, eye));
        assert!(!cut.spares(Vec3::new(-8.0, 12.0, 0.0), -1.0, eye));
        // Nor does a dither spare anything this way.
        let dither = Cutaway {
            dither: true,
            ..on()
        };
        assert!(!dither.spares(Vec3::new(-8.0, 12.0, 0.0), 1.0, eye));
    }

    #[test]
    fn a_dither_sends_its_radius_negated_and_reveals_no_room() {
        let dither = Cutaway {
            dither: true,
            ..on()
        };
        assert_eq!(dither.lanes(), [-20.0, 10.0, 0.0, 0.0]);
        assert!(!dither.reveals(Vec3::new(-5.0, -1.0, -5.0), Vec3::new(5.0, 6.0, 5.0)));
        // The cylinder is the shaders' as for a cut (the pick stops at a dither regardless).
        assert!(dither.cuts(Vec3::new(5.0, 10.5, -5.0)));
    }

    #[test]
    fn off_by_default() {
        let off = Cutaway::default();
        assert!(!off.cuts(Vec3::new(0.0, 1.0e6, 0.0)));
        assert!(!off.reveals(Vec3::splat(-1.0), Vec3::splat(1.0)));
        assert_eq!(off.lanes(), [0.0; 4]);
    }

    #[test]
    fn cuts_above_the_plane_inside_the_cylinder_only() {
        let on = on();
        assert!(on.cuts(Vec3::new(5.0, 10.5, -5.0)));
        assert!(!on.cuts(Vec3::new(5.0, 9.5, -5.0)), "below the plane");
        assert!(!on.cuts(Vec3::new(25.0, 50.0, 0.0)), "outside the cylinder");
        assert_eq!(on.lanes(), [20.0, 10.0, 0.0, 0.0]);
    }

    #[test]
    fn reveals_rooms_inside_the_cylinder_at_or_above_the_feet() {
        let on = on();
        // The room the player stands in, and one beside it whose wall is 15 yd off.
        assert!(on.reveals(Vec3::new(-5.0, -1.0, -5.0), Vec3::new(5.0, 6.0, 5.0)));
        assert!(on.reveals(Vec3::new(15.0, 0.0, -5.0), Vec3::new(30.0, 6.0, 5.0)));
        // A room 25 yd off, and a cellar under the floor.
        assert!(!on.reveals(Vec3::new(25.0, 0.0, -5.0), Vec3::new(40.0, 6.0, 5.0)));
        assert!(!on.reveals(Vec3::new(-5.0, -8.0, -5.0), Vec3::new(5.0, -2.0, 5.0)));
    }
}
