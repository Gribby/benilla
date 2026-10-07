//! The ARPG view: a fixed top-down camera, on only with `WOW_ARPG=1`. Not 1.12.1: the client
//! orbits behind the character, and this fork replaces that with one pose, so the switch is off
//! by default and stock behaviour is untouched.
//!
//! `WOW_ARPG_PITCH` (degrees looking down, default 55), `WOW_ARPG_YAW` (degrees, default 45) and
//! `WOW_ARPG_DIST` (yards, default 28) shape the pose.

use super::camera::CAM_PITCH_LIMIT;
use super::camera_zoom::CAM_DIST_MAX;
use super::*;

/// A fixed camera pose that [`camera::seat_camera`] applies each frame in place of the orbit the
/// drag, the key turn and the wheel would have set.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct ArpgPin {
    pub(super) yaw: f32,
    /// Negative looks down, as `LOGIN_PITCH` does.
    pub(super) pitch: f32,
    pub(super) distance: f32,
}

impl ArpgPin {
    /// The pose for a view `pitch_deg` degrees down, turned `yaw_deg`, `distance` yards out; the
    /// pitch stays inside the rig's own limit and the distance inside its zoom range.
    fn new(yaw_deg: f32, pitch_deg: f32, distance: f32) -> Self {
        Self {
            yaw: yaw_deg.to_radians(),
            pitch: (-pitch_deg.to_radians()).clamp(-CAM_PITCH_LIMIT, CAM_PITCH_LIMIT),
            distance: distance.clamp(5.0, CAM_DIST_MAX),
        }
    }

    /// The pose the environment asks for, `None` with `WOW_ARPG` unset, empty, `0` or `off`.
    fn from_env() -> Option<Self> {
        let on = std::env::var("WOW_ARPG").is_ok_and(|v| !matches!(v.trim(), "" | "0" | "off"));
        on.then(|| {
            Self::new(
                env_f32("WOW_ARPG_YAW", 45.0),
                env_f32("WOW_ARPG_PITCH", 55.0),
                env_f32("WOW_ARPG_DIST", 28.0),
            )
        })
    }
}

fn env_f32(name: &str, default: f32) -> f32 {
    std::env::var(name)
        .ok()
        .and_then(|v| v.trim().parse::<f32>().ok())
        .filter(|v| v.is_finite())
        .unwrap_or(default)
}

/// The pose to hold while the view is on.
#[derive(Resource)]
struct ArpgView(ArpgPin);

/// Hand the rig its pose; a no-op unless `WOW_ARPG` is on.
pub(super) fn plugin(app: &mut App) {
    let Some(pin) = ArpgPin::from_env() else {
        return;
    };
    app.insert_resource(ArpgView(pin)).add_systems(
        Update,
        pin_view
            .in_set(WorldStage::Input)
            .before(control)
            .in_set(crate::char_select::InWorldGated),
    );
}

/// Set the rig's pin once; `seat_camera` re-applies it each frame.
fn pin_view(view: Res<ArpgView>, mut rig: ResMut<CameraControl>) {
    if rig.arpg_pin != Some(view.0) {
        rig.arpg_pin = Some(view.0);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_default_pose_looks_down_and_turns_a_quarter_of_a_half_turn() {
        let pin = ArpgPin::new(45.0, 55.0, 28.0);
        assert!(pin.pitch < 0.0, "negative pitch looks down");
        assert!((pin.pitch + 55.0_f32.to_radians()).abs() < 1.0e-6);
        assert!((pin.yaw - std::f32::consts::FRAC_PI_4).abs() < 1.0e-6);
        assert_eq!(pin.distance, 28.0);
    }

    #[test]
    fn an_extreme_request_stays_inside_the_rig_limits() {
        let pin = ArpgPin::new(0.0, 170.0, 500.0);
        assert_eq!(pin.pitch, -CAM_PITCH_LIMIT);
        assert_eq!(pin.distance, CAM_DIST_MAX);
        let near = ArpgPin::new(0.0, 0.0, 0.0);
        assert_eq!(near.distance, 5.0);
    }
}
