//! Fork-only, not 1.12.1: the ARPG view's dodge roll and health flask (cmangos
//! `Arpg/ArpgActions.h`). The Jump binding (Space) rolls instead of jumping: toward the walk the
//! movement keys ask for, else toward the cursor's aim. The server answers with a knockback, which
//! the client flies as it does any other, and while airborne the character dodges every blow. The
//! Strafe Left binding (Q) drinks a flask charge; the A key still walks left. The server sends the
//! flask's charges and the roll's cooldown (`SMSG_ARPG_STATUS`), which the health orb shows
//! (`ArpgHud_Status` in `arpg_hud.lua`). The dungeon difficulty tier the ARPG View page sets
//! (`arpgDungeonTier`) goes to the server at each world entry and on every change.

use benilla_protocol::messages::ArpgStatus;
use benilla_protocol::{SessionEvent, SessionEventKind};
use benilla_ui::script::UiScript;

use crate::net::NetHandlerApp;

use super::*;

/// How far ahead of the character a roll along the walk aims, in yards; the server's leap is
/// about seven yards whatever the point's distance.
const ROLL_AIM: f32 = 8.0;

/// The server's last word on the flask and the roll, and when it came.
#[derive(Resource, Default)]
pub(crate) struct ArpgActionState {
    status: Option<ArpgStatus>,
    /// The session clock, in seconds, at which the roll is ready again.
    dodge_ready_at: f64,
    /// Bumped by every status, so the orb is told once per change.
    generation: u64,
}

/// Registered whatever the view, so every session event kind has its owner.
pub(super) fn register_net(app: &mut App) {
    app.init_resource::<ArpgActionState>()
        .net_handler(SessionEventKind::ArpgStatus, on_status);
}

fn on_status(In(ev): In<SessionEvent>, mut state: ResMut<ArpgActionState>, time: Res<Time>) {
    if let SessionEvent::ArpgStatus { status } = ev {
        state.dodge_ready_at = time.elapsed_secs_f64() + f64::from(status.dodge_ready_ms) / 1000.0;
        state.status = Some(status);
        state.generation += 1;
    }
}

/// The dungeon tier setting.
pub(crate) const CVAR_DUNGEON_TIER: &str = "arpgDungeonTier";
/// The highest tier, Torment III.
const MAX_TIER: f32 = 5.0;
/// The tier goes again this long after a world entry, in seconds, as the hello does: the first
/// can reach the server while it still loads the character.
const TIER_RESEND_SECS: f64 = 3.0;

fn wanted_tier(cvars: &crate::cvars::Cvars) -> u8 {
    cvars
        .num(CVAR_DUNGEON_TIER)
        .filter(|v| v.is_finite())
        .unwrap_or(0.0)
        .round()
        .clamp(0.0, MAX_TIER) as u8
}

/// The page's slider moved: tell the server.
fn on_tier_changed(
    ev: On<crate::cvars::CvarChanged>,
    cvars: Res<crate::cvars::Cvars>,
    net: Res<NetCommands>,
) {
    if ev.is(CVAR_DUNGEON_TIER) {
        let _ = net.0.send(ClientCommand::ArpgTier {
            tier: wanted_tier(&cvars),
        });
    }
}

/// Tell the server the tier at each world entry, and again a moment later.
fn send_tier_on_entry(
    mut entered: MessageReader<crate::net::EnteredWorldMessage>,
    cvars: Option<Res<crate::cvars::Cvars>>,
    net: Res<NetCommands>,
    time: Res<Time>,
    mut resend_at: Local<Option<f64>>,
) {
    let now = time.elapsed_secs_f64();
    let fresh = entered.read().count() > 0;
    let due = resend_at.is_some_and(|at| now >= at);
    if !fresh && !due {
        return;
    }
    *resend_at = fresh.then_some(now + TIER_RESEND_SECS);
    if let Some(cvars) = cvars {
        let _ = net.0.send(ClientCommand::ArpgTier {
            tier: wanted_tier(&cvars),
        });
    }
}

/// The view's half: the keys, and the orb's line.
pub(super) fn plugin(app: &mut App) {
    app.add_observer(on_tier_changed)
        .add_systems(Update, send_tier_on_entry);
    app.add_systems(
        Update,
        press_actions
            .in_set(WorldStage::Input)
            .after(pin_view)
            .before(control)
            .in_set(crate::char_select::InWorldGated),
    )
    .add_systems(Update, push_status);
}

/// Where a roll goes: along the movement keys' walk when one is held, else at the cursor's aim.
fn roll_point(binds: &BindingsState, pos: Vec3, cam_yaw: f32, aim: Vec3) -> Vec3 {
    let key = |input: Input| i32::from(binds.pressed(input));
    let up = key(Input::MoveForward) - key(Input::MoveBackward);
    let right = (key(Input::StrafeRight) + key(Input::TurnRight) - key(Input::TurnLeft)).signum();
    match screen_facing(cam_yaw, up, right) {
        Some(yaw) => pos + Vec3::new(-yaw.sin(), 0.0, -yaw.cos()) * ROLL_AIM,
        None => aim,
    }
}

/// Space rolls and Q drinks, in the ARPG view.
fn press_actions(
    binds: Res<BindingsState>,
    player: Res<Player>,
    rig: Res<CameraControl>,
    cast_aim: Res<crate::spell::ArpgCastAim>,
    net: Res<NetCommands>,
    time: Res<Time>,
    mut state: ResMut<ArpgActionState>,
) {
    let Some(pin) = rig.arpg_pin else {
        return;
    };
    let now = time.elapsed_secs_f64();
    if binds.fired(Input::Jump) && now >= state.dodge_ready_at {
        let at = roll_point(&binds, player.pos, pin.yaw, cast_aim.at);
        let [x, y, _] = benilla_assets::coords::bevy_to_wow(at);
        let _ = net.0.send(ClientCommand::ArpgDodge { x, y });
        // Until the server's answer, assume the roll went: one press, one roll.
        let cooldown = state.status.map_or(2500, |s| s.dodge_cooldown_ms);
        state.dodge_ready_at = now + f64::from(cooldown) / 1000.0;
    }
    if binds.just_pressed(Input::StrafeLeft) && state.status.is_none_or(|s| s.flask_charges > 0) {
        let _ = net.0.send(ClientCommand::ArpgFlask);
    }
}

/// The Lua chunk that tells the health orb about the flask and the roll.
fn status_chunk(s: &ArpgStatus) -> String {
    format!(
        "if not ArpgHud_Status then return false end\nArpgHud_Status({}, {}, {}, {}, {})\nreturn true",
        s.flask_charges, s.flask_max, s.flask_progress, s.dodge_ready_ms, s.dodge_cooldown_ms
    )
}

/// Tell the orb each new status.
fn push_status(
    script: Option<NonSendMut<UiScript>>,
    state: Res<ArpgActionState>,
    mut pushed: Local<Option<(u64, u64)>>,
) {
    let (Some(script), Some(status)) = (script, state.status) else {
        return;
    };
    let mark = (script.session(), state.generation);
    if *pushed == Some(mark) {
        return;
    }
    match script.eval::<bool>(&status_chunk(&status)) {
        Ok(true) => *pushed = Some(mark),
        Ok(false) => {}
        Err(e) => {
            warn!("arpg: the orb refused the status: {e}");
            *pushed = Some(mark);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_status_chunk_calls_the_orb_with_every_field() {
        let chunk = status_chunk(&ArpgStatus {
            flask_charges: 2,
            flask_max: 3,
            flask_progress: 40,
            dodge_ready_ms: 900,
            dodge_cooldown_ms: 2500,
        });
        assert!(chunk.contains("ArpgHud_Status(2, 3, 40, 900, 2500)"));
        let script = UiScript::new().unwrap();
        script
            .run("function ArpgHud_Status(c, m, p, r, d) GOT = c + m + p + r + d end")
            .unwrap();
        assert!(script.eval::<bool>(&chunk).unwrap());
        assert_eq!(
            script.eval::<u32>("return GOT").unwrap(),
            2 + 3 + 40 + 900 + 2500
        );
        assert!(!UiScript::new().unwrap().eval::<bool>(&chunk).unwrap());
    }

    #[test]
    fn a_roll_with_no_key_held_goes_to_the_aim() {
        let binds = BindingsState::default();
        let aim = Vec3::new(4.0, 0.0, -3.0);
        assert_eq!(roll_point(&binds, Vec3::ZERO, 0.0, aim), aim);
    }
}
