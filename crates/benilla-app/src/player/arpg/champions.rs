//! Fork-only, not 1.12.1: the ARPG server's champions and rares (cmangos `Arpg/ArpgPacks.h`). The
//! server tells the client of each one near it once (`SMSG_ARPG_CHAMPIONS`): its tier, its own
//! name if it has one, and its affixes. The hover bar shows them for the enemy under the cursor
//! (`ArpgHud_Champion` in `arpg_hud.lua`), and a ring in the tier's colour lies under each living
//! one: blue for a champion, yellow for a rare.

use std::collections::HashMap;
use std::f32::consts::FRAC_PI_2;

use benilla_protocol::messages::ArpgChampion;
use benilla_protocol::{SessionEvent, SessionEventKind};
use benilla_ui::script::UiScript;

use crate::net::{GuidIndex, NetHandlerApp, ObjectStore};
use crate::target::arpg_soft::ArpgEnemyHover;

use super::*;

/// A champion's tier, as the wire numbers it.
const TIER_RARE: u8 = 2;

/// The rings' colours: a champion's blue, a rare's yellow, as Diablo names them.
const CHAMPION_RGB: [f32; 3] = [0.30, 0.55, 1.0];
const RARE_RGB: [f32; 3] = [1.0, 0.85, 0.25];
const RING_ALPHA: f32 = 0.55;
/// The ring's radii, in yards, and its lift off the ground.
const RING_INNER: f32 = 1.05;
const RING_OUTER: f32 = 1.35;
const RING_LIFT: f32 = 0.06;

/// Every champion the server has told of, by guid.
#[derive(Resource, Default)]
pub(crate) struct ArpgChampions {
    by_guid: HashMap<u64, ArpgChampion>,
    /// Bumped by every message.
    generation: u64,
}

/// Registered whatever the view, so every session event kind has its owner.
pub(super) fn register_net(app: &mut App) {
    app.init_resource::<ArpgChampions>()
        .net_handler(SessionEventKind::ArpgChampions, on_champions);
}

fn on_champions(In(ev): In<SessionEvent>, mut state: ResMut<ArpgChampions>) {
    if let SessionEvent::ArpgChampions { champions } = ev {
        for champion in champions {
            state.by_guid.insert(champion.guid, champion);
        }
        state.generation += 1;
    }
}

/// The view's half: the hover bar's champion line and the rings.
pub(super) fn plugin(app: &mut App) {
    app.add_systems(Update, (push_hover, place_rings));
}

/// The Lua chunk that tells the hover bar about `champion` (`None`: an ordinary enemy, or none).
fn hover_chunk(champion: Option<&ArpgChampion>) -> String {
    let call = match champion {
        None => "ArpgHud_Champion(0)".to_string(),
        Some(c) => format!(
            "ArpgHud_Champion({}, {}, {})",
            c.tier,
            super::tree::lua_str(&c.name),
            super::tree::lua_str(&c.affixes.join(", "))
        ),
    };
    format!("if not ArpgHud_Champion then return false end\n{call}\nreturn true")
}

/// Tell the hover bar whether the enemy under the cursor is a champion, when that changes.
fn push_hover(
    script: Option<NonSendMut<UiScript>>,
    hover: Option<Res<ArpgEnemyHover>>,
    champions: Res<ArpgChampions>,
    mut pushed: Local<Option<(u64, Option<u64>, u64)>>,
) {
    let Some(script) = script else {
        return;
    };
    let champion = hover
        .and_then(|h| h.0.map(|(_, guid)| guid))
        .and_then(|guid| champions.by_guid.get(&guid));
    let mark = (
        script.session(),
        champion.map(|c| c.guid),
        champions.generation,
    );
    if *pushed == Some(mark) {
        return;
    }
    match script.eval::<bool>(&hover_chunk(champion)) {
        Ok(true) => *pushed = Some(mark),
        Ok(false) => {}
        Err(e) => {
            warn!("arpg: the hover bar refused a champion: {e}");
            *pushed = Some(mark);
        }
    }
}

/// A ring under a champion, by its guid.
#[derive(Component)]
struct ChampionRing(u64);

/// The ring mesh and its two materials, made on first use.
type RingArt = (
    Handle<Mesh>,
    Handle<StandardMaterial>,
    Handle<StandardMaterial>,
);

/// Keep a ring under each living champion that is streamed, and none under any other.
#[allow(clippy::too_many_arguments)]
fn place_rings(
    mut commands: Commands,
    champions: Res<ArpgChampions>,
    index: Res<GuidIndex>,
    units: Query<(&Transform, &ObjectStore), Without<ChampionRing>>,
    mut rings: Query<(Entity, &ChampionRing, &mut Transform)>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut art: Local<Option<RingArt>>,
) {
    let mut placed: HashMap<u64, Entity> = HashMap::new();
    for (entity, ring, mut transform) in &mut rings {
        let unit = champions
            .by_guid
            .get(&ring.0)
            .and_then(|_| index.0.get(&ring.0))
            .and_then(|e| units.get(*e).ok())
            .filter(|(_, store)| store.0.unit_health().unwrap_or(1) > 0);
        match unit {
            Some((at, _)) => {
                transform.translation = at.translation + Vec3::Y * RING_LIFT;
                placed.insert(ring.0, entity);
            }
            None => commands.entity(entity).despawn(),
        }
    }
    for (guid, champion) in &champions.by_guid {
        if placed.contains_key(guid) {
            continue;
        }
        let Some((at, store)) = index.0.get(guid).and_then(|e| units.get(*e).ok()) else {
            continue;
        };
        if store.0.unit_health().unwrap_or(1) == 0 {
            continue;
        }
        let (mesh, champion_mat, rare_mat) = art
            .get_or_insert_with(|| {
                let mut material = |rgb: [f32; 3]| {
                    materials.add(StandardMaterial {
                        base_color: Color::srgba(rgb[0], rgb[1], rgb[2], RING_ALPHA),
                        unlit: true,
                        alpha_mode: AlphaMode::Blend,
                        cull_mode: None,
                        ..default()
                    })
                };
                let champion = material(CHAMPION_RGB);
                let rare = material(RARE_RGB);
                (
                    meshes.add(Annulus::new(RING_INNER, RING_OUTER)),
                    champion,
                    rare,
                )
            })
            .clone();
        let material = if champion.tier == TIER_RARE {
            rare_mat
        } else {
            champion_mat
        };
        commands.spawn((
            Mesh3d(mesh),
            MeshMaterial3d(material),
            Transform::from_translation(at.translation + Vec3::Y * RING_LIFT)
                .with_rotation(Quat::from_rotation_x(-FRAC_PI_2)),
            ChampionRing(*guid),
        ));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_hover_chunk_names_the_champion_and_its_affixes() {
        let rare = ArpgChampion {
            guid: 7,
            tier: 2,
            name: "Gorefang the Swift".into(),
            affixes: vec!["Extra Fast".into(), "Vampiric".into()],
        };
        let script = UiScript::new().unwrap();
        script
            .run("function ArpgHud_Champion(t, n, a) GOT = { t = t, n = n, a = a } end")
            .unwrap();
        assert!(script.eval::<bool>(&hover_chunk(Some(&rare))).unwrap());
        assert_eq!(script.eval::<u32>("return GOT.t").unwrap(), 2);
        assert_eq!(
            script.eval::<String>("return GOT.n").unwrap(),
            "Gorefang the Swift"
        );
        assert_eq!(
            script.eval::<String>("return GOT.a").unwrap(),
            "Extra Fast, Vampiric"
        );
        assert!(script.eval::<bool>(&hover_chunk(None)).unwrap());
        assert_eq!(script.eval::<u32>("return GOT.t").unwrap(), 0);
        assert!(!UiScript::new()
            .unwrap()
            .eval::<bool>(&hover_chunk(None))
            .unwrap());
    }
}
