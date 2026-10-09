# benilla ARPG fork

This fork (Gribby/benilla, branch `arpg`) turns benilla into a Diablo-style ARPG client for a
self-hosted cmangos Classic + playerbots server. It pairs with the server fork Gribby/mangos-classic
(branch `arpg`), which owns hit resolution. **Read this before `AGENTS.md`'s rules:** upstream
benilla is a faithful 1.12.1 client; this fork deliberately is not, but only behind a switch.

## Ground rules

- Everything ARPG is gated. Client: `WOW_ARPG=1`. Server: `Arpg.Enable = 1` in `mangosd.conf`, and
  a player becomes ARPG only after the client's hello. With either off, stock behaviour is untouched
  (bots, stock clients, the stock benilla build).
- Mark fork code with `Fork-only, not 1.12.1:` in its doc comment, as the existing ARPG code does.
- Keep upstream's quality bar: `cargo clippy --all-targets -- -D warnings`, `cargo fmt`, tests for
  new logic, doc comments in the house style.

## What exists

Client (`crates/`):

| Area | Where |
|---|---|
| Top-down pinned camera, camera-relative WASD, 8-way walk facing the cursor, smooth turning | `benilla-app/src/player/arpg.rs` (`steer`, `aim_and_walk`, `turn_toward`) |
| Free cursor, hold-to-move, click to swing/interact | `benilla-app/src/player/camera.rs` (`run_arpg_clicks`) |
| No selection; magnet hover on enemies | `benilla-app/src/target/arpg_soft.rs` |
| Casts at the cursor (`CMSG_ARPG_ACTION` kind 3) | `benilla-app/src/spell/cast_send.rs`, `cast_target.rs`, `targeting/world.rs` |
| Wire format | `benilla-protocol/src/world/writer/arpg.rs` |
| Indoor roof/cave cutaway; outdoor see-through dither | `benilla-world/src/cutaway.rs`, lanes in `lighting/global_light.rs`, discards in `static_gx.wgsl`, `wow_model.wgsl`, `terrain.wgsl` |
| Hit flash on struck units | `benilla-app/src/player/arpg/fx.rs`, `benilla-world/src/instance_tint.rs::with_flash`, `wow_model.wgsl` |
| Hover health bar, health/power orbs and the options window's ARPG View page (Lua addon the client installs) | `benilla-app/src/player/arpg_hud.lua` |
| The Attack key toggles a swing at the enemy under the cursor | `ArpgAttackKey` in `player/arpg.rs`, `ui_action/drain.rs` |
| Ground loot: glows, beams, labels, click to pick up, gold on walk-over | `benilla-app/src/player/arpg/loot.rs`, `benilla-protocol/src/messages/arpg.rs` |

Floating damage numbers are stock benilla (`combat_text`).

View settings: Esc → Options → **ARPG View** (the page the ARPG HUD addon adds to benilla's
options window) has sliders for camera distance, pitch and yaw, the indoor cutaway's height and
radius, and a see-through checkbox. They are saved CVars (`arpgCameraYaw` 45, `arpgCameraPitch` 55,
`arpgCameraDistance` 28, `arpgCutHeight` 2.8, `arpgCutRadius` 30, `arpgSeeThrough` 1) that the view
reads every frame (`ARPG_KNOBS` in `player/arpg.rs`), so a slider acts at once. The env variables
`WOW_ARPG_YAW`, `WOW_ARPG_PITCH`, `WOW_ARPG_DIST`, `WOW_ARPG_CUT`, `WOW_ARPG_CUT_RADIUS` and
`WOW_ARPG_XRAY` still work: each seeds its setting for that session only, and then the slider's
moves aren't saved. `WOW_ARPG_MAGNET` (3) stays env-only.

Server (`src/game/Arpg/`, plus hooks in `Player`, `Unit`, `Spell`, `SpellEffects`,
`UnitAuraProcHandler`, `Opcodes`, `World`): `CMSG_ARPG_ACTION` = 0x33C, protocol version 2.
Kinds: 0 hello (`u8` version), 1 swing start (`u64` intended), 2 swing stop, 3 cast (`u32` spell,
`u8` aim 0 enemy/1 ally, `f32` x y z WoW coords, `u64` intended), 4 aim (`f32` x y z, `u64`
intended: re-aims the running cast), 5 loot (`u64` corpse, `u8` loot slot, 0xFF the gold), 6 loot
query (`u64` corpse). A swing, cast, aim or loot action also counts as the hello (the first hello
can arrive while the character still loads). Server to client: `SMSG_ARPG_LOOT` = 0x33D (`u64`
corpse, `u32` gold, `u8` n, then n × `u8` slot, `u32` item, `u32` display, `u8` quality, `u8`
count), sent at the kill, after any change, and on a query (`src/game/Arpg/ArpgLoot.{h,cpp}`). Swings strike whoever is in the
frontal arc or whiff; line skillshots pick the first enemy along the aim and, with none (or a target
that dies mid-flight), fly on to max range and are spent.

## Roadmap

Done: phase 1 (targetless combat, whiffs, cutaway, hover bar), phase 2 (empty-air skillshots,
hit flash, smooth turning, outdoor dither, orbs), skillshots that re-aim at the cursor while the
cast runs and hit neutral wild creatures, and ground loot (phase 3's first item).

Ground loot today: grey and white drops glow faintly with no beam; green, blue, purple and orange
glow brighter with a beam that grows with the quality (`look()` in `loot.rs`). Labels show within
30 yd, all of them with Alt. The server drops the group loot rules for ARPG players (their loot is
free for all, nothing waits on a roll) and binds nothing to them, on pickup, equip or use. Known
gaps: drops vanish with the corpse (no persistent ground items); no item models yet (weapons and
shields could lie as their real M2s, everything else a sack).

Next:
- Camera (banked by Jeff, after gameplay): the height cut fails in multi-storey instances
  (Naxxramas, Blackrock Spire). Plan: cut by room, hiding whole WMO groups above the player's
  room (`wmo_portal::PlayerWmoRoom` already tracks it); fade anything left on the camera→player
  sightline as the outdoor dither does; keep the height cut as the fallback for open caves
  (Molten Core). Floors more than `FLOOR_RISE` over the plane are already cut.
- Phase 1/2 leftovers: torches/fire and water above the cut plane still draw; the dither skips
  animated M2 doodads drawn by `wow_model.wgsl` (only WMO/interior there are cut).
- Phase 3: more and denser mobs (server spawn scaling), mob packs, dodge/evade movement skill,
  potions on hotkeys, real item models on the ground.
- Itemisation: see `docs/ARPG-ITEMISATION.md`.
- Phase 4: vanilla raids as weekly-lockout solo ARPG dungeons (scaled bosses, trash density,
  pacing like vanilla's raid week).

## Building and verifying

Windows (the user's machine): `cargo build --release -p benilla`, then run `target\release\benilla.exe` with
`WOW_ARPG=1` and `WOW_DATA` pointing at a 1.12.1 `Data` folder. Use `cargo build`, not
`cargo test`, on Windows (some upstream tests use `std::os::unix`).

Linux sandbox (cloud sessions, ~7 GB RAM):

```sh
apt-get install -y libasound2-dev libudev-dev libwayland-dev libxkbcommon-dev
export RUSTUP_TOOLCHAIN=stable CARGO_PROFILE_DEV_DEBUG=0   # pinned 1.98.1 may be missing
cargo clippy -j2 -p benilla-app -p benilla-world -p benilla-assets --all-targets -- -D warnings
cargo test -j2 -p benilla-app player::arpg
cargo test -j2 -p benilla-world cutaway instance_tint
cargo test -j2 -p benilla-protocol arpg
```

Run clippy, tests and the server build one at a time, or the linker gets OOM-killed. Shader
changes cannot be exercised without a GPU: validate them with naga + naga_oil (compose with the
bevy_pbr/render/shader/core_pipeline/mesh 0.18.1 sources as import roots and `crates/`).

Server: cmangos builds with CMake (`-DPCH=1` recommended; re-run cmake after adding files, sources
are globbed). `make -j2 mangosd` in the build directory.
