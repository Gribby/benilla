//! Fork-only, not 1.12.1: the ARPG server's scenes (`SMSG_ARPG_SCENE`, cmangos
//! `Arpg/ArpgScenes.h`): a camera track over the live world, with subtitles, played by the
//! cinematic player ([`crate::cinematic`]) as a shot of its own, so it has the letterbox, the
//! hidden UI and cursor, the fades and the ESC skip, and ends with `CMSG_COMPLETE_CINEMATIC` as a
//! cinematic does (the server's sign that the scene is over).

use benilla_protocol::messages::arpg::ArpgScene;
use benilla_world::schedule::WorldStage;
use bevy::prelude::*;

use crate::cinematic::Cinematic;
use crate::net::NetHandlerApp;

/// Registered whatever the view, so every session event kind has its owner.
pub(super) fn register_net(app: &mut App) {
    app.net_handler(benilla_protocol::SessionEventKind::ArpgScene, on_scene);
}

pub(super) fn plugin(app: &mut App) {
    app.add_systems(Startup, spawn_subtitles).add_systems(
        Update,
        drive_subtitles
            .in_set(WorldStage::Input)
            .after(crate::cinematic::CinematicSet),
    );
}

fn on_scene(In(ev): In<benilla_protocol::SessionEvent>, mut cine: ResMut<Cinematic>) {
    if let benilla_protocol::SessionEvent::ArpgScene { scene } = ev {
        info!(
            "scene: {} arrived, {} ms, {} keys, {} lines",
            scene.id,
            scene.duration_ms,
            scene.keys.len(),
            scene.lines.len()
        );
        cine.queue_scene(scene);
    }
}

/// The eye and the point it looks at, `ms` into `scene`, in WoW coordinates: a Catmull-Rom
/// spline through the keys, so the camera glides through them; held at the first and last.
pub(crate) fn scene_view(scene: &ArpgScene, ms: u32) -> ([f32; 3], [f32; 3]) {
    let keys = &scene.keys;
    match keys.len() {
        0 => return ([0.0; 3], [0.0, 0.0, -1.0]),
        1 => return (keys[0].eye, keys[0].target),
        _ => {}
    }
    if ms <= keys[0].ms {
        return (keys[0].eye, keys[0].target);
    }
    let last = keys.len() - 1;
    if ms >= keys[last].ms {
        return (keys[last].eye, keys[last].target);
    }
    // The segment `i .. i + 1` holding `ms`.
    let i = keys
        .windows(2)
        .position(|w| ms < w[1].ms)
        .unwrap_or(last - 1);
    let (a, b) = (&keys[i], &keys[i + 1]);
    let span = b.ms.saturating_sub(a.ms).max(1) as f32;
    let t = (ms.saturating_sub(a.ms)) as f32 / span;
    let p0 = &keys[i.saturating_sub(1)];
    let p3 = &keys[(i + 2).min(last)];
    (
        catmull_rom(p0.eye, a.eye, b.eye, p3.eye, t),
        catmull_rom(p0.target, a.target, b.target, p3.target, t),
    )
}

fn catmull_rom(p0: [f32; 3], p1: [f32; 3], p2: [f32; 3], p3: [f32; 3], t: f32) -> [f32; 3] {
    let (t2, t3) = (t * t, t * t * t);
    std::array::from_fn(|k| {
        0.5 * (2.0 * p1[k]
            + (p2[k] - p0[k]) * t
            + (2.0 * p0[k] - 5.0 * p1[k] + 4.0 * p2[k] - p3[k]) * t2
            + (3.0 * p1[k] - p0[k] - 3.0 * p2[k] + p3[k]) * t3)
    })
}

/// The subtitle block: the speaker over the line, centred in the lower letterbox bar.
#[derive(Component)]
struct Subtitles;
#[derive(Component)]
struct SubtitleSpeaker;
#[derive(Component)]
struct SubtitleLine;

/// The speaker's name: the gold of a quest giver's name in the quest frame.
const SPEAKER: Color = Color::srgb(1.0, 0.82, 0.0);

fn spawn_subtitles(mut commands: Commands, assets: Res<AssetServer>) {
    let font: Handle<Font> = assets.load("mpq://Fonts/FRIZQT__.ttf");
    commands
        .spawn((
            Subtitles,
            Node {
                position_type: PositionType::Absolute,
                left: Val::Percent(10.0),
                width: Val::Percent(80.0),
                bottom: Val::Px(0.0),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                ..default()
            },
            // Over the letterbox bars (900).
            GlobalZIndex(901),
            Visibility::Hidden,
        ))
        .with_children(|block| {
            block.spawn((
                SubtitleSpeaker,
                Text::new(""),
                TextFont {
                    font: font.clone(),
                    font_size: 16.0,
                    ..default()
                },
                TextColor(SPEAKER),
                TextLayout::new_with_justify(Justify::Center),
            ));
            block.spawn((
                SubtitleLine,
                Text::new(""),
                TextFont {
                    font,
                    font_size: 21.0,
                    ..default()
                },
                TextColor(Color::WHITE),
                TextLayout::new_with_justify(Justify::Center),
            ));
        });
}

/// Show the scene's line for this moment, if any, in the lower bar.
#[allow(clippy::type_complexity)]
fn drive_subtitles(
    cine: Res<Cinematic>,
    windows: Query<&Window>,
    mut block: Query<(&mut Node, &mut Visibility), With<Subtitles>>,
    mut speaker: Query<&mut Text, (With<SubtitleSpeaker>, Without<SubtitleLine>)>,
    mut line: Query<&mut Text, (With<SubtitleLine>, Without<SubtitleSpeaker>)>,
) {
    let now = cine
        .scene_at()
        .and_then(|(scene, ms)| scene.lines.iter().find(|l| ms >= l.from_ms && ms < l.to_ms));
    let Ok((mut node, mut vis)) = block.single_mut() else {
        return;
    };
    let want = if now.is_some() {
        Visibility::Inherited
    } else {
        Visibility::Hidden
    };
    if *vis != want {
        *vis = want;
    }
    let Some(now) = now else { return };
    // Inside the lower bar, a little above the screen's edge.
    let bar = windows.iter().next().map_or(0.0, |w| {
        crate::cinematic::letterbox_bar(w.width(), w.height().max(1.0))
    });
    let bottom = Val::Px((bar * 0.5 - 22.0).max(12.0));
    if node.bottom != bottom {
        node.bottom = bottom;
    }
    if let Ok(mut text) = speaker.single_mut() {
        if text.0 != now.speaker {
            text.0.clone_from(&now.speaker);
        }
    }
    if let Ok(mut text) = line.single_mut() {
        if text.0 != now.text {
            text.0.clone_from(&now.text);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use benilla_protocol::messages::arpg::ArpgSceneKey;

    fn key(ms: u32, x: f32) -> ArpgSceneKey {
        ArpgSceneKey {
            ms,
            eye: [x, 0.0, 10.0],
            target: [x, 5.0, 0.0],
        }
    }

    #[test]
    fn the_track_passes_through_its_keys_and_holds_at_the_ends() {
        let scene = ArpgScene {
            keys: vec![key(1000, 0.0), key(3000, 10.0), key(6000, 40.0)],
            ..Default::default()
        };
        assert_eq!(scene_view(&scene, 0).0, [0.0, 0.0, 10.0]);
        assert_eq!(scene_view(&scene, 3000).0[0], 10.0);
        assert_eq!(scene_view(&scene, 9000).0[0], 40.0);
        let mid = scene_view(&scene, 2000).0[0];
        assert!(mid > 0.0 && mid < 10.0, "{mid}");
        let later = scene_view(&scene, 4500).0[0];
        assert!(later > 10.0 && later < 40.0, "{later}");
    }

    #[test]
    fn no_keys_is_no_panic() {
        let scene = ArpgScene::default();
        let _ = scene_view(&scene, 500);
    }
}
