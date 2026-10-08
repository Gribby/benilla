//! The ARPG view: a fixed top-down camera, on only with `WOW_ARPG=1`. Not 1.12.1: the client
//! orbits behind the character, and this fork replaces that with one pose, so the switch is off
//! by default and stock behaviour is untouched.
//!
//! `WOW_ARPG_PITCH` (degrees looking down, default 55), `WOW_ARPG_YAW` (degrees, default 45) and
//! `WOW_ARPG_DIST` (yards, default 28) shape the pose. Indoors and in caves the camera does not
//! collide; walls and ceilings above `WOW_ARPG_CUT` yards over the feet (default 2.8) are cut away
//! instead (`benilla_world::cutaway`). Outdoors, roofs, awnings and tree crowns between the player
//! and the camera draw see-through instead (a dither; `WOW_ARPG_XRAY=0` turns it off).
//!
//! A unit the player hits flashes white (`fx`), and the character turns to a new facing quickly
//! rather than snapping ([`TURN_RATE`]).
//!
//! WASD then walk relative to that camera, not the character ([`steer`]): W is up the screen, S
//! down, A left, D right, and the character turns to face the way it walks.
//!
//! The mouse never starts a mouse-look, so the cursor stays on screen (`camera::run_arpg_clicks`).
//! Holding the left button on the ground walks toward the cursor, and letting go walks on to the
//! last point. Holding either button on an enemy, or the left with Shift anywhere, swings: the
//! character walks to a clicked enemy until in reach, then swings at the cursor on its weapon
//! timer, and the server strikes whoever stands in the arc or whiffs. A click on a friendly unit,
//! a corpse or an object is the stock interact (talk, loot, use). While a spell waits for a target
//! or a ground point, a left-click is the stock select click that places it.
//!
//! There is no selection: a unit-word spell goes to the server at the cursor's aim point
//! (`crate::spell::ArpgCastAim`), which picks the unit, and a ground spell cast from a key lands at
//! the cursor at once (`crate::spell::targeting::quick_cast_location`; one clicked on the bar waits
//! for a click). These need a server with the ARPG patch, greeted at every world entry with
//! `ClientCommand::ArpgHello` ([`greet_server`]).
//!
//! The character always faces the cursor: standing, it turns to the aim; walking, it faces within
//! 22.5° of the aim and walks with the stock forward, backpedal and strafe moves ([`steer`]).

mod fx;

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

/// The pose to hold while the view is on, the cutaway's height above the feet and radius, and
/// whether outdoor occluders dither (`WOW_ARPG_XRAY`).
#[derive(Resource)]
struct ArpgView(ArpgPin, f32, f32, bool);

/// Hand the rig its pose; a no-op unless `WOW_ARPG` is on.
pub(super) fn plugin(app: &mut App) {
    let Some(pin) = ArpgPin::from_env() else {
        // The stock client shows no ARPG hover bar: drop the addon an ARPG session installed.
        app.add_systems(Startup, remove_hud);
        return;
    };
    // The enemy under the cursor, for the swing and the hover bar; no selection.
    crate::target::arpg_soft::plugin(app);
    // The struck unit's white flash.
    fx::plugin(app);
    let cut = env_f32("WOW_ARPG_CUT", CUT_HEIGHT).clamp(1.5, 10.0);
    let cut_radius = env_f32("WOW_ARPG_CUT_RADIUS", CUT_RADIUS).clamp(5.0, 120.0);
    app.add_systems(Startup, install_hud);
    // `WOW_ARPG_XRAY=0` turns the outdoor see-through off.
    let xray = std::env::var("WOW_ARPG_XRAY").map_or(true, |v| v.trim() != "0");
    app.insert_resource(ArpgView(pin, cut, cut_radius, xray))
        .init_resource::<crate::spell::ArpgCastAim>()
        .add_systems(
            Update,
            pin_view
                .in_set(WorldStage::Input)
                .before(control)
                .in_set(crate::char_select::InWorldGated),
        )
        // Ungated: the world entry's message must be read the frame it is written.
        .add_systems(Update, greet_server.before(pin_view))
        // The quick ground cast after the script calls, whose casts enter the targeting mode.
        .add_systems(
            Update,
            crate::spell::targeting::quick_cast_location
                .in_set(crate::target::TargetUpdate)
                .after(crate::script_calls::apply_script_calls),
        );
}

/// The hover health bar addon, written into `AddOns/ArpgHud` in the config folder.
const HUD_TOC: &str = "## Interface: 11200\n## Title: ARPG Hud\n## Notes: The benilla ARPG \
client's hover health bar, installed by the client.\nArpgHud.lua\n";
const HUD_LUA: &str = include_str!("arpg_hud.lua");

/// Where the hover health bar addon lives, `AddOns/ArpgHud` in the config folder.
fn hud_dir() -> Option<std::path::PathBuf> {
    crate::config_dir().map(|d| d.join("AddOns").join("ArpgHud"))
}

/// Remove the hover health bar addon, if an ARPG session installed one, so the stock client does
/// not run it. Only the two files the client writes go, and the folder if that empties it.
fn remove_hud() {
    let Some(dir) = hud_dir() else {
        return;
    };
    if !dir.is_dir() {
        return;
    }
    for name in ["ArpgHud.toc", "ArpgHud.lua"] {
        let _ = std::fs::remove_file(dir.join(name));
    }
    let _ = std::fs::remove_dir(&dir);
    info!(
        "arpg: removed the hover health bar addon from {}",
        dir.display()
    );
}

/// Install (or refresh) the hover health bar addon, so the next UI load runs it. A failure only
/// costs the bar.
fn install_hud() {
    let Some(dir) = hud_dir() else {
        return;
    };
    let write = |name: &str, body: &str| -> std::io::Result<()> {
        let path = dir.join(name);
        if std::fs::read_to_string(&path).is_ok_and(|old| old == body) {
            return Ok(());
        }
        std::fs::write(path, body)
    };
    let result = std::fs::create_dir_all(&dir)
        .and_then(|_| write("ArpgHud.toc", HUD_TOC))
        .and_then(|_| write("ArpgHud.lua", HUD_LUA));
    match result {
        Ok(()) => info!("arpg: the hover health bar addon is in {}", dir.display()),
        Err(e) => warn!(
            "arpg: could not install the hover health bar in {}: {e}",
            dir.display()
        ),
    }
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
/// A swing at a clicked enemy walks until it is this far inside the server's melee reach, in
/// yards, so a step of lag or the enemy's own step does not leave the swing short.
const SWING_MARGIN: f32 = 1.0;
/// Once swinging, the enemy may step this much further before the character walks after it
/// again, in yards, so a gap that wavers at the edge does not toggle the swing every frame.
const SWING_SLACK: f32 = 1.5;
/// Our own combat reach when the store has none, in yards (the client's default, as the server's).
const DEFAULT_REACH: f32 = 1.5;
/// With the cursor on a panel or the sky, a cast aims this far ahead of the facing, in yards.
const BLIND_AIM: f32 = 10.0;
/// Indoors, static geometry this far above the feet is cut away, in yards: above any player's
/// head (`WOW_ARPG_CUT` overrides).
const CUT_HEIGHT: f32 = 2.8;
/// The cutaway reaches this far from the feet on the ground, in yards (`WOW_ARPG_CUT_RADIUS`
/// overrides): past the camera's own footprint (28 yd out at 55° is 16 yd across), so nothing
/// between it and the player stands, while distant hills, trees and roofs keep their tops.
const CUT_RADIUS: f32 = 30.0;
/// Outdoors, the see-through dither ([`benilla_world::cutaway::Cutaway::dither`]): a disc this far
/// toward the camera from the feet, this wide, over this height above them, in yards. It covers
/// the slope between the player and the camera, where a roof, an awning or a tree crown hides the
/// player, and stops short of the player's own ground.
const XRAY_AHEAD: f32 = 4.0;
const XRAY_RADIUS: f32 = 6.5;
const XRAY_HEIGHT: f32 = 1.8;
/// The WMO group flags (`MOGP`) that make a room outdoors, as `wmo_portal::indoors_at` reads them:
/// `EXTERIOR` and `EXTERIOR_LIT`, a city's open streets and valleys (Stormwind, Orgrimmar).
const GROUP_OUTDOOR: u32 = 0x8 | 0x40;

/// Whether `room` is an indoor room or a cave, by its group's flags.
fn room_is_indoor(
    room: benilla_world::wmo_portal::WmoRoom,
    instances: &Query<&benilla_world::wmo_portal::WmoPortalInstance>,
    wmos: &Assets<benilla_assets::WmoModel>,
) -> bool {
    instances
        .get(room.instance)
        .ok()
        .and_then(|inst| wmos.get(&inst.handle))
        .and_then(|wmo| wmo.group_nav.get(usize::from(room.group)))
        .is_some_and(|nav| nav.flags & GROUP_OUTDOOR == 0)
}

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
    /// The live enemy under the cursor and its guid, last frame's (`crate::target::arpg_soft`).
    pub(super) over_enemy: Option<(Entity, u64)>,
    /// Shift is held: a left press swings in place wherever the cursor is.
    pub(super) force_attack: bool,
    /// A swing is held on the left and the right button.
    pub(super) swing_left: bool,
    pub(super) swing_right: bool,
    /// The enemy the swing was pressed on and its guid, which the character walks to until in
    /// reach; dropped once it dies or streams out, and the swing goes on in place.
    pub(super) swing_target: Option<(Entity, u64)>,
    /// That enemy's feet, last frame's; `None` for a swing in place.
    pub(super) swing_target_at: Option<Vec3>,
    /// The flat distance at which the server can strike it: the stock melee reach, both bodies'
    /// combat reach and 4/3 yd, at least 5 (`ObjectStore::unit_combat_reach`), less a margin.
    pub(super) swing_reach: f32,
    /// The swing the server should be running: held, and at a clicked enemy only once in reach.
    pub(super) swing_wanted: bool,
    /// The swing the server was last told of: `Some(intended)` running, `None` stopped.
    pub(super) swing_sent: Option<u64>,
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
            over_enemy: None,
            force_attack: false,
            swing_left: false,
            swing_right: false,
            swing_target: None,
            swing_target_at: None,
            swing_reach: 5.0 - SWING_MARGIN,
            swing_wanted: false,
            swing_sent: None,
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

    /// A swing press, on `enemy` (walked to) or in place: the swing owns the body, so any walk
    /// ends.
    pub(super) fn press_swing(&mut self, enemy: Option<(Entity, u64)>) {
        self.holding = false;
        self.goal = None;
        self.swing_target = enemy;
        self.reset_stall();
    }

    /// A swing is held on either button.
    pub(super) fn swing_held(&self) -> bool {
        self.swing_left || self.swing_right
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

/// The pick this frame's mouse reads, all last frame's: one parameter under Bevy's limit.
type ArpgPicks<'w, 's> = (
    Option<Res<'w, crate::target::Hovered>>,
    Option<Res<'w, crate::target::HoveredObject>>,
    Option<Res<'w, crate::target::PickOcclusion>>,
    Option<Res<'w, crate::target::arpg_soft::ArpgEnemyHover>>,
    Option<Res<'w, crate::spell::targeting::SpellTargeting>>,
    Option<Res<'w, crate::ui_script::PointerOverUiPanel>>,
    // Every unit's fields: the swing target's life and reach, ours.
    Query<'w, 's, &'static crate::net::ObjectStore>,
    Query<'w, 's, &'static crate::net::ObjectStore, With<crate::net::SelfPlayer>>,
);

/// The room the player stands in and what tells an indoor one: one parameter under Bevy's limit.
type ArpgRooms<'w, 's> = (
    Res<'w, benilla_world::wmo_portal::PlayerWmoRoom>,
    Query<'w, 's, &'static benilla_world::wmo_portal::WmoPortalInstance>,
    Res<'w, Assets<benilla_assets::WmoModel>>,
);

/// Set the rig's pin, refresh the cursor's pick for the mouse, publish the cast aim, and talk to
/// the ARPG server: the hello once per character in world, and the swing whenever the controller's
/// wish changes. A walk to a point ends any Click-to-Move approach or `/follow`, as a movement key
/// does.
fn pin_view(
    view: Res<ArpgView>,
    picks: ArpgPicks,
    keys: Res<ButtonInput<KeyCode>>,
    player: Res<Player>,
    net: Res<NetCommands>,
    transforms: Query<&Transform>,
    mut cast_aim: ResMut<crate::spell::ArpgCastAim>,
    // The room the player stands in, and the cutaway it opens.
    rooms: ArpgRooms,
    mut cutaway: ResMut<benilla_world::cutaway::Cutaway>,
    mut approach: ResMut<approach::Approach>,
    mut follow: ResMut<FollowState>,
    mut rig: ResMut<CameraControl>,
) {
    let (hovered, object, occlusion, enemy_hover, spell_targeting, over_panel, stores, me) = picks;
    let (room, instances, wmos) = rooms;
    if rig.arpg_pin != Some(view.0) {
        rig.arpg_pin = Some(view.0);
    }
    // Indoors (a WMO room or a cave), the walls, ceiling and any hill above the head are cut away
    // around the player.
    let indoor = room.0.filter(|r| room_is_indoor(*r, &instances, &wmos));
    let fresh = match indoor {
        Some(_) => benilla_world::cutaway::Cutaway {
            plane: Some(player.pos.y + view.1),
            center: player.pos,
            radius: view.2,
            dither: false,
        },
        // Outdoors, whatever stands between the player and the camera goes see-through.
        None if view.3 => xray_cut(player.pos, view.0.yaw),
        None => benilla_world::cutaway::Cutaway::default(),
    };
    if *cutaway != fresh {
        *cutaway = fresh;
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
    mouse.over_enemy = enemy_hover.and_then(|h| h.0);
    mouse.force_attack = keys.any_pressed([KeyCode::ShiftLeft, KeyCode::ShiftRight]);
    // The clicked enemy, while it lives: once it dies or streams out the swing goes on in place.
    let target_store = mouse.swing_target.and_then(|(e, _)| stores.get(e).ok());
    if mouse.swing_target.is_some() && target_store.is_none_or(|s| s.0.unit_reads_dead()) {
        mouse.swing_target = None;
    }
    mouse.swing_target_at = mouse
        .swing_target
        .and_then(|(e, _)| transforms.get(e).ok())
        .map(|t| t.translation);
    let own_reach = me
        .single()
        .map_or(DEFAULT_REACH, |s| s.0.unit_combat_reach());
    let their_reach = target_store.map_or(DEFAULT_REACH, |s| s.0.unit_combat_reach());
    mouse.swing_reach = (own_reach + their_reach + 4.0 / 3.0).max(5.0) - SWING_MARGIN;
    // The cast aim: the cursor, or ahead of the facing with the cursor on a panel or the sky, and
    // the unit under the cursor, which the server lets catch the spell.
    let ahead = Vec3::new(-player.face_yaw.sin(), 0.0, -player.face_yaw.cos()) * BLIND_AIM;
    let hovered_unit = hovered.as_ref().and_then(|h| h.guid).filter(|_| !on_panel);
    *cast_aim = crate::spell::ArpgCastAim {
        at: aim.unwrap_or(player.pos + ahead),
        intended: hovered_unit.unwrap_or(0),
    };
    // The swing the controller asked for last frame, at the enemy it was pressed on; a press on
    // another enemy while swinging re-sends the start with the new one.
    let wanted = mouse
        .swing_wanted
        .then(|| mouse.swing_target.map_or(0, |(_, guid)| guid));
    if wanted != mouse.swing_sent {
        let _ = net.0.send(ClientCommand::ArpgSwing(wanted));
        mouse.swing_sent = wanted;
    }
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

/// How long after the world entry the hello goes out a second time, in seconds.
const REGREET_SECS: f64 = 3.0;

/// Greet the ARPG server at every world entry, a reconnect and a relog of the same character
/// included, as the server's player is a new one each time; and after any map transfer re-send the
/// held swing, as a release sent during the loading screen never arrives and the server ends the
/// swing at the transfer.
fn greet_server(
    mut entered: MessageReader<crate::net::EnteredWorldMessage>,
    mut transfers: MessageReader<WorldportMessage>,
    net: Res<NetCommands>,
    rig: Option<ResMut<CameraControl>>,
    time: Res<Time>,
    // When to say hello again: the first can reach the server while it is still loading the
    // character, and the session drops it.
    mut regreet_at: Local<Option<f64>>,
) {
    let greet = entered.read().count() > 0;
    let transferred = transfers.read().count() > 0;
    let now = time.elapsed_secs_f64();
    if greet {
        info!("arpg: greeting the server as an ARPG client");
        let _ = net.0.send(ClientCommand::ArpgHello);
        *regreet_at = Some(now + REGREET_SECS);
    } else if regreet_at.is_some_and(|at| now >= at) {
        info!("arpg: greeting the server again");
        let _ = net.0.send(ClientCommand::ArpgHello);
        *regreet_at = None;
    }
    if greet || transferred {
        if let Some(mut rig) = rig {
            rig.arpg.swing_sent = None;
        }
    }
}

/// The aim is turned to only past this far from the feet, in yards: closer, a pixel of mouse
/// travel swings the bearing wildly.
const AIM_MIN: f32 = 0.5;
/// A standing character re-faces the aim only past this, in radians (about 1°): each facing change
/// is a packet, and float noise in the pick must not send one a frame.
const AIM_DEADZONE: f32 = 0.02;
/// How fast the character turns to a new facing, in radians a second (about 860°/s): a half turn
/// takes a fifth of a second, quick enough to feel instant at the mouse and slow enough that the
/// body visibly swings round rather than snapping.
const TURN_RATE: f32 = 15.0;

/// `from` turned toward `to` by at most `TURN_RATE × dt`, the short way round. A zero or
/// negative `dt` snaps, so a first frame never leaves the facing stale.
fn turn_toward(from: f32, to: f32, dt: f32) -> f32 {
    let delta = wrap_pi(to - from);
    let step = TURN_RATE * dt;
    if dt <= 0.0 || delta.abs() <= step {
        return wrap_pi(to);
    }
    wrap_pi(from + step.copysign(delta))
}

/// The outdoor see-through for a player at `feet` under a camera turned `cam_yaw`: the camera
/// looks along `(−sin, 0, −cos)` of its yaw ([`screen_facing`]), so it stands the other way.
fn xray_cut(feet: Vec3, cam_yaw: f32) -> benilla_world::cutaway::Cutaway {
    let toward_camera = Vec3::new(cam_yaw.sin(), 0.0, cam_yaw.cos());
    benilla_world::cutaway::Cutaway {
        plane: Some(feet.y + XRAY_HEIGHT),
        center: feet + toward_camera * XRAY_AHEAD,
        radius: XRAY_RADIUS,
        dither: true,
    }
}

/// The distance from `pos` to `at` on the ground, in yards.
fn flat_distance(pos: Vec3, at: Vec3) -> f32 {
    Vec2::new(at.x - pos.x, at.z - pos.z).length()
}

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
    let (fwd, side) = octant_axes(octant);
    (facing, fwd, side)
}

/// The stock axes nearest a walk along `move_yaw` from the body's actual `facing`: while the body
/// turns toward [`aim_and_walk`]'s facing, the walk keeps to the pressed way within 22.5°.
fn walk_axes(move_yaw: f32, facing: f32) -> (i32, i32) {
    use std::f32::consts::FRAC_PI_4;
    octant_axes((wrap_pi(move_yaw - facing) / FRAC_PI_4).round() as i32)
}

/// `(fwd, side)` for a walk `octant` eighths of a turn left of the facing, side positive right.
fn octant_axes(octant: i32) -> (i32, i32) {
    match octant {
        0 => (1, 0),
        1 => (1, -1),
        -1 => (1, 1),
        2 => (0, -1),
        -2 => (0, 1),
        3 => (-1, -1),
        -3 => (-1, 1),
        _ => (-1, 0),
    }
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
    // The held swing at a clicked enemy walks to it until in reach; a key still steers. Once
    // swinging, the enemy may step a little further before the walk resumes.
    let reach = if mouse.swing_wanted {
        mouse.swing_reach + SWING_SLACK
    } else {
        mouse.swing_reach
    };
    let swing_gap = mouse.swing_target_at.map(|at| flat_distance(pos, at));
    let swing_walk = match (mouse.swing_held(), mouse.swing_target_at, swing_gap) {
        (true, Some(at), Some(gap)) if gap > reach && may_turn => yaw_toward(pos, at),
        _ => None,
    };
    let move_yaw = move_yaw.or(swing_walk);
    // The server swings while the swing is held, once a clicked enemy is in reach.
    mouse.swing_wanted = mouse.swing_held() && swing_gap.is_none_or(|gap| gap <= reach);
    let aim_yaw = mouse.aim.and_then(|at| yaw_toward(pos, at));
    let (mut fwd, mut side) = (axes.fwd, 0);
    match (move_yaw, aim_yaw) {
        // Walking and aiming: face the aim, walk the way the player asked.
        (Some(walk), Some(aim)) if may_turn => {
            let (facing, _, _) = aim_and_walk(walk, aim);
            *face_yaw = turn_toward(*face_yaw, facing, dt);
            (fwd, side) = walk_axes(walk, *face_yaw);
        }
        // Walking with no aim (the cursor on a panel or the sky): face the walk.
        (Some(walk), _) => {
            if may_turn {
                *face_yaw = turn_toward(*face_yaw, walk, dt);
                (fwd, side) = walk_axes(walk, *face_yaw);
            } else {
                fwd = 1;
            }
        }
        // Standing with nothing else steering: face the aim.
        (None, Some(aim))
            if axes.fwd == 0
                && may_turn
                && standing
                && wrap_pi(aim - *face_yaw).abs() > AIM_DEADZONE =>
        {
            *face_yaw = turn_toward(*face_yaw, aim, dt);
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
    fn the_outdoor_dither_sits_between_the_player_and_the_camera() {
        let feet = Vec3::new(10.0, 5.0, -3.0);
        for cam in [0.0, 0.785, -2.0] {
            let cut = xray_cut(feet, cam);
            assert!(cut.dither);
            // Up the screen is away from the camera, so the disc lies the other way.
            let up = screen_facing(cam, 1, 0).unwrap();
            let away = Vec3::new(-up.sin(), 0.0, -up.cos());
            assert!(
                (cut.center - feet).dot(away) < -XRAY_AHEAD + 1e-4,
                "cam {cam}"
            );
            // A roof edge toward the camera is inside it, a wall beyond the player and the
            // player's own ground are not.
            assert!(!cut.cuts(feet + away * 3.0 + Vec3::Y * 4.0));
            assert!(!cut.cuts(feet + Vec3::Y));
            assert!(cut.cuts(feet - away * XRAY_AHEAD + Vec3::Y * 4.0));
        }
    }

    #[test]
    fn the_hud_addon_loads_and_paints() {
        let script = benilla_ui::script::UiScript::new().expect("VM");
        // The addon's own code, then a paint of each part in the same chunk, where its locals are
        // in scope; the bare VM has no player, so the orbs read empty.
        let chunk = format!(
            "if not UIParent then CreateFrame(\"Frame\", \"UIParent\") end\n\
             for _, f in ipairs({{\"GameFontHighlight\", \"GameFontNormalSmall\"}}) do\n\
             if not _G[f] then CreateFont(f) end end\n{HUD_LUA}\n\
             refresh()\nrefreshOrbs()\n\
             paintOrb(healthOrb, 0.5, HEALTH_COLOR, 50)\n\
             assert(healthOrb.filled == ORB_SLICES / 2, healthOrb.filled)\n\
             paintOrb(powerOrb, 2, POWER_COLORS[1], 9)\n\
             assert(powerOrb.filled == ORB_SLICES)\n"
        );
        script.run(&chunk).expect("the HUD addon runs");
    }

    #[test]
    fn a_walk_mid_turn_keeps_to_the_pressed_way() {
        use std::f32::consts::PI;
        // Facing ahead and asked to walk back: backpedal until the body has turned.
        assert_eq!(walk_axes(PI, 0.0), (-1, 0));
        // Half way round, a strafe.
        assert_eq!(walk_axes(PI, PI / 2.0), (0, -1));
        // Turned, forward.
        assert_eq!(walk_axes(PI, PI - 0.1), (1, 0));
    }

    #[test]
    fn a_turn_is_rate_limited_and_takes_the_short_way() {
        use std::f32::consts::PI;
        // A small turn lands at once; a half turn takes several frames.
        assert!(gap(turn_toward(0.0, 0.1, 1.0 / 60.0), 0.1) < 1e-6);
        let one = turn_toward(0.0, PI - 0.01, 1.0 / 60.0);
        assert!((one - TURN_RATE / 60.0).abs() < 1e-5);
        // Across the ±π seam, the turn goes the short way, not the long way round.
        let seam = turn_toward(PI - 0.1, -PI + 0.1, 1.0 / 60.0);
        assert!(gap(seam, PI) < 0.11, "{seam}");
        // A zero step snaps.
        assert_eq!(turn_toward(0.0, 2.0, 0.0), 2.0);
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
