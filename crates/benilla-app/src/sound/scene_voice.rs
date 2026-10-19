//! Fork-only, not 1.12.1: the recorded lines of the ARPG scenes (`SMSG_ARPG_SCENE`, cmangos
//! `Arpg/ArpgScenes.cpp`). Each subtitle that has a recording speaks it as its window opens; the
//! next line, the scene's end or a skip cuts it off. The clips ship inside the binary, since no
//! MPQ holds them; a line with none stays a subtitle.
//!
//! Keyed by scene id and line index, which the server never reuses: retiming a scene's windows
//! there keeps its recordings, re-ordering its lines needs this table changed too.

use bevy::prelude::*;

use benilla_world::schedule::WorldStage;

use super::mixer::{self, StreamingSoundHandle};
use super::SoundOutput;
use crate::cinematic::Cinematic;

/// `(scene id, line index, mp3)`.
const VOICES: &[(u32, usize, &[u8])] = &[
    // Scene 1, Marshal McBride in Northshire.
    (1, 0, include_bytes!("../../assets/voice/mcbride_1.mp3")),
    (1, 1, include_bytes!("../../assets/voice/mcbride_2.mp3")),
    (1, 2, include_bytes!("../../assets/voice/mcbride_3.mp3")),
    // Scene 3, Hogger first sighted.
    (3, 0, include_bytes!("../../assets/voice/hogger.mp3")),
];

/// A line's recording, if it has one.
fn clip(scene: u32, line: usize) -> Option<&'static [u8]> {
    VOICES
        .iter()
        .find(|&&(s, l, _)| s == scene && l == line)
        .map(|&(_, _, bytes)| bytes)
}

/// The line being spoken: `(cinematic run, line index)`, so a replayed scene speaks again.
#[derive(Default)]
struct SceneVoice {
    line: Option<(u64, usize)>,
    handle: Option<StreamingSoundHandle<kira::sound::FromFileError>>,
}

impl SceneVoice {
    fn is_live(&self) -> bool {
        self.handle
            .as_ref()
            .is_some_and(|h| !matches!(h.state(), kira::sound::PlaybackState::Stopped))
    }

    fn stop(&mut self) {
        self.line = None;
        if let Some(mut h) = self.handle.take() {
            h.stop(mixer::fade(40));
        }
    }
}

pub(super) fn plugin(app: &mut App) {
    // In `Present`, after the cinematic driver in `Input` has settled this frame's time, and after
    // a logout has stopped the scene, so its voice stops in the same frame.
    app.add_systems(
        Update,
        drive_scene_voice
            .in_set(WorldStage::Present)
            .after(crate::ui_logout::drain_logout),
    );
}

fn drive_scene_voice(
    cine: Option<Res<Cinematic>>,
    mut voice: Local<SceneVoice>,
    mut out: NonSendMut<SoundOutput>,
) {
    speak(cine.as_deref(), &mut voice, &mut out);
    out.scene_streams = usize::from(voice.is_live());
}

fn speak(cine: Option<&Cinematic>, voice: &mut SceneVoice, out: &mut SoundOutput) {
    let playing = cine.and_then(|c| Some((c.playing_shot()?.0, c.scene_at()?)));
    let Some((run, (scene, ms))) = playing else {
        // No scene, or it just ended or was skipped.
        voice.stop();
        return;
    };
    // Between windows the last line plays out.
    let Some(line) = scene
        .lines
        .iter()
        .position(|l| ms >= l.from_ms && ms < l.to_ms)
    else {
        return;
    };
    if voice.line == Some((run, line)) {
        return;
    }
    voice.stop();
    voice.line = Some((run, line));
    let Some(bytes) = clip(scene.id, line) else {
        return;
    };
    let Some(mixer_ref) = out.mixer.as_mut() else {
        return;
    };
    match mixer::stream_from_bytes(bytes.to_vec()).and_then(|d| mixer_ref.play_stream(d)) {
        Ok(h) => voice.handle = Some(h),
        Err(e) => warn!("scene voice: scene {} line {line} — {e:#}", scene.id),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_recording_decodes() {
        for &(scene, line, bytes) in VOICES {
            assert!(
                mixer::stream_from_bytes(bytes.to_vec()).is_ok(),
                "scene {scene} line {line}"
            );
        }
        assert!(clip(1, 2).is_some());
        assert!(clip(1, 3).is_none());
    }
}
