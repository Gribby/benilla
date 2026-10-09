//! Fork-only, not 1.12.1: the ARPG view's ground loot. A corpse's loot shows as items lying on the
//! ground around it, not in a loot window: each is a quality-coloured glow on the floor (grey,
//! white, green, blue, purple, orange), uncommon and better also a beam of light whose height
//! grows with the quality, and a name label over it. The gold lies there too, as its own drop.
//!
//! The loot stays on the corpse, where the server keeps it: an ARPG server sends what a corpse
//! holds for us (`SMSG_ARPG_LOOT`) when it dies and whenever that changes, and this lays the items
//! out around the corpse, always in the same place for the same loot slot. A left click on a
//! label or a glow walks to it and picks it up (`ClientCommand::ArpgLoot`); walking onto the gold
//! picks it up. Labels show within [`LABEL_RANGE`] of the player, and all of them with Alt held.
//! A unique (`super::uniques`) also wears the unique colour: a thin gold core inside a taller
//! beam, and a gold border round its label.
//! The loot filter (`arpgLootFilter`, the options window's ARPG View page) hides the labels of
//! the low qualities ([`label_shown`]); their glows stay, and Alt still shows them.
//! A corpse that despawns or streams out takes its drops with it.

use std::collections::HashMap;
use std::f32::consts::FRAC_PI_2;

use bevy::window::PrimaryWindow;

use benilla_protocol::messages::ArpgLootItem;
use benilla_protocol::{SessionEvent, SessionEventKind};
use benilla_ui::script::{JustifyH, JustifyV, Outline};
use benilla_world::view::WorldCamera;

use crate::net::{ClientCommand, GuidIndex, NetCommands, NetHandlerApp};
use crate::ui_pass::{UiQuad, UiQuadAppend, UiQuads};
use crate::ui_text::{layout_text_quads, FontSpec, Justify, UiFontAtlas};

use super::*;

/// The loot slot that names the gold, as the server reads it.
const GOLD_SLOT: u8 = benilla_protocol::messages::arpg::LOOT_SLOT_GOLD;
/// The art index of the gold: one past the item qualities.
const GOLD_ART: usize = 7;

/// Labels show for drops this close to the player, in yards; Alt shows every one on screen.
const LABEL_RANGE: f32 = 30.0;
/// The loot filter's setting: 0 shows every label, 1 hides grey ones, 2 grey and white, 3 everything
/// below blue. The gold always shows.
const CVAR_LOOT_FILTER: &str = "arpgLootFilter";
/// The filter with no setting: greys hidden.
const LOOT_FILTER_DEFAULT: u8 = 1;
/// A clicked drop is picked up once the player is this close to it, flat, in yards. The server
/// takes it from within 6 yd of the corpse plus both bodies' reach, and drops lie within
/// `RING_MIN + 2 × RING_STEP` of it, so this is always inside the server's reach.
const PICK_RANGE: f32 = 1.5;
/// Walking this close to the gold picks it up, in yards.
const GOLD_RANGE: f32 = 1.2;
/// A cursor on the ground this close to a drop hovers it, in yards.
const HOVER_RANGE: f32 = 0.8;
/// After a pick-up is sent, the drop is not asked for again for this long, in seconds: the
/// server's answer (the corpse's new list) normally removes it first.
const PICK_RETRY: f64 = 1.5;

/// The ring the drops lie on around the corpse, in yards from its feet.
const RING_MIN: f32 = 1.1;
const RING_STEP: f32 = 0.45;
/// The gold lies this far from the corpse.
const GOLD_RING: f32 = 0.7;
/// Successive slots step round the corpse by the golden angle, so few ever overlap.
const GOLDEN_ANGLE: f32 = 2.399_963;

/// The glow's lift off the ground, against z-fighting, in yards.
const GLOW_LIFT: f32 = 0.06;
/// The beam's radius, in yards.
const BEAM_RADIUS: f32 = 0.09;
/// A unique's gold beam core: this share of the beam's radius, this much taller, and never
/// shorter than [`UNIQUE_CORE_MIN`] yards.
const UNIQUE_CORE_RADIUS: f32 = 0.45;
const UNIQUE_CORE_TALLER: f32 = 1.3;
const UNIQUE_CORE_MIN: f32 = 6.0;
/// The art index whose colour is the unique colour: the Artifact quality's.
const UNIQUE_ART: usize = 6;
/// A unique's label border, in screen pixels.
const UNIQUE_BORDER: f32 = 1.5;
/// The label sits this far above the drop, in yards.
const LABEL_LIFT: f32 = 0.5;

/// The append lane's keys for the labels: over the floating combat text, under the bubbles.
const Z_LABEL_BG: u64 = 16;
const Z_LABEL_TEXT: u64 = 17;

/// The quality colours, sRGB: the item tooltip's (`GetItemQualityColor`), then the gold's.
const QUALITY_RGB: [[f32; 3]; 8] = [
    [0.616, 0.616, 0.616], // 0 poor, grey
    [1.0, 1.0, 1.0],       // 1 common, white
    [0.118, 1.0, 0.0],     // 2 uncommon, green
    [0.0, 0.439, 0.867],   // 3 rare, blue
    [0.639, 0.208, 0.933], // 4 epic, purple
    [1.0, 0.502, 0.0],     // 5 legendary, orange
    [0.902, 0.8, 0.502],   // 6 artifact
    [1.0, 0.82, 0.0],      // the gold
];

/// One drop's look, by its art index ([`art_index`]): the glow's radius and alpha, and the beam's
/// height and alpha (no beam at height 0). Grey and white glow faintly with no beam, so a pile of
/// vendor trash stays quiet; each step up the qualities glows brighter and beams taller.
#[derive(Clone, Copy, Debug, PartialEq)]
struct Look {
    glow_radius: f32,
    glow_alpha: f32,
    beam_height: f32,
    beam_alpha: f32,
}

fn look(art: usize) -> Look {
    let (glow_radius, glow_alpha, beam_height, beam_alpha) = match art {
        0 => (0.35, 0.30, 0.0, 0.0),
        1 => (0.40, 0.45, 0.0, 0.0),
        2 => (0.50, 0.55, 2.5, 0.30),
        3 => (0.55, 0.60, 4.5, 0.35),
        4 => (0.60, 0.65, 6.5, 0.40),
        5 | 6 => (0.65, 0.70, 9.0, 0.45),
        _ => (0.30, 0.50, 0.0, 0.0), // the gold
    };
    Look {
        glow_radius,
        glow_alpha,
        beam_height,
        beam_alpha,
    }
}

/// The art index of an item's quality, clamped to the table.
fn art_index(quality: u8) -> usize {
    usize::from(quality).min(6)
}

/// A drop by the corpse it lies by and its loot slot ([`GOLD_SLOT`] for the gold).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(in crate::player) struct LootKey {
    pub(in crate::player) corpse: u64,
    pub(in crate::player) slot: u8,
}

/// What a drop is: an item, or the corpse's gold.
#[derive(Clone, Copy, Debug, PartialEq)]
enum DropKind {
    Item(ArpgLootItem),
    Gold(u32),
}

/// One drop on the ground.
struct GroundDrop {
    kind: DropKind,
    /// Where it lies: its glow's centre at the ground.
    pos: Vec3,
    /// The glow and the beam.
    visuals: Vec<Entity>,
    /// When a pick-up was last sent for it.
    sent_at: Option<f64>,
    /// Set down on the ground under it yet ([`settle_drops`]).
    settled: bool,
}

/// Every drop on the ground, by key.
#[derive(Resource, Default)]
struct GroundLoot(HashMap<LootKey, GroundDrop>);

/// The shared meshes, and the materials by art index.
#[derive(Clone)]
struct Art {
    glow_mesh: Handle<Mesh>,
    beam_mesh: Handle<Mesh>,
    glow_mats: Vec<Handle<StandardMaterial>>,
    beam_mats: Vec<Handle<StandardMaterial>>,
}

/// The shared art, made on first use.
#[derive(Resource, Default)]
struct LootArt(Option<Art>);

/// Last frame's label boxes, in screen pixels, for the hover test.
#[derive(Resource, Default)]
struct LootLabels(Vec<(Rect, LootKey)>);

/// A drop's glow, which breathes gently so the ground does not look painted.
#[derive(Component)]
struct LootGlow {
    radius: f32,
    phase: f32,
}

/// A corpse's loot list, as the server sent it ([`SessionEvent::ArpgLoot`]).
#[derive(Message, Clone, Debug)]
pub(crate) struct ArpgLootList {
    corpse: u64,
    gold: u32,
    items: Vec<ArpgLootItem>,
}

/// The wire's half, registered whatever the view so every session event kind has its owner: a
/// stock server never sends the message.
pub(super) fn register_net(app: &mut App) {
    app.add_message::<ArpgLootList>()
        .net_handler(SessionEventKind::ArpgLoot, on_arpg_loot);
}

fn on_arpg_loot(In(ev): In<SessionEvent>, mut out: MessageWriter<ArpgLootList>) {
    if let SessionEvent::ArpgLoot {
        corpse,
        gold,
        items,
    } = ev
    {
        info!(
            "arpg: ground loot for {corpse:#x}: {} item(s), {gold} copper",
            items.len()
        );
        out.write(ArpgLootList {
            corpse,
            gold,
            items,
        });
    }
}

/// The view's half: the drops, their look, the labels, the hover, the walk and the pick-up.
pub(super) fn plugin(app: &mut App) {
    app.init_resource::<GroundLoot>()
        .init_resource::<LootArt>()
        .init_resource::<LootLabels>()
        .add_systems(
            Update,
            (
                (apply_lists, settle_drops).chain(),
                ask_for_lists.in_set(crate::char_select::InWorldGated),
                breathe,
                hover_drops.before(super::pin_view),
                walk_and_pick
                    .after(super::pin_view)
                    .before(control)
                    .in_set(crate::char_select::InWorldGated),
            ),
        )
        .add_systems(Update, draw_labels.in_set(UiQuadAppend));
}

/// Where the drop in `slot` lies around a corpse at `corpse_pos`: a ring position fixed by the
/// slot, so a list that loses an item leaves the others where they were. `seed` turns the whole
/// layout per corpse, so two corpses' drops do not line up.
fn drop_position(corpse_pos: Vec3, slot: u8, seed: u64) -> Vec3 {
    let turn = (seed % 6283) as f32 / 1000.0;
    let (angle, radius) = if slot == GOLD_SLOT {
        (turn + std::f32::consts::PI, GOLD_RING)
    } else {
        (
            turn + f32::from(slot) * GOLDEN_ANGLE,
            RING_MIN + RING_STEP * f32::from(slot % 3),
        )
    };
    corpse_pos + Vec3::new(angle.cos() * radius, 0.0, angle.sin() * radius)
}

/// Gold, silver and copper, as the money frame writes them: `1g 20s 5c`, zero parts left out.
fn coin_text(copper: u32) -> String {
    let (g, s, c) = (copper / 10_000, copper / 100 % 100, copper % 100);
    let mut parts = Vec::new();
    if g > 0 {
        parts.push(format!("{g}g"));
    }
    if s > 0 {
        parts.push(format!("{s}s"));
    }
    if c > 0 || parts.is_empty() {
        parts.push(format!("{c}c"));
    }
    parts.join(" ")
}

/// The shared art, made once.
fn art<'a>(
    art: &'a mut LootArt,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
) -> &'a Art {
    art.0.get_or_insert_with(|| {
        let mut material = |rgb: [f32; 3], alpha: f32| {
            materials.add(StandardMaterial {
                base_color: Color::srgba(rgb[0], rgb[1], rgb[2], alpha),
                // The raid marks' world-pass state: unlit, depth-tested, blended, two-sided.
                unlit: true,
                alpha_mode: AlphaMode::Blend,
                cull_mode: None,
                ..default()
            })
        };
        let glows = (0..QUALITY_RGB.len())
            .map(|i| material(QUALITY_RGB[i], look(i).glow_alpha))
            .collect();
        let beams = (0..QUALITY_RGB.len())
            .map(|i| material(QUALITY_RGB[i], look(i).beam_alpha))
            .collect();
        Art {
            glow_mesh: meshes.add(Circle::new(1.0)),
            beam_mesh: meshes.add(Cylinder::new(1.0, 1.0)),
            glow_mats: glows,
            beam_mats: beams,
        }
    })
}

/// Lay out each list that arrived, and clear the drops of corpses that are gone.
#[allow(clippy::too_many_arguments)]
fn apply_lists(
    mut lists: MessageReader<ArpgLootList>,
    index: Res<GuidIndex>,
    transforms: Query<&Transform>,
    mut ground: ResMut<GroundLoot>,
    mut loot_art: ResMut<LootArt>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    uniques: Option<Res<super::ArpgUniques>>,
    mut commands: Commands,
) {
    for list in lists.read() {
        let corpse_pos = index
            .0
            .get(&list.corpse)
            .and_then(|e| transforms.get(*e).ok())
            .map(|t| t.translation);
        let mut wanted: Vec<(LootKey, DropKind)> = list
            .items
            .iter()
            .map(|item| {
                (
                    LootKey {
                        corpse: list.corpse,
                        slot: item.slot,
                    },
                    DropKind::Item(*item),
                )
            })
            .collect();
        if list.gold > 0 {
            wanted.push((
                LootKey {
                    corpse: list.corpse,
                    slot: GOLD_SLOT,
                },
                DropKind::Gold(list.gold),
            ));
        }
        // Gone from the list (taken), or the corpse is unknown here: off the ground.
        let stale: Vec<LootKey> = ground
            .0
            .iter()
            .filter(|(k, d)| {
                k.corpse == list.corpse
                    && (corpse_pos.is_none()
                        || !wanted.iter().any(|(w, kind)| w == *k && *kind == d.kind))
            })
            .map(|(k, _)| *k)
            .collect();
        for key in stale {
            if let Some(drop) = ground.0.remove(&key) {
                despawn_all(&mut commands, &drop.visuals);
            }
        }
        let Some(corpse_pos) = corpse_pos else {
            continue;
        };
        let Art {
            glow_mesh,
            beam_mesh,
            glow_mats,
            beam_mats,
        } = art(&mut loot_art, &mut meshes, &mut materials).clone();
        for (key, kind) in wanted {
            if ground.0.contains_key(&key) {
                continue;
            }
            let pos = drop_position(corpse_pos, key.slot, list.corpse);
            let art_at = match kind {
                DropKind::Item(item) => art_index(item.quality),
                DropKind::Gold(_) => GOLD_ART,
            };
            let lk = look(art_at);
            let mut visuals = vec![commands
                .spawn((
                    Mesh3d(glow_mesh.clone()),
                    MeshMaterial3d(glow_mats[art_at].clone()),
                    Transform::from_translation(pos + Vec3::Y * GLOW_LIFT)
                        .with_rotation(Quat::from_rotation_x(-FRAC_PI_2))
                        .with_scale(Vec3::splat(lk.glow_radius)),
                    LootGlow {
                        radius: lk.glow_radius,
                        phase: f32::from(key.slot) * 0.7,
                    },
                ))
                .id()];
            if lk.beam_height > 0.0 {
                visuals.push(
                    commands
                        .spawn((
                            Mesh3d(beam_mesh.clone()),
                            MeshMaterial3d(beam_mats[art_at].clone()),
                            Transform::from_translation(pos + Vec3::Y * (lk.beam_height * 0.5))
                                .with_scale(Vec3::new(BEAM_RADIUS, lk.beam_height, BEAM_RADIUS)),
                        ))
                        .id(),
                );
            }
            if let Some(core) = unique_core(&kind, lk, uniques.as_deref()) {
                visuals.push(
                    commands
                        .spawn((
                            Mesh3d(beam_mesh.clone()),
                            MeshMaterial3d(beam_mats[UNIQUE_ART].clone()),
                            Transform::from_translation(pos + Vec3::Y * (core * 0.5)).with_scale(
                                Vec3::new(
                                    BEAM_RADIUS * UNIQUE_CORE_RADIUS,
                                    core,
                                    BEAM_RADIUS * UNIQUE_CORE_RADIUS,
                                ),
                            ),
                        ))
                        .id(),
                );
            }
            ground.0.insert(
                key,
                GroundDrop {
                    kind,
                    pos,
                    visuals,
                    sent_at: None,
                    settled: false,
                },
            );
        }
    }
    // A corpse that despawned or streamed out takes its drops with it.
    let gone: Vec<LootKey> = ground
        .0
        .keys()
        .filter(|k| !index.0.contains_key(&k.corpse))
        .copied()
        .collect();
    for key in gone {
        if let Some(drop) = ground.0.remove(&key) {
            despawn_all(&mut commands, &drop.visuals);
        }
    }
}

/// Whether `kind` is a unique, by the server's table.
fn is_unique(kind: &DropKind, uniques: Option<&super::ArpgUniques>) -> bool {
    match kind {
        DropKind::Item(item) => uniques.is_some_and(|u| u.line(item.item_id).is_some()),
        DropKind::Gold(_) => false,
    }
}

/// The height of a unique's gold beam core over a drop of look `lk`, or `None` for a drop that
/// is not a unique.
fn unique_core(kind: &DropKind, lk: Look, uniques: Option<&super::ArpgUniques>) -> Option<f32> {
    is_unique(kind, uniques).then(|| (lk.beam_height * UNIQUE_CORE_TALLER).max(UNIQUE_CORE_MIN))
}

fn despawn_all(commands: &mut Commands, entities: &[Entity]) {
    for e in entities {
        if let Ok(mut ec) = commands.get_entity(*e) {
            ec.despawn();
        }
    }
}

/// Ask for the list of each lootable corpse in sight that has no drops here and has not been asked
/// about since it came into sight: the server sends a list at the kill, which a client that was
/// not there then (walked back, relogged, a group member's kill out of sight) never saw.
fn ask_for_lists(
    units: Query<(
        &crate::net::Guid,
        &crate::net::NetEntity,
        &crate::net::ObjectStore,
    )>,
    ground: Res<GroundLoot>,
    net: Res<NetCommands>,
    mut asked: Local<std::collections::HashSet<u64>>,
) {
    let mut lootable = std::collections::HashSet::new();
    for (guid, entity, store) in &units {
        if entity.kind != benilla_protocol::EntityKind::Unit || !store.0.unit_lootable() {
            continue;
        }
        lootable.insert(guid.0);
        if asked.contains(&guid.0) || ground.0.keys().any(|k| k.corpse == guid.0) {
            continue;
        }
        asked.insert(guid.0);
        let _ = net.0.send(ClientCommand::ArpgLootQuery { corpse: guid.0 });
    }
    // Out of sight or no longer lootable: a corpse that comes back is asked about again.
    asked.retain(|g| lootable.contains(g));
}

/// How far above a new drop its ground ray starts, and how far down it looks, in yards: above the
/// corpse's feet so a slope rising toward the drop is found, and under a ceiling indoors.
const SETTLE_UP: f32 = 1.5;
const SETTLE_DOWN: f32 = 4.0;

/// Set each new drop down on the ground under it, so on a slope it lies on the hill rather than
/// floating at the corpse's height or sinking into it; its glow and beam move with it.
fn settle_drops(
    spatial: avian3d::prelude::SpatialQuery,
    occluders: Query<(), With<benilla_world::collision::PickOccluder>>,
    mut ground: ResMut<GroundLoot>,
    mut transforms: Query<&mut Transform>,
) {
    for drop in ground.0.values_mut().filter(|d| !d.settled) {
        drop.settled = true;
        let Some(hit) = spatial.cast_ray_predicate(
            drop.pos + Vec3::Y * SETTLE_UP,
            Dir3::NEG_Y,
            SETTLE_UP + SETTLE_DOWN,
            true,
            &benilla_world::collision::WorldCollision::body_filter(),
            &|e| occluders.contains(e),
        ) else {
            continue;
        };
        let dy = SETTLE_UP - hit.distance;
        drop.pos.y += dy;
        for e in &drop.visuals {
            if let Ok(mut tf) = transforms.get_mut(*e) {
                tf.translation.y += dy;
            }
        }
    }
}

/// The glows breathe: a slow swell of a tenth of their size.
fn breathe(time: Res<Time>, mut glows: Query<(&LootGlow, &mut Transform)>) {
    let t = time.elapsed_secs();
    for (glow, mut tf) in &mut glows {
        let s = glow.radius * (1.0 + 0.1 * (t * 2.2 + glow.phase).sin());
        tf.scale = Vec3::splat(s);
    }
}

/// The drop under the cursor: a label the cursor is on, else a glow the cursor's ground point is
/// near. Nothing over a UI panel.
fn hover_drops(
    ground: Res<GroundLoot>,
    labels: Res<LootLabels>,
    window: Query<&Window, With<PrimaryWindow>>,
    occlusion: Option<Res<crate::target::PickOcclusion>>,
    over_panel: Option<Res<crate::ui_script::PointerOverUiPanel>>,
    mut rig: ResMut<CameraControl>,
) {
    let mut hovered = None;
    let mut on_label = false;
    if !over_panel.as_ref().is_some_and(|p| p.0) {
        let cursor = window.single().ok().and_then(|w| w.cursor_position());
        if let Some(cursor) = cursor {
            // The last drawn label on top wins, as it painted over the others.
            hovered = labels
                .0
                .iter()
                .rev()
                .find(|(rect, key)| rect.contains(cursor) && ground.0.contains_key(key))
                .map(|(_, key)| *key);
            on_label = hovered.is_some();
        }
        if hovered.is_none() {
            if let Some(point) = occlusion.as_ref().and_then(|o| o.point) {
                hovered = nearest_drop(&ground, point, HOVER_RANGE);
            }
        }
    }
    if rig.arpg.over_loot != hovered || rig.arpg.over_loot_label != on_label {
        rig.arpg.over_loot = hovered;
        rig.arpg.over_loot_label = on_label;
    }
}

/// The drop nearest `point` on the ground within `reach` yards, if any.
fn nearest_drop(ground: &GroundLoot, point: Vec3, reach: f32) -> Option<LootKey> {
    ground
        .0
        .iter()
        .map(|(k, d)| (k, flat_distance(point, d.pos)))
        .filter(|(_, dist)| *dist <= reach)
        .min_by(|a, b| a.1.total_cmp(&b.1))
        .map(|(k, _)| *k)
}

/// Walk to the clicked drop and pick it up once in reach, and pick up the gold underfoot.
fn walk_and_pick(
    player: Res<Player>,
    time: Res<Time>,
    net: Res<NetCommands>,
    mut ground: ResMut<GroundLoot>,
    mut rig: ResMut<CameraControl>,
) {
    let now = time.elapsed_secs_f64();
    let pos = player.pos;
    let mouse = &mut rig.arpg;
    let send = |key: LootKey, drop: &mut GroundDrop| {
        if drop.sent_at.is_some_and(|at| now - at < PICK_RETRY) {
            return;
        }
        drop.sent_at = Some(now);
        let _ = net.0.send(ClientCommand::ArpgLoot {
            corpse: key.corpse,
            slot: key.slot,
        });
    };
    if let Some(key) = mouse.loot_goal {
        // A fresh click asks again, even inside the retry window (after full bags, say).
        if !mouse.loot_walking {
            if let Some(drop) = ground.0.get_mut(&key) {
                drop.sent_at = None;
            }
        }
        match ground.0.get_mut(&key) {
            None => mouse.end_loot_walk(),
            Some(drop) if flat_distance(pos, drop.pos) <= PICK_RANGE => {
                send(key, drop);
                mouse.end_loot_walk();
                mouse.goal = None;
            }
            // A key, a stuck walk or another click ended the walk there.
            Some(_) if mouse.loot_walking && mouse.goal.is_none() => mouse.end_loot_walk(),
            Some(drop) => {
                mouse.goal = Some(drop.pos);
                mouse.loot_walking = true;
            }
        }
    }
    for (key, drop) in ground.0.iter_mut() {
        if matches!(drop.kind, DropKind::Gold(_)) && flat_distance(pos, drop.pos) <= GOLD_RANGE {
            send(*key, drop);
        }
    }
}

/// The label's text: the item's name (with its count), or the coins.
fn label_text(kind: &DropKind, name: Option<&str>) -> String {
    match kind {
        DropKind::Gold(copper) => coin_text(*copper),
        DropKind::Item(item) => {
            let name = name.unwrap_or("...");
            if item.count > 1 {
                format!("{name} ({})", item.count)
            } else {
                name.to_string()
            }
        }
    }
}

/// Whether a drop's label shows under loot filter `filter` (see [`CVAR_LOOT_FILTER`]); Alt held
/// shows everything.
fn label_shown(kind: &DropKind, filter: u8, alt: bool) -> bool {
    match kind {
        DropKind::Gold(_) => true,
        DropKind::Item(item) => alt || item.quality >= filter.min(3),
    }
}

/// Draw the labels, nearest the camera last so they paint on top, pushed up off each other so
/// a pile of drops stays readable, and remember their boxes for the hover.
#[allow(clippy::too_many_arguments)]
fn draw_labels(
    ground: Res<GroundLoot>,
    player: Res<Player>,
    keys: Res<ButtonInput<KeyCode>>,
    items: Res<crate::items::Items>,
    net: Res<NetCommands>,
    rig: Res<CameraControl>,
    (cvars, uniques): (
        Option<Res<crate::cvars::Cvars>>,
        Option<Res<super::ArpgUniques>>,
    ),
    camera: Query<(&Camera, &Transform), With<WorldCamera>>,
    mut atlas: Option<ResMut<UiFontAtlas>>,
    mut quads: ResMut<UiQuads>,
    mut labels: ResMut<LootLabels>,
) {
    labels.0.clear();
    let (Ok((cam, pose)), Some(atlas)) = (camera.single(), atlas.as_mut()) else {
        return;
    };
    let cam_tf = GlobalTransform::from(*pose);
    let Some(viewport) = cam.logical_viewport_size() else {
        return;
    };
    let all = keys.any_pressed([KeyCode::AltLeft, KeyCode::AltRight]);
    let filter = cvars
        .as_ref()
        .and_then(|c| c.num(CVAR_LOOT_FILTER))
        .filter(|v| v.is_finite())
        .map_or(LOOT_FILTER_DEFAULT, |v| v.round().clamp(0.0, 3.0) as u8);
    let px = (viewport.y / 768.0 * 13.0).clamp(11.0, 24.0);
    let pad = Vec2::new(px * 0.4, px * 0.2);

    // Each label's anchor on screen, then laid out bottom of the screen first: the drops nearest
    // the camera claim their spot first and the far ones move up past them.
    let mut shown: Vec<(LootKey, &GroundDrop, Vec2)> = ground
        .0
        .iter()
        .filter(|(_, d)| all || flat_distance(player.pos, d.pos) <= LABEL_RANGE)
        .filter(|(_, d)| label_shown(&d.kind, filter, all))
        .filter_map(|(k, d)| {
            crate::ui_pass::project_overlay(cam, &cam_tf, d.pos + Vec3::Y * LABEL_LIFT, viewport)
                .map(|screen| (*k, d, screen))
        })
        .collect();
    shown.sort_by(|a, b| b.2.y.total_cmp(&a.2.y));

    let mut placed: Vec<Rect> = Vec::new();
    for (key, ground_drop, screen) in shown {
        let name = match ground_drop.kind {
            DropKind::Item(item) => items
                .template(item.item_id, 0, &net)
                .map(|info| info.name.as_str()),
            DropKind::Gold(_) => None,
        };
        let text = label_text(&ground_drop.kind, name);
        let art_at = match ground_drop.kind {
            DropKind::Item(item) => art_index(item.quality),
            DropKind::Gold(_) => GOLD_ART,
        };
        let rgb = QUALITY_RGB[art_at];
        let mut e = atlas.lock();
        let mut glyphs = layout_text_quads(
            &mut e,
            &text,
            Rect::from_center_size(screen, Vec2::ZERO),
            [rgb[0], rgb[1], rgb[2], 1.0],
            Justify {
                h: JustifyH::Center,
                v: JustifyV::Middle,
            },
            Z_LABEL_TEXT,
            FontSpec {
                path: None,
                height: Some(px),
                outline: Outline::None,
                alpha_gradient: None,
            },
            crate::ui_text::TextSeat::Exact,
        );
        std::mem::drop(e);
        let Some(ink) = glyphs.iter().map(|q| q.rect).reduce(|a, b| a.union(b)) else {
            continue;
        };
        let mut boxed = Rect {
            min: ink.min - pad,
            max: ink.max + pad,
        };
        // Up past any label already placed that it would cover.
        for _ in 0..24 {
            let Some(hit) = placed.iter().find(|r| !r.intersect(boxed).is_empty()) else {
                break;
            };
            let up = boxed.max.y - hit.min.y + 1.0;
            boxed.min.y -= up;
            boxed.max.y -= up;
        }
        let shift = boxed.min + pad - ink.min;
        for q in &mut glyphs {
            q.rect.min += shift;
            q.rect.max += shift;
        }
        let hovered = rig.arpg.over_loot == Some(key);
        // A unique's gold border, under the box.
        if is_unique(&ground_drop.kind, uniques.as_deref()) {
            let g = super::UNIQUE_RGB;
            quads.overlays.push(UiQuad {
                rect: Rect {
                    min: boxed.min - Vec2::splat(UNIQUE_BORDER),
                    max: boxed.max + Vec2::splat(UNIQUE_BORDER),
                },
                z_key: Z_LABEL_BG,
                color: [g[0], g[1], g[2], 0.95],
                ..default()
            });
        }
        quads.overlays.push(UiQuad {
            rect: boxed,
            z_key: Z_LABEL_BG,
            color: if hovered {
                [0.22, 0.22, 0.22, 0.9]
            } else {
                [0.0, 0.0, 0.0, 0.65]
            },
            ..default()
        });
        quads.overlays.append(&mut glyphs);
        placed.push(boxed);
        labels.0.push((boxed, key));
    }
}

/// The distance between two points on the ground, ignoring height.
fn flat_distance(a: Vec3, b: Vec3) -> f32 {
    Vec2::new(a.x - b.x, a.z - b.z).length()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The ring's outer edge: the third step out.
    const RING_MAX: f32 = RING_MIN + 2.0 * RING_STEP;

    fn item(slot: u8, quality: u8) -> ArpgLootItem {
        ArpgLootItem {
            slot,
            item_id: 100 + u32::from(slot),
            display_id: 0,
            quality,
            count: 1,
        }
    }

    /// A bare app with the corpse at `CORPSE` and the list system.
    fn lists_app() -> (App, Entity) {
        const CORPSE: u64 = 0xF130_0000_0000_0042;
        let mut app = App::new();
        app.init_resource::<Assets<Mesh>>()
            .init_resource::<Assets<StandardMaterial>>()
            .init_resource::<GroundLoot>()
            .init_resource::<LootArt>()
            .add_message::<ArpgLootList>()
            .add_systems(Update, apply_lists);
        let corpse = app
            .world_mut()
            .spawn(Transform::from_xyz(5.0, 1.0, 5.0))
            .id();
        let mut index = GuidIndex::default();
        index.0.insert(CORPSE, corpse);
        app.insert_resource(index);
        (app, corpse)
    }

    fn send(app: &mut App, gold: u32, items: Vec<ArpgLootItem>) {
        app.world_mut().write_message(ArpgLootList {
            corpse: 0xF130_0000_0000_0042,
            gold,
            items,
        });
        app.update();
    }

    fn visuals(app: &mut App) -> usize {
        app.world_mut().query::<&Mesh3d>().iter(app.world()).count()
    }

    #[test]
    fn a_list_lays_its_drops_out_and_a_shorter_one_takes_the_gone_ones_away() {
        let (mut app, _) = lists_app();
        // A grey item (glow only), a blue one (glow and beam), and gold (glow only).
        send(&mut app, 120, vec![item(0, 0), item(1, 3)]);
        assert_eq!(app.world().resource::<GroundLoot>().0.len(), 3);
        assert_eq!(visuals(&mut app), 4);
        let blue = LootKey {
            corpse: 0xF130_0000_0000_0042,
            slot: 1,
        };
        let before = app.world().resource::<GroundLoot>().0[&blue].pos;
        // The grey item and the gold were taken: the blue one stays where it lay.
        send(&mut app, 0, vec![item(1, 3)]);
        let ground = app.world().resource::<GroundLoot>();
        assert_eq!(ground.0.len(), 1);
        assert_eq!(ground.0[&blue].pos, before);
        app.update();
        assert_eq!(visuals(&mut app), 2);
        // Nothing left.
        send(&mut app, 0, vec![]);
        assert!(app.world().resource::<GroundLoot>().0.is_empty());
    }

    #[test]
    fn a_corpse_that_is_gone_takes_its_drops_with_it() {
        let (mut app, _) = lists_app();
        send(&mut app, 5, vec![item(0, 4)]);
        assert_eq!(app.world().resource::<GroundLoot>().0.len(), 2);
        app.world_mut().resource_mut::<GuidIndex>().0.clear();
        app.update();
        assert!(app.world().resource::<GroundLoot>().0.is_empty());
        app.update();
        assert_eq!(visuals(&mut app), 0);
    }

    #[test]
    fn coins_read_like_the_money_frame() {
        assert_eq!(coin_text(0), "0c");
        assert_eq!(coin_text(5), "5c");
        assert_eq!(coin_text(12_005), "1g 20s 5c");
        assert_eq!(coin_text(30_000), "3g");
        assert_eq!(coin_text(150), "1s 50c");
    }

    #[test]
    fn a_drop_keeps_its_place_and_lies_on_the_ring() {
        let corpse = Vec3::new(10.0, 2.0, -4.0);
        for slot in 0..8u8 {
            let at = drop_position(corpse, slot, 0xABCD);
            assert_eq!(at, drop_position(corpse, slot, 0xABCD));
            let r = flat_distance(at, corpse);
            assert!(
                (RING_MIN - 1e-4..=RING_MAX + 1e-4).contains(&r),
                "slot {slot} at {r}"
            );
            assert_eq!(at.y, corpse.y);
        }
        assert!(
            (flat_distance(drop_position(corpse, GOLD_SLOT, 1), corpse) - GOLD_RING).abs() < 1e-4
        );
        // Inside the server's reach of the corpse (6 yd plus both bodies' reach).
        const { assert!(RING_MAX + PICK_RANGE < 6.0) };
    }

    #[test]
    fn better_drops_glow_brighter_and_beam_taller() {
        // Grey and white: a faint glow, no beam; uncommon up: a beam, taller each step.
        assert_eq!(look(0).beam_height, 0.0);
        assert_eq!(look(1).beam_height, 0.0);
        assert!(look(0).glow_alpha > 0.0 && look(1).glow_alpha > look(0).glow_alpha);
        for q in 2..5 {
            assert!(look(q + 1).beam_height > look(q).beam_height);
            assert!(look(q + 1).glow_alpha >= look(q).glow_alpha);
        }
        assert_eq!(art_index(9), 6);
    }

    #[test]
    fn labels_name_the_item_with_its_count_or_the_coins() {
        let item = ArpgLootItem {
            slot: 0,
            item_id: 1,
            display_id: 0,
            quality: 1,
            count: 3,
        };
        assert_eq!(
            label_text(&DropKind::Item(item), Some("Wolf Meat")),
            "Wolf Meat (3)"
        );
        assert_eq!(label_text(&DropKind::Item(item), None), "... (3)");
        assert_eq!(label_text(&DropKind::Gold(250), None), "2s 50c");
    }

    #[test]
    fn a_unique_gets_a_gold_core_taller_than_its_beam() {
        let item = |item_id| {
            DropKind::Item(ArpgLootItem {
                slot: 0,
                item_id,
                display_id: 0,
                quality: 3,
                count: 1,
            })
        };
        let mut uniques = super::super::ArpgUniques::default();
        uniques
            .0
            .insert(5201, "Fireball launches 1 extra fireball.".into());
        let lk = look(3);
        let core = unique_core(&item(5201), lk, Some(&uniques)).unwrap();
        assert!(core > lk.beam_height && core >= UNIQUE_CORE_MIN);
        assert_eq!(unique_core(&item(25), lk, Some(&uniques)), None);
        assert_eq!(unique_core(&item(5201), lk, None), None);
        assert!(!is_unique(&DropKind::Gold(5), Some(&uniques)));
    }

    #[test]
    fn the_filter_hides_low_labels_but_never_the_gold_and_alt_shows_all() {
        let of = |quality| {
            DropKind::Item(ArpgLootItem {
                slot: 0,
                item_id: 1,
                display_id: 0,
                quality,
                count: 1,
            })
        };
        // Off: everything.
        assert!(label_shown(&of(0), 0, false));
        // Greys hidden: white and up show.
        assert!(!label_shown(&of(0), 1, false));
        assert!(label_shown(&of(1), 1, false));
        // Blue and better only.
        assert!(!label_shown(&of(2), 3, false));
        assert!(label_shown(&of(3), 3, false));
        assert!(label_shown(&of(5), 3, false));
        // The gold always, and Alt overrides.
        assert!(label_shown(&DropKind::Gold(5), 3, false));
        assert!(label_shown(&of(0), 3, true));
    }

    #[test]
    fn the_cursor_hovers_the_nearest_drop_in_reach() {
        let mut ground = GroundLoot::default();
        for (slot, x) in [(0u8, 0.0), (1, 1.0)] {
            ground.0.insert(
                LootKey { corpse: 1, slot },
                GroundDrop {
                    kind: DropKind::Gold(1),
                    pos: Vec3::new(x, 0.0, 0.0),
                    visuals: Vec::new(),
                    sent_at: None,
                    settled: true,
                },
            );
        }
        let near = nearest_drop(&ground, Vec3::new(0.8, 5.0, 0.0), HOVER_RANGE);
        assert_eq!(near, Some(LootKey { corpse: 1, slot: 1 }));
        assert_eq!(
            nearest_drop(&ground, Vec3::new(5.0, 0.0, 0.0), HOVER_RANGE),
            None
        );
    }
}
