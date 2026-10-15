//! Fork-only, not 1.12.1: the ARPG class screen's five stages. The stock client has one glue
//! booth, and its create screen stands one character in one race's `UI_*` scene. The class screen
//! (`crate::class_showcase`) stands five side by side instead, one per class the ARPG server
//! offers, each a stock glue scene with the server's showcase character for that class standing
//! in it (`SMSG_ARPG_SHOWCASE`).
//!
//! Each stage is a booth of its own, built as the glue booth builds its own: the scene through
//! [`build_stage`] and the character through [`bake_character`], so the five look as the create
//! screen does. A stage renders into a target the width of one slice of the window and its full
//! height while the screen is up, and drops back to a small idle target with its scene and
//! character torn down when it closes.

use std::collections::HashMap;

use benilla_assets::materials::WowModelMaterial;
use benilla_assets::{m2_url, M2Model};
use bevy::camera::visibility::RenderLayers;
use bevy::camera::PerspectiveProjection;
use bevy::prelude::*;
use bevy::window::PrimaryWindow;

use crate::entities::Creatures;

use super::framing::{diag_to_vert, GLUE_AUTHORED_ASPECT};
use super::glue_booth::{
    apply_yaw, bake_character, build_stage, resize_target, scene_rig, scene_token, CharacterLane,
    StageKit, StageSpec,
};
use super::{
    aim, new_target_image, Booth, BoothCam, BoothLight, Booths, GluePreviewBake, GlueScene,
    PortraitImages, PortraitSource, SelectLook, SHOWCASE_LAYER_BASE, SHOWCASE_SLOTS,
};

/// Fork-only, not 1.12.1: the class screen's slices, left to right, as (class id, the stock glue
/// scene it stands in). The scenes are the race stages of the create screen, chosen to suit each
/// class rather than the race it is shown as: Paladin before Stormwind's gate (`UI_Human`), Mage
/// in the Undercity's green gloom (`UI_Scourge`), Druid in the moonlit grove (`UI_NightElf`),
/// Warlock before Orgrimmar's gate (`UI_Orc`; the login screen's Dark Portal has no seat for a
/// character, which stood floating at its origin), and Shaman on the
/// plains of Mulgore (`UI_Tauren`).
pub(crate) const SHOWCASE: [(u8, GlueScene); SHOWCASE_SLOTS] = [
    (2, GlueScene::Race(1)),
    (8, GlueScene::Race(5)),
    (11, GlueScene::Race(4)),
    (9, GlueScene::Race(2)),
    (7, GlueScene::Race(6)),
];

/// Fork-only, not 1.12.1: each slice's booth token, its key in [`PortraitImages`] and
/// [`Booths`], in [`SHOWCASE`]'s order.
pub(crate) const SHOWCASE_TOKENS: [&str; SHOWCASE_SLOTS] = [
    "showcase0",
    "showcase1",
    "showcase2",
    "showcase3",
    "showcase4",
];

/// Each stage's light buffer key in the palette, tint and mat-anim mirror tables, and its GPU
/// label. One per stage: two buffers under one key would leave one of them stale.
const LIGHT_KEYS: [&str; SHOWCASE_SLOTS] = [
    "showcase_scene0",
    "showcase_scene1",
    "showcase_scene2",
    "showcase_scene3",
    "showcase_scene4",
];

/// The idle target's side: a stage with nothing to show keeps a token target, not a slice.
const IDLE_SIZE: u32 = 64;

/// How far each slice closes the authored vertical opening. A slice is tall and narrow (about
/// 0.36:1 on a 16:9 window), so the scene's sides crop away; closing in a little puts the
/// character, not the stage's floor and sky, in the frame.
const SHOWCASE_ZOOM: f32 = 0.8;

/// Fork-only, not 1.12.1: the class screen's input to its stages, written by
/// `crate::class_showcase`: whether the screen is up, the look standing in each slice (`None`
/// until the server's showcase lands, or for a class it sent none for), and the facing all five
/// share, in radians.
#[derive(Resource, Default)]
pub(crate) struct ShowcasePreview {
    pub(crate) active: bool,
    pub(crate) looks: [Option<SelectLook>; SHOWCASE_SLOTS],
    pub(crate) yaw: f32,
}

/// Fork-only, not 1.12.1: the entities-side builder's output per slice, as
/// [`GluePreviewBake`] is the glue booth's; each re-bakes only when its own revision moves.
#[derive(Resource, Default)]
pub(crate) struct ShowcaseBakes(pub(crate) [GluePreviewBake; SHOWCASE_SLOTS]);

/// One slice's stage: the scene standing in it and the seat it authors for the character, the
/// stage half of what [`super::glue_booth::CreateScene`] holds for the glue booth.
struct Stage {
    /// The scene's own root, which never yaws.
    root: Entity,
    /// The standing scene's token; `None` while the screen is closed.
    token: Option<&'static str>,
    handle: Option<Handle<M2Model>>,
    spawned: bool,
    /// The scene's authored camera 0, captured at spawn.
    cam: Option<benilla_assets::PortraitCamera>,
    /// The character's seat, scene attachment 0, in the stage's frame ([`build_stage`]).
    char_spot: Vec3,
    char_facing: Quat,
    char_scale: f32,
    /// The stage's light buffer, kept across a close so a reopen reuses it.
    light: Option<bevy::render::render_resource::Buffer>,
    /// The character's materials relit onto that buffer.
    variants: HashMap<AssetId<WowModelMaterial>, Handle<WowModelMaterial>>,
    /// Bumped on every scene spawn: the character re-lights onto the fresh stage.
    rev: u64,
}

/// The five stages, in [`SHOWCASE`]'s order.
#[derive(Resource)]
pub(super) struct ShowcaseStages([Stage; SHOWCASE_SLOTS]);

/// Stand up the five idle booths (called from [`super::setup_booths`]), each the glue booth's
/// camera and booth on a layer of its own from [`SHOWCASE_LAYER_BASE`].
pub(super) fn spawn_showcase_booths(
    commands: &mut Commands,
    images: &mut Assets<Image>,
    portraits: &mut PortraitImages,
    booths: &mut Booths,
) {
    let stages = std::array::from_fn(|i| {
        let token = SHOWCASE_TOKENS[i];
        let layer_index = SHOWCASE_LAYER_BASE + i;
        let image = images.add(new_target_image(IDLE_SIZE));
        portraits
            .0
            .insert(token.to_string(), PortraitSource::Live(image.clone()));
        let layer = RenderLayers::layer(layer_index);
        let root = commands
            .spawn((Transform::IDENTITY, Visibility::Visible, layer.clone()))
            .id();
        commands.spawn((
            // The shared booth view shape; [`super`] says why every booth camera spawns through it.
            super::booth_view_shape(),
            Camera {
                order: -100 + layer_index as isize,
                // Transparent, as the glue booth's: the screen's page shows through until the
                // scene stands.
                clear_color: ClearColorConfig::Custom(Color::NONE),
                ..default()
            },
            bevy::camera::RenderTarget::Image(image.clone().into()),
            // The glue scene's FFX pass pair, as the glue booth's camera carries it.
            benilla_world::ffx_glow::FfxGlow::GLUE_SCENE,
            // Placeholder: `sync_showcase_scenes` aims it once the scene stands.
            Projection::from(PerspectiveProjection {
                fov: super::PORTRAIT_FOV,
                near: 0.02,
                far: 100.0,
                ..default()
            }),
            layer.clone(),
            BoothCam(token.to_string()),
        ));
        booths.0.insert(
            token.to_string(),
            Booth {
                layer: layer.clone(),
                root,
                target: image,
                baked: None,
                baked_guid: None,
                snap: None,
                shown: false,
                show_rev: 0,
                // `gate_booth_cameras` keeps the camera live while the screen is up.
                wake: 0,
                live: false,
                pending: Vec::new(),
                pending_since: None,
                pipes_settling: false,
                pipes_since: None,
                aspect: 1.0,
                rigged: false,
                parked: false,
                turn: super::Turn::default(),
            },
        );
        // The scene's own root; the character root yaws, this one never does.
        let scene_root = commands
            .spawn((Transform::IDENTITY, Visibility::Visible, layer))
            .id();
        Stage {
            root: scene_root,
            token: None,
            handle: None,
            spawned: false,
            cam: None,
            char_spot: Vec3::ZERO,
            char_facing: Quat::IDENTITY,
            char_scale: 1.0,
            light: None,
            variants: HashMap::new(),
            rev: 0,
        }
    });
    commands.insert_resource(ShowcaseStages(stages));
}

/// Drive the five stages off [`ShowcasePreview`]: while the screen is up, size each target to its
/// slice of the window, load and stand each scene, and hold each camera on its scene's camera 0,
/// framed for the slice ([`showcase_rig`]); when it closes, tear the scenes down, take their
/// buffers out of the mirror tables and shrink the targets back to idle.
#[allow(clippy::too_many_arguments)]
pub(super) fn sync_showcase_scenes(
    mut commands: Commands,
    preview: Res<ShowcasePreview>,
    stages: Option<ResMut<ShowcaseStages>>,
    booths: Res<Booths>,
    asset_server: Res<AssetServer>,
    m2s: Res<Assets<M2Model>>,
    booth_light: Res<BoothLight>,
    // What each stage's build writes ([`build_stage`]).
    mut kit: StageKit,
    mut images: ResMut<Assets<Image>>,
    mut cams: Query<(&BoothCam, &mut Transform, &mut Projection)>,
    window: Query<&Window, With<PrimaryWindow>>,
) {
    let Some(mut stages) = stages else {
        return;
    };
    // One slice of the window, in physical pixels, so the slice samples its target one to one.
    let slice = window.single().ok().map(|w| {
        (
            (w.physical_width() / SHOWCASE_SLOTS as u32).max(1),
            w.physical_height().max(1),
        )
    });
    for (i, stage) in stages.0.iter_mut().enumerate() {
        let Some(booth) = booths.0.get(SHOWCASE_TOKENS[i]) else {
            continue;
        };
        if !preview.active {
            if stage.token.is_some() {
                commands.entity(stage.root).despawn_related::<Children>();
                // Strip the rig state (pose buffer, player, palette slot): the root outlives it.
                super::clear_booth_rig(&mut commands, stage.root);
                // Out of the mirror tables, so the closed screen's buffers stop taking the
                // per-frame palette, tint and mat-anim uploads. A reopen re-registers each
                // buffer frames later, never in the frame it left, so the uploads that gate on
                // the table's size see both edges.
                kit.mirrors.0.remove(LIGHT_KEYS[i]);
                kit.tint_mirrors.0.remove(LIGHT_KEYS[i]);
                kit.anim_mirrors.0.remove(LIGHT_KEYS[i]);
                stage.token = None;
                stage.handle = None;
                stage.spawned = false;
                stage.cam = None;
                stage.char_spot = Vec3::ZERO;
                stage.char_facing = Quat::IDENTITY;
                stage.char_scale = 1.0;
                resize_target(&mut images, &booth.target, IDLE_SIZE, IDLE_SIZE);
            }
            continue;
        }
        if let Some((w, h)) = slice {
            resize_target(&mut images, &booth.target, w, h);
        }
        let token = scene_token(SHOWCASE[i].1);
        if stage.token != Some(token) {
            stage.token = Some(token);
            stage.handle = Some(asset_server.load(m2_url(&format!(
                "Interface\\Glues\\Models\\UI_{token}\\UI_{token}.mdx"
            ))));
            stage.spawned = false;
            stage.cam = None;
        }
        if !stage.spawned {
            let Some(handle) = stage.handle.clone() else {
                continue;
            };
            let Some(model) = m2s.get(&handle) else {
                continue; // still loading (the page shows in the meantime)
            };
            if booth_light.studio.buffer.is_none() {
                continue; // headless: no booth pipeline, no scene
            }
            // The create screen's lighting: the scene's authored rig, fogged.
            let rig = scene_rig(&model.lights);
            let built = build_stage(
                &mut commands,
                &mut kit,
                &mut stage.light,
                StageSpec {
                    handle: &handle,
                    model,
                    token,
                    root: stage.root,
                    layer: &booth.layer,
                    rig: &rig,
                    fog: true,
                    light_key: LIGHT_KEYS[i],
                    light_label: LIGHT_KEYS[i],
                    log: "class screen",
                },
            );
            stage.spawned = true;
            stage.cam = built.cam;
            stage.char_spot = built.char_spot;
            stage.char_facing = built.char_facing;
            stage.char_scale = built.char_scale;
            stage.rev += 1;
            info!(
                "class screen: {} stands in UI_{token} — camera {}, stage {:?}",
                crate::char_select::class_name(SHOWCASE[i].0),
                stage.cam.is_some(),
                stage.char_spot,
            );
            // Seat the character at the current facing (the booth's yaw path runs on a change).
            apply_yaw(
                &mut commands,
                booth.root,
                stage.char_spot,
                stage.char_facing,
                stage.char_scale,
                preview.yaw,
            );
        }
        if let Some(cam) = stage.cam {
            aim(
                &mut cams,
                SHOWCASE_TOKENS[i],
                &showcase_rig(&cam, stage.char_spot),
            );
        }
    }
}

/// A slice's camera: the scene's camera 0, its eye and pitch kept, its look-at panned about the
/// vertical onto the character ([`panned_target`]); the authored 4:3 vertical opening, closed by
/// [`SHOWCASE_ZOOM`], held at the slice's aspect so the narrow slice crops the scene's sides.
///
/// Not a sub-view of the authored frame: a sub-view culls against the whole frustum and then
/// crops, so the slice would pay for the full scene and frame the character off centre.
fn showcase_rig(cam: &benilla_assets::PortraitCamera, spot: Vec3) -> (Transform, Projection) {
    let target = panned_target(cam.eye, cam.target, spot);
    let fwd = (target - cam.eye).normalize_or_zero();
    let up = Quat::from_axis_angle(fwd, cam.roll) * Vec3::Y;
    (
        Transform::from_translation(cam.eye).looking_at(target, up),
        Projection::from(PerspectiveProjection {
            fov: diag_to_vert(cam.fov, GLUE_AUTHORED_ASPECT) * SHOWCASE_ZOOM,
            near: cam.near,
            // Kept generous, as the glue booth keeps it.
            far: cam.far.max(1000.0),
            ..default()
        }),
    )
}

/// The look-at that turns the camera at `eye` from `target` toward `spot` about the vertical
/// alone: the same reach and the same height difference, so the pitch holds, with the horizontal
/// direction onto `spot`. Every `UI_*` camera looks off its stage's spot by some degrees, which a
/// full-width frame absorbs and a slice would not. A degenerate input keeps `target`.
fn panned_target(eye: Vec3, target: Vec3, spot: Vec3) -> Vec3 {
    let fwd = target - eye;
    let reach = Vec2::new(fwd.x, fwd.z).length();
    let toward = Vec2::new(spot.x - eye.x, spot.z - eye.z);
    if reach <= f32::EPSILON || toward.length_squared() <= f32::EPSILON {
        return target;
    }
    let pan = toward.normalize() * reach;
    eye + Vec3::new(pan.x, fwd.y, pan.y)
}

/// Re-bake each slice's character when its look or its stage changes, relit onto that stage's
/// light, and spin all five to the shared yaw. A character stands only once its stage does: with
/// no scene there is no light to stand in and no camera to see it.
#[allow(clippy::too_many_arguments)]
pub(super) fn sync_showcase_booths(
    mut commands: Commands,
    preview: Res<ShowcasePreview>,
    bakes: Res<ShowcaseBakes>,
    mut booths: ResMut<Booths>,
    stages: Option<ResMut<ShowcaseStages>>,
    mut booth_light: ResMut<BoothLight>,
    mut materials: ResMut<Assets<WowModelMaterial>>,
    creatures: Option<Res<Creatures>>,
    anim_data: Option<Res<crate::creature_anim::AnimData>>,
    mut palettes: ResMut<benilla_world::rig_palette::RigPalettes>,
    mut tints: ResMut<benilla_world::instance_tint::InstanceTints>,
    mut last: Local<[Option<(u64, f32, u64)>; SHOWCASE_SLOTS]>,
) {
    let Some(mut stages) = stages else {
        return;
    };
    for (i, stage) in stages.0.iter_mut().enumerate() {
        let Some(booth) = booths.0.get_mut(SHOWCASE_TOKENS[i]) else {
            continue;
        };
        let bake = &bakes.0[i];
        let (spot, facing, scale) = if stage.spawned {
            (stage.char_spot, stage.char_facing, stage.char_scale)
        } else {
            (Vec3::ZERO, Quat::IDENTITY, 1.0)
        };
        let (last_rev, last_yaw, last_stage) = last[i].unwrap_or((u64::MAX, f32::NAN, u64::MAX));
        // An empty bake clears whether or not a stage stands.
        let staged = stage.spawned || bake.parts.is_empty();
        let rebake = staged && (last_rev != bake.revision || last_stage != stage.rev);
        let reyaw = last_yaw != preview.yaw;
        if !rebake && !reyaw {
            continue;
        }
        if rebake {
            if bake.parts.is_empty() {
                commands.entity(booth.root).despawn_related::<Children>();
                // The rig state on the root needs its own strip.
                super::clear_booth_rig(&mut commands, booth.root);
                booth.rigged = false;
                booth.parked = false;
                last[i] = Some((bake.revision, preview.yaw, stage.rev));
                apply_yaw(&mut commands, booth.root, spot, facing, scale, preview.yaw);
                continue;
            }
            // The rig comes from the display cache the builder gated on; if not ready, retry.
            let Some(creatures) = creatures.as_deref() else {
                continue;
            };
            let light = stage.light.clone();
            let baked = bake_character(
                &mut commands,
                booth,
                bake,
                creatures,
                light.as_ref().map(|buf| (buf, &mut stage.variants)),
                CharacterLane {
                    booth_light: &mut booth_light,
                    materials: &mut materials,
                    palettes: &mut palettes,
                    tints: &mut tints,
                    anim_data: anim_data.as_deref().map(|a| &a.0),
                },
                ("showcase rebake", SHOWCASE_TOKENS[i]),
            );
            if baked.is_none() {
                continue;
            }
            info!(
                "class screen: {} stands at {spot:?} ({} part(s))",
                crate::char_select::class_name(SHOWCASE[i].0),
                bake.parts.len()
            );
        }
        apply_yaw(&mut commands, booth.root, spot, facing, scale, preview.yaw);
        // A yaw-only pass must not record revisions it did not commit.
        last[i] = Some((
            if rebake { bake.revision } else { last_rev },
            preview.yaw,
            if rebake { stage.rev } else { last_stage },
        ));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The slices are the five classes the class screen offers, each once, left to right as the
    /// screen lays them out; each slice's booth token and light key are its own.
    #[test]
    fn the_five_slices_are_five_classes_with_their_own_booths() {
        let classes: Vec<u8> = SHOWCASE.iter().map(|(c, _)| *c).collect();
        assert_eq!(
            classes,
            [2, 8, 11, 9, 7],
            "Paladin, Mage, Druid, Warlock, Shaman"
        );
        assert_eq!(
            SHOWCASE.map(|(_, scene)| scene_token(scene)),
            ["Human", "Scourge", "NightElf", "Orc", "Tauren"],
        );
        for keys in [SHOWCASE_TOKENS, LIGHT_KEYS] {
            let mut sorted = keys.to_vec();
            sorted.sort_unstable();
            sorted.dedup();
            assert_eq!(sorted.len(), SHOWCASE_SLOTS, "{keys:?} must be distinct");
        }
        assert!(!SHOWCASE_TOKENS.contains(&super::super::GLUE_SLOT));
        assert!(!LIGHT_KEYS.contains(&"glue_scene"));
    }

    /// The pan turns the camera about the vertical alone: the pitch and the eye hold, and the
    /// look-at lands on the character's bearing.
    #[test]
    fn the_pan_keeps_the_pitch_and_faces_the_character() {
        let eye = Vec3::new(0.0, 2.0, 10.0);
        // Looking down the -Z axis and 10° down; the character stands off to the right.
        let target = eye + Vec3::new(0.0, -10f32.to_radians().tan() * 5.0, -5.0);
        let spot = Vec3::new(3.0, 0.0, 0.0);
        let panned = panned_target(eye, target, spot);
        let pitch = |v: Vec3| v.y.atan2(Vec2::new(v.x, v.z).length());
        assert!((pitch(panned - eye) - pitch(target - eye)).abs() < 1e-5);
        let bearing = |v: Vec3| v.x.atan2(-v.z);
        assert!((bearing(panned - eye) - bearing(spot - eye)).abs() < 1e-5);
        // Already on the bearing, nothing moves; a spot under the eye keeps the authored aim.
        let straight = panned_target(eye, target, Vec3::new(0.0, 0.0, 0.0));
        assert!((straight - target).length() < 1e-5);
        assert_eq!(
            panned_target(eye, target, Vec3::new(0.0, -4.0, 10.0)),
            target
        );
    }

    /// The slice holds the authored vertical opening, closed by the zoom, whatever its aspect.
    #[test]
    fn a_slice_holds_the_authored_vertical_opening_closed_by_the_zoom() {
        let cam = benilla_assets::PortraitCamera {
            eye: Vec3::new(0.0, 2.0, 10.0),
            target: Vec3::ZERO,
            roll: 0.0,
            fov: 0.9,
            near: 0.1,
            far: 30.0,
        };
        let (_, projection) = showcase_rig(&cam, Vec3::ZERO);
        let Projection::Perspective(p) = projection else {
            panic!("a slice is a perspective view");
        };
        assert!((p.fov - diag_to_vert(0.9, GLUE_AUTHORED_ASPECT) * SHOWCASE_ZOOM).abs() < 1e-6);
        assert!(p.far >= 1000.0, "kept generous, as the glue booth keeps it");
    }
}
