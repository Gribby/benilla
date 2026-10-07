//! The ARPG view: a fixed top-down camera, on only with `WOW_ARPG=1`. Not 1.12.1: the client
//! orbits behind the character, and this fork replaces that with one pose, so the switch is off
//! by default and stock behaviour is untouched.
//!
//! `WOW_ARPG_PITCH` (degrees looking down, default 55), `WOW_ARPG_YAW` (degrees, default 45) and
//! `WOW_ARPG_DIST` (yards, default 28) shape the pose.
//!
//! WASD then walk relative to that camera, not the character ([`steer`]): W is up the screen, S
//! down, A left, D right, and the character turns to face the way it walks.
//!
//! The mouse never starts a mouse-look, so the cursor stays on screen (`camera::run_arpg_clicks`).
//! Holding the left button on the ground walks toward the cursor, and letting go walks on to the
//! last point; a left-click on a unit or object is the stock right-click interact (attack, talk,
//! loot, use). While a spell waits for a target or a ground point, a left-click is the stock
//! select click that places it. The right button keeps its stock click.
//!
//! Spells need no tab-targeting: the enemy under the cursor is the target
//! (`crate::target::arpg_soft`), a ground spell cast from a key lands at the cursor at once
//! (`crate::spell::targeting::quick_cast_location`; one clicked on the bar waits for a click).
//!
//! The character always faces the cursor: standing, it turns to the aim; walking, it faces within
//! 22.5° of the aim and walks with the stock forward, backpedal and strafe moves ([`steer`]).

use super::camera::CAM_PITCH_LIMIT;
use super::camera_zoom::CAM_DIST_MAX;
use super::input::MoveAxes;
use super::*;
use crate::bindings::{BindingsState, Input};

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
    // The enemy under the cursor is the target: no tab-targeting.
    crate::target::arpg_soft::plugin(app);
    app.insert_resource(ArpgView(pin))
        .add_systems(
            Update,
            pin_view
                .in_set(WorldStage::Input)
                .before(control)
                .in_set(crate::char_select::InWorldGated),
        )
        // The quick ground cast after the script calls, whose casts enter the targeting mode.
        .add_systems(
            Update,
            crate::spell::targeting::quick_cast_location
                .in_set(crate::target::TargetUpdate)
                .after(crate::script_calls::apply_script_calls),
        );
}

/// A walk to a point stops this close to it, in yards, flat.
const GOAL_STOP: f32 = 0.75;
/// While the button is held, a cursor this close to the character walks nowhere, so a cursor
/// resting on the character does not spin it in place.
const HOLD_DEADZONE: f32 = 1.0;
/// A released walk that closes on its point slower than this, in yards per second, is stuck.
const STALL_RATE: f32 = 0.5;
/// How long a released walk may stay stuck before it gives up, in seconds.
const STALL_LIMIT: f32 = 0.5;

/// The ARPG view's mouse state, on [`CameraControl`] so the look session and the controller share
/// it. The pick fields are last frame's, the frame `crate::target`'s press latch also reads.
#[derive(Clone, Copy, Debug)]
pub(super) struct ArpgMouse {
    /// The world point under the cursor, if the cursor ray hits anything.
    pub(super) ground: Option<Vec3>,
    /// The cursor is over a unit, a corpse or a GameObject: a left press there interacts.
    pub(super) over_target: bool,
    /// A spell is waiting for a target or a point: a left press is the stock select click.
    pub(super) spell_targeting: bool,
    /// The left press in flight settles as the stock select click, not the interact.
    pub(super) left_selects: bool,
    /// The left button went down on the ground and is still held.
    pub(super) holding: bool,
    /// Where the character is walking to.
    pub(super) goal: Option<Vec3>,
    /// Where the cursor aims: the hovered unit's feet, else the world point under the cursor;
    /// `None` over a UI panel or the sky. The character always faces it.
    pub(super) aim: Option<Vec3>,
    /// The flat distance to `goal` last frame, for the stall test.
    last_dist: f32,
    /// Seconds the released walk has been stuck.
    stall: f32,
}

impl Default for ArpgMouse {
    fn default() -> Self {
        Self {
            ground: None,
            over_target: false,
            spell_targeting: false,
            left_selects: false,
            holding: false,
            goal: None,
            aim: None,
            last_dist: f32::INFINITY,
            stall: 0.0,
        }
    }
}

impl ArpgMouse {
    /// A left press on the ground: walk to the cursor, a fresh walk.
    pub(super) fn press_ground(&mut self) {
        self.holding = true;
        self.goal = self.ground;
        self.reset_stall();
    }

    /// A left press on a unit or object, a spell-targeting click, or a right press: the click owns
    /// any walk from here (an interact's Click-to-Move approach).
    pub(super) fn press_target(&mut self) {
        self.holding = false;
        self.goal = None;
        self.reset_stall();
    }

    fn reset_stall(&mut self) {
        self.last_dist = f32::INFINITY;
        self.stall = 0.0;
    }

    /// The facing that walks from `pos` to the goal this frame, `None` when there is no goal, it
    /// is reached, or a released walk is stuck; a reached or stuck released walk ends.
    fn walk_toward_goal(&mut self, pos: Vec3, dt: f32) -> Option<f32> {
        let goal = self.goal?;
        let (dx, dz) = (goal.x - pos.x, goal.z - pos.z);
        let dist = (dx * dx + dz * dz).sqrt();
        if self.holding {
            self.reset_stall();
            return (dist >= HOLD_DEADZONE).then(|| f32::atan2(-dx, -dz));
        }
        let closing = if dt > 0.0 {
            (self.last_dist - dist) / dt
        } else {
            f32::INFINITY
        };
        self.stall = if closing < STALL_RATE {
            self.stall + dt
        } else {
            0.0
        };
        self.last_dist = dist;
        if dist < GOAL_STOP || self.stall > STALL_LIMIT {
            self.goal = None;
            self.reset_stall();
            return None;
        }
        Some(f32::atan2(-dx, -dz))
    }
}

/// Set the rig's pin and refresh the cursor's pick for the mouse. A walk to a point ends any
/// Click-to-Move approach or `/follow`, as a movement key does.
fn pin_view(
    view: Res<ArpgView>,
    hovered: Option<Res<crate::target::Hovered>>,
    object: Option<Res<crate::target::HoveredObject>>,
    occlusion: Option<Res<crate::target::PickOcclusion>>,
    spell_targeting: Option<Res<crate::spell::targeting::SpellTargeting>>,
    over_panel: Option<Res<crate::ui_script::PointerOverUiPanel>>,
    transforms: Query<&Transform>,
    mut approach: ResMut<approach::Approach>,
    mut follow: ResMut<FollowState>,
    mut rig: ResMut<CameraControl>,
) {
    if rig.arpg_pin != Some(view.0) {
        rig.arpg_pin = Some(view.0);
    }
    // The negation of the ground leg in `crate::target::click::act_on_right_click`.
    let over_target = hovered
        .as_ref()
        .is_some_and(|h| h.any().is_some() || h.refused)
        || object.as_ref().is_some_and(|o| o.target.is_some());
    let ground = occlusion.as_ref().and_then(|o| o.point);
    // Aim at a hovered unit's feet: the world point behind a standing model lies past it.
    let hovered_feet = hovered
        .as_ref()
        .and_then(|h| h.target)
        .and_then(|e| transforms.get(e).ok())
        .map(|t| t.translation);
    let on_panel = over_panel.as_ref().is_some_and(|p| p.0);
    let aim = if on_panel {
        None
    } else {
        hovered_feet.or(ground)
    };
    let mouse = &mut rig.arpg;
    mouse.over_target = over_target;
    mouse.ground = ground;
    mouse.spell_targeting = spell_targeting.as_ref().is_some_and(|t| t.active());
    mouse.aim = aim;
    if mouse.holding || mouse.goal.is_some() {
        if approach.active() {
            approach.stop();
        }
        if follow.guid.is_some() {
            follow.stop();
        }
    }
}

/// The facing that walks `up` (+1 toward the top of the screen, -1 toward the bottom) and `right`
/// (+1 right, -1 left) from a camera at `cam_yaw`, `None` for no direction. A yaw grows leftward
/// and faces `(-sin, -cos)` on the ground, as the camera's and the character's both do.
fn screen_facing(cam_yaw: f32, up: i32, right: i32) -> Option<f32> {
    let (sin, cos) = cam_yaw.sin_cos();
    let (up, right) = (up as f32, right as f32);
    let x = -sin * up + cos * right;
    let z = -cos * up - sin * right;
    (x != 0.0 || z != 0.0).then(|| f32::atan2(-x, -z))
}

/// The aim is turned to only past this far from the feet, in yards: closer, a pixel of mouse
/// travel swings the bearing wildly.
const AIM_MIN: f32 = 0.5;
/// A standing character re-faces the aim only past this, in radians (about 1°): each facing change
/// is a packet, and float noise in the pick must not send one a frame.
const AIM_DEADZONE: f32 = 0.02;

/// The yaw that faces from `pos` to `at` on the ground, `None` with `at` within [`AIM_MIN`].
fn yaw_toward(pos: Vec3, at: Vec3) -> Option<f32> {
    let (dx, dz) = (at.x - pos.x, at.z - pos.z);
    ((dx * dx + dz * dz).sqrt() >= AIM_MIN).then(|| f32::atan2(-dx, -dz))
}

/// Split a walk along `move_yaw` into the stock axes around a facing near `aim_yaw`. WoW walks in
/// eight directions relative to the facing (forward, back, the strafes and the four diagonals), so
/// the facing snaps to the one that makes the walk go exactly `move_yaw`, which leaves the
/// character within 22.5° of the aim. Returns `(facing, fwd, side)`, side positive right; a yaw
/// grows leftward, so a walk a quarter turn left of the facing is a strafe left.
fn aim_and_walk(move_yaw: f32, aim_yaw: f32) -> (f32, i32, i32) {
    use std::f32::consts::FRAC_PI_4;
    let octant = (wrap_pi(move_yaw - aim_yaw) / FRAC_PI_4).round() as i32;
    let facing = wrap_pi(move_yaw - octant as f32 * FRAC_PI_4);
    let (fwd, side) = match octant {
        0 => (1, 0),
        1 => (1, -1),
        -1 => (1, 1),
        2 => (0, -1),
        -2 => (0, 1),
        3 => (-1, -1),
        -3 => (-1, 1),
        _ => (-1, 0),
    };
    (facing, fwd, side)
}

/// Turn the frame's movement keys and the mouse walk into a walk relative to the camera, with the
/// character always facing the cursor's aim. A walk is split into the stock forward, back and
/// strafe axes around that facing ([`aim_and_walk`]), so the animations, the speeds (backpedal is
/// slower, as in WoW) and the movement packets are the stock ones; the keys' own strafe and turn
/// are dropped, as A and D are directions here. A key beats the mouse and ends its walk. With
/// neither, autorun, `/follow` and a Click-to-Move approach keep their own facing; otherwise a
/// standing character turns to the aim. A character that cannot turn (stunned, dead) keeps its
/// facing.
pub(super) fn steer(
    axes: MoveAxes,
    binds: &BindingsState,
    face_yaw: &mut f32,
    pos: Vec3,
    cam_yaw: f32,
    mouse: &mut ArpgMouse,
    may_turn: bool,
    // On our feet: a seated body is not turned to the aim, as the facing commit refuses one.
    standing: bool,
    dt: f32,
) -> MoveAxes {
    let key = |input: Input| i32::from(binds.pressed(input));
    let up = key(Input::MoveForward) - key(Input::MoveBackward);
    let right = (key(Input::StrafeRight) + key(Input::TurnRight)
        - key(Input::StrafeLeft)
        - key(Input::TurnLeft))
    .signum();
    let key_facing = screen_facing(cam_yaw, up, right);
    if key_facing.is_some() {
        mouse.goal = None;
    } else if mouse.holding {
        // The held walk follows the cursor; off the world (the sky) it keeps the last point.
        if let Some(at) = mouse.ground {
            mouse.goal = Some(at);
        }
    }
    // A character that cannot turn does not walk the mouse's way along a stale facing.
    let mouse_facing = || {
        if may_turn {
            mouse.walk_toward_goal(pos, dt)
        } else {
            None
        }
    };
    let move_yaw = key_facing.or_else(mouse_facing);
    let aim_yaw = mouse.aim.and_then(|at| yaw_toward(pos, at));
    let (mut fwd, mut side) = (axes.fwd, 0);
    match (move_yaw, aim_yaw) {
        // Walking and aiming: face the aim, walk the way the player asked.
        (Some(walk), Some(aim)) if may_turn => {
            let (facing, f, s) = aim_and_walk(walk, aim);
            *face_yaw = facing;
            (fwd, side) = (f, s);
        }
        // Walking with no aim (the cursor on a panel or the sky): face the walk.
        (Some(walk), _) => {
            if may_turn {
                *face_yaw = walk;
            }
            fwd = 1;
        }
        // Standing with nothing else steering: face the aim.
        (None, Some(aim))
            if axes.fwd == 0
                && may_turn
                && standing
                && wrap_pi(aim - *face_yaw).abs() > AIM_DEADZONE =>
        {
            *face_yaw = aim;
        }
        _ => {}
    }
    MoveAxes {
        fwd,
        side,
        mouselook: false,
        turning: false,
        translating: fwd != 0 || side != 0,
        strafe_left: side < 0,
        strafe_right: side > 0,
        turn_left: false,
        turn_right: false,
        ..axes
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The angle between two yaws, whatever their wrap.
    fn gap(a: f32, b: f32) -> f32 {
        wrap_pi(a - b).abs()
    }

    #[test]
    fn up_the_screen_faces_the_way_the_camera_looks() {
        for cam in [0.0, 0.7, -2.0, 3.0] {
            let yaw = screen_facing(cam, 1, 0).unwrap();
            assert!(gap(yaw, cam) < 1.0e-5, "cam {cam} gave {yaw}");
        }
    }

    #[test]
    fn right_turns_a_quarter_clockwise_and_down_turns_about() {
        let cam = 0.7;
        let right = screen_facing(cam, 0, 1).unwrap();
        assert!(gap(right, cam - std::f32::consts::FRAC_PI_2) < 1.0e-5);
        let left = screen_facing(cam, 0, -1).unwrap();
        assert!(gap(left, cam + std::f32::consts::FRAC_PI_2) < 1.0e-5);
        let down = screen_facing(cam, -1, 0).unwrap();
        assert!(gap(down, cam + std::f32::consts::PI) < 1.0e-5);
    }

    #[test]
    fn a_released_walk_faces_its_point_and_ends_on_arrival() {
        let mut mouse = ArpgMouse {
            ground: Some(Vec3::new(0.0, 0.0, -10.0)),
            ..default()
        };
        mouse.press_ground();
        mouse.holding = false;
        // Due -Z from the origin is yaw 0.
        let yaw = mouse.walk_toward_goal(Vec3::ZERO, 0.1).unwrap();
        assert!(gap(yaw, 0.0) < 1.0e-5);
        assert_eq!(mouse.walk_toward_goal(Vec3::new(0.0, 0.0, -9.5), 0.1), None);
        assert_eq!(mouse.goal, None);
    }

    #[test]
    fn the_aim_bearing_ignores_height_and_a_cursor_underfoot() {
        let yaw = yaw_toward(Vec3::ZERO, Vec3::new(-10.0, 5.0, 0.0)).unwrap();
        // Due -X is a quarter turn left of -Z.
        assert!(gap(yaw, std::f32::consts::FRAC_PI_2) < 1.0e-5);
        assert_eq!(yaw_toward(Vec3::ZERO, Vec3::new(0.1, 3.0, 0.0)), None);
    }

    /// The ground direction a facing and the stock axes walk, as `control` builds it.
    fn walked(facing: f32, fwd: i32, side: i32) -> f32 {
        let rot = Quat::from_rotation_y(facing);
        let dir = rot * Vec3::NEG_Z * fwd as f32 + rot * Vec3::X * side as f32;
        f32::atan2(-dir.x, -dir.z)
    }

    #[test]
    fn a_walk_goes_exactly_its_way_while_the_facing_stays_near_the_aim() {
        use std::f32::consts::FRAC_PI_8;
        for walk in [0.0, 0.5, 1.6, -2.2, 3.1] {
            for aim in [0.0, 0.9, -1.4, 2.8, -3.0] {
                let (facing, fwd, side) = aim_and_walk(walk, aim);
                assert!(
                    gap(walked(facing, fwd, side), walk) < 1.0e-4,
                    "walk {walk} aim {aim}"
                );
                assert!(
                    gap(facing, aim) <= FRAC_PI_8 + 1.0e-4,
                    "walk {walk} aim {aim}"
                );
            }
        }
    }

    #[test]
    fn walking_toward_the_aim_is_forward_and_away_is_backpedal() {
        let (facing, fwd, side) = aim_and_walk(0.4, 0.4);
        assert!(gap(facing, 0.4) < 1.0e-6);
        assert_eq!((fwd, side), (1, 0));
        let (_, fwd, side) = aim_and_walk(0.4 + std::f32::consts::PI, 0.4);
        assert_eq!((fwd, side), (-1, 0));
        // A quarter turn left of the aim is a strafe left.
        let (_, fwd, side) = aim_and_walk(0.4 + std::f32::consts::FRAC_PI_2, 0.4);
        assert_eq!((fwd, side), (0, -1));
    }

    #[test]
    fn a_held_walk_rests_inside_the_deadzone_and_keeps_its_goal() {
        let mut mouse = ArpgMouse {
            ground: Some(Vec3::new(0.5, 0.0, 0.0)),
            ..default()
        };
        mouse.press_ground();
        assert_eq!(mouse.walk_toward_goal(Vec3::ZERO, 0.1), None);
        assert!(mouse.goal.is_some());
    }

    #[test]
    fn a_released_walk_into_a_wall_gives_up() {
        let mut mouse = ArpgMouse {
            ground: Some(Vec3::new(20.0, 0.0, 0.0)),
            ..default()
        };
        mouse.press_ground();
        mouse.holding = false;
        let mut frames = 0;
        while mouse.walk_toward_goal(Vec3::ZERO, 0.1).is_some() {
            frames += 1;
            assert!(frames < 20, "a stuck walk never gave up");
        }
        assert_eq!(mouse.goal, None);
    }

    #[test]
    fn a_diagonal_splits_the_difference_and_no_key_gives_none() {
        let cam = -0.4;
        let ne = screen_facing(cam, 1, 1).unwrap();
        assert!(gap(ne, cam - std::f32::consts::FRAC_PI_4) < 1.0e-5);
        assert_eq!(screen_facing(cam, 0, 0), None);
    }

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
