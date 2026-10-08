//! Fork-only, not 1.12.1: the ARPG view's cutaway. The ARPG client's camera looks down on the
//! player from high above, so indoors and in caves the roof and the upper walls hide everything.
//! While [`Cutaway::plane`] is set, static world geometry (WMO groups, static doodads, terrain)
//! above that height and within [`Cutaway::radius`] of the player, on the ground, is not drawn,
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
