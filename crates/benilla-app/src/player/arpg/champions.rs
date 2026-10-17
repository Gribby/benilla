//! Fork-only, not 1.12.1: the ARPG server's champions and rares (cmangos `Arpg/ArpgPacks.h`). The
//! server tells the client of each one near it once (`SMSG_ARPG_CHAMPIONS`): its tier, its own
//! name if it has one, and its affixes. The hover bar shows them for the enemy under the cursor
//! (`ArpgHud_Champion` in `arpg_hud.lua`). Each living one wears the game's own selection ring
//! under it in its tier's colour (blue for a champion, yellow for a rare, as Diablo names them),
//! turning slowly, and the aura of a 1.12 spell for each of its affixes, as that spell's aura
//! looks on whoever holds it: Fire Shield for Fire Enchanted, Frost Armor for Cold Enchanted, and
//! so on ([`AFFIX_AURAS`]). The auras are drawing only: the creature holds no such aura.

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
const RING_ALPHA: f32 = 0.9;
/// The ring's side, in yards (the selection texture's circle fills most of it), its lift off the
/// ground, and its turn, in radians a second.
const RING_SIZE: f32 = 3.0;
const RING_LIFT: f32 = 0.06;
const RING_TURN: f32 = 0.6;
/// The selection circle under a unit.
const RING_TEXTURE: &str = "mpq://textures/unitselecttexture.blp";

/// The spells whose aura each affix wears, by its name as the server sends it; the first with a
/// state kit in the client's data is the one shown.
const AFFIX_AURAS: [(&str, &[u32]); 9] = [
    ("Fire Enchanted", &[134, 2947, 2601]), // Fire Shield
    ("Cold Enchanted", &[168, 7302, 7300]), // Frost Armor, Ice Armor
    ("Stone Skin", &[20594, 8071]),         // Stoneform, Stoneskin Totem's
    ("Extra Strong", &[8599, 12880, 5229]), // Enrage
    ("Extra Fast", &[2983, 8696, 2379]),    // Sprint, Swiftness
    ("Vampiric", &[706, 687, 15473]),       // Demon Armor, Demon Skin, Shadowform
    ("Thorns", &[467, 782]),                // Thorns
    ("Teleporter", &[1463, 12042]),         // Mana Shield, Arcane Power
    ("Healer", &[139, 774]),                // Renew, Rejuvenation
];
/// The affix auras' keys among a unit's kits: past any spell id, so they never meet a real aura.
const AURA_KEY: u32 = 0xA400_0000;

/// The states' looks, for those the server keeps to itself (Chill, Freeze, Daze and stuns are real
/// auras the client draws already): by state bit, the spells whose aura each wears, the first with
/// a state kit in the client's data.
const STATE_AURAS: [(u16, &[u32]); 6] = [
    (1 << 2, &[348, 2120, 133]),   // Ignited: Immolate, Flamestrike, Fireball
    (1 << 3, &[772, 703, 1822]),   // Bleeding: Rend, Garrote, Rake
    (1 << 4, &[2818, 1978, 5570]), // Poisoned: Deadly Poison, Serpent Sting, Insect Swarm
    (1 << 5, &[589, 172, 980]),    // Withered: Shadow Word: Pain, Corruption, Curse of Agony
    (1 << 6, &[2855, 12042]),      // Overloaded: Detect Magic, Arcane Power
    (1 << 7, &[21183, 20185, 20184]), // Judged: the Judgements' debuffs
];
/// The state auras' keys, apart from the affixes'.
const STATE_KEY: u32 = 0xA500_0000;

/// Every champion the server has told of, by guid.
#[derive(Resource, Default)]
pub(crate) struct ArpgChampions {
    by_guid: HashMap<u64, ArpgChampion>,
    /// Bumped by every message.
    generation: u64,
}

/// Every nearby enemy's states the server has told of (`SMSG_ARPG_STATES`, cmangos
/// `Arpg/ArpgStates.h`): its state bits and Daze build-up, by guid. A unit whose states ended is
/// dropped.
#[derive(Resource, Default)]
pub(crate) struct ArpgEnemyStates {
    by_guid: HashMap<u64, (u16, u8)>,
}

/// Fork-only, not 1.12.1: the player's demon and horde (`SMSG_ARPG_MINIONS`, cmangos
/// `Arpg/ArpgMinions.h`), for the HUD (`ArpgHud_Minions`, pushed from `actions.rs`).
#[derive(Resource, Default)]
pub(crate) struct ArpgMinionsHud(pub(crate) benilla_protocol::messages::arpg::ArpgMinions);

/// Registered whatever the view, so every session event kind has its owner.
pub(super) fn register_net(app: &mut App) {
    app.init_resource::<ArpgChampions>()
        .init_resource::<ArpgEnemyStates>()
        .init_resource::<ArpgMinionsHud>()
        .net_handler(SessionEventKind::ArpgChampions, on_champions)
        .net_handler(SessionEventKind::ArpgStates, on_states)
        .net_handler(SessionEventKind::ArpgMinions, on_minions);
}

fn on_minions(In(ev): In<SessionEvent>, mut hud: ResMut<ArpgMinionsHud>) {
    if let SessionEvent::ArpgMinions { minions } = ev {
        hud.0 = minions;
    }
}

fn on_states(In(ev): In<SessionEvent>, mut states: ResMut<ArpgEnemyStates>) {
    if let SessionEvent::ArpgStates { states: changes } = ev {
        for unit in changes {
            if unit.mask == 0 && unit.daze == 0 {
                states.by_guid.remove(&unit.guid);
            } else {
                states.by_guid.insert(unit.guid, (unit.mask, unit.daze));
            }
        }
    }
}

/// The hover bar's states line: the states by name, and the Daze building ("Judged · Daze 60%").
fn states_text(mask: u16, daze: u8) -> String {
    use benilla_protocol::messages::arpg::ARPG_STATE_NAMES;
    let mut parts: Vec<String> = ARPG_STATE_NAMES
        .iter()
        .enumerate()
        .filter(|(i, _)| mask & (1 << i) != 0)
        .map(|(_, name)| (*name).to_string())
        .collect();
    if daze > 0 {
        parts.push(format!("Daze {daze}%"));
    }
    parts.join(" \u{b7} ")
}

/// Tell the hover bar the states of the enemy under the cursor, when they change.
fn push_hover_states(
    script: Option<NonSendMut<UiScript>>,
    hover: Option<Res<ArpgEnemyHover>>,
    states: Res<ArpgEnemyStates>,
    mut pushed: Local<Option<(u64, String)>>,
) {
    let Some(script) = script else {
        return;
    };
    let text = hover
        .and_then(|h| h.0.map(|(_, guid)| guid))
        .and_then(|guid| states.by_guid.get(&guid))
        .map_or_else(String::new, |&(mask, daze)| states_text(mask, daze));
    let mark = (script.session(), text);
    if pushed.as_ref() == Some(&mark) {
        return;
    }
    let chunk = format!(
        "if not ArpgHud_States then return false end\nArpgHud_States({})\nreturn true",
        super::tree::lua_str(&mark.1)
    );
    match script.eval::<bool>(&chunk) {
        Ok(true) => *pushed = Some(mark),
        Ok(false) => {}
        Err(e) => {
            warn!("arpg: the hover bar refused the states: {e}");
            *pushed = Some(mark);
        }
    }
}

impl ArpgChampions {
    /// The champion tier of the unit with `guid` (1 a champion, 2 a rare), if it is one.
    pub(super) fn tier_of(&self, guid: u64) -> Option<u8> {
        self.by_guid.get(&guid).map(|c| c.tier)
    }
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
    app.add_systems(
        Update,
        (
            push_hover,
            push_hover_states,
            place_rings,
            wear_affix_auras,
            wear_state_auras,
        ),
    );
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

/// The spell whose aura stands for `affix`, if the client's data has a state kit for one.
fn affix_aura(
    affix: &str,
    spells: &crate::ui_action::Spells,
    visuals: &crate::creature_anim::SpellVisuals,
) -> Option<(u32, Vec<crate::creature_anim::spell_visual::FxSlot>)> {
    let (_, candidates) = AFFIX_AURAS.iter().find(|(name, _)| *name == affix)?;
    candidates.iter().find_map(|&spell| {
        let effects = crate::creature_anim::state_kit_effects(spells, visuals, spell);
        (!effects.is_empty()).then_some((spell, effects))
    })
}

/// Dress each living champion in its affixes' auras, and take them off when it dies or goes.
#[allow(clippy::too_many_arguments)]
fn wear_affix_auras(
    champions: Res<ArpgChampions>,
    index: Res<GuidIndex>,
    units: Query<&ObjectStore>,
    spells: Option<Res<crate::ui_action::Spells>>,
    visuals: Option<Res<crate::creature_anim::SpellVisuals>>,
    mut fx: MessageWriter<crate::creature_anim::SpellKitFx>,
    mut worn: Local<HashMap<Entity, Vec<u32>>>,
    mut logged: Local<std::collections::HashSet<String>>,
) {
    use crate::creature_anim::{FxClass, FxStage, SpellKitFx};
    let (Some(spells), Some(visuals)) = (spells.as_deref(), visuals.as_deref()) else {
        return;
    };
    // Who should wear what: the living, streamed champions.
    let mut want: HashMap<Entity, &ArpgChampion> = HashMap::new();
    for (guid, champion) in &champions.by_guid {
        let Some(&entity) = index.0.get(guid) else {
            continue;
        };
        if units
            .get(entity)
            .is_ok_and(|s| s.0.unit_health().unwrap_or(1) > 0)
        {
            want.insert(entity, champion);
        }
    }
    worn.retain(|entity, keys| {
        if want.contains_key(entity) {
            return true;
        }
        for &key in keys.iter() {
            fx.write(SpellKitFx::Reap {
                entity: *entity,
                spell_id: key,
                class: FxClass::Hold,
            });
        }
        false
    });
    for (entity, champion) in want {
        if worn.contains_key(&entity) {
            continue;
        }
        let mut keys = Vec::new();
        for affix in &champion.affixes {
            match affix_aura(affix, spells, visuals) {
                Some((spell, effects)) => {
                    let key = AURA_KEY | spell;
                    if keys.contains(&key) {
                        continue;
                    }
                    if logged.insert(affix.clone()) {
                        info!("arpg: champions' {affix} wears spell {spell}'s aura");
                    }
                    fx.write(SpellKitFx::Begin {
                        entity,
                        spell_id: key,
                        persistent: true,
                        class: FxClass::Hold,
                        stage: FxStage::State,
                        effects,
                    });
                    keys.push(key);
                }
                None => {
                    if logged.insert(affix.clone()) {
                        info!("arpg: no aura in the client data for the {affix} affix");
                    }
                }
            }
        }
        worn.insert(entity, keys);
    }
}

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
    asset_server: Option<Res<AssetServer>>,
    time: Res<Time>,
    mut art: Local<Option<RingArt>>,
) {
    let turn =
        Quat::from_rotation_y(time.elapsed_secs() * RING_TURN) * Quat::from_rotation_x(-FRAC_PI_2);
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
                transform.rotation = turn;
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
                let texture = asset_server.as_ref().map(|a| a.load::<Image>(RING_TEXTURE));
                let mut material = |rgb: [f32; 3]| {
                    materials.add(StandardMaterial {
                        base_color: Color::srgba(rgb[0], rgb[1], rgb[2], RING_ALPHA),
                        base_color_texture: texture.clone(),
                        unlit: true,
                        alpha_mode: AlphaMode::Blend,
                        cull_mode: None,
                        ..default()
                    })
                };
                let champion = material(CHAMPION_RGB);
                let rare = material(RARE_RGB);
                (
                    meshes.add(Rectangle::new(RING_SIZE, RING_SIZE)),
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
            Transform::from_translation(at.translation + Vec3::Y * RING_LIFT).with_rotation(turn),
            ChampionRing(*guid),
        ));
    }
}

/// The look `state`'s bit wears, if the client's data has one of its spells' auras.
fn state_aura(
    bit: u16,
    spells: &crate::ui_action::Spells,
    visuals: &crate::creature_anim::SpellVisuals,
) -> Option<(u32, Vec<crate::creature_anim::spell_visual::FxSlot>)> {
    let (_, candidates) = STATE_AURAS.iter().find(|(b, _)| *b == bit)?;
    candidates.iter().find_map(|&spell| {
        let effects = crate::creature_anim::state_kit_effects(spells, visuals, spell);
        (!effects.is_empty()).then_some((spell, effects))
    })
}

/// Each enemy wears the looks of the states the server keeps to itself (Judged, Ignited,
/// Bleeding...), as they come and go.
#[allow(clippy::too_many_arguments)]
fn wear_state_auras(
    states: Res<ArpgEnemyStates>,
    index: Res<GuidIndex>,
    units: Query<&ObjectStore>,
    spells: Option<Res<crate::ui_action::Spells>>,
    visuals: Option<Res<crate::creature_anim::SpellVisuals>>,
    mut fx: MessageWriter<crate::creature_anim::SpellKitFx>,
    mut worn: Local<HashMap<Entity, Vec<(u16, u32)>>>,
) {
    use crate::creature_anim::{FxClass, FxStage, SpellKitFx};
    let (Some(spells), Some(visuals)) = (spells.as_deref(), visuals.as_deref()) else {
        return;
    };
    // What each living, streamed enemy should wear: its state bits that have a look.
    let mut want: HashMap<Entity, u16> = HashMap::new();
    for (guid, &(mask, _)) in &states.by_guid {
        let Some(&entity) = index.0.get(guid) else {
            continue;
        };
        let alive = units
            .get(entity)
            .is_ok_and(|s| s.0.unit_health().unwrap_or(1) > 0);
        let shown = STATE_AURAS
            .iter()
            .fold(0u16, |m, (bit, _)| m | (mask & bit));
        if alive && shown != 0 {
            want.insert(entity, shown);
        }
    }
    // Off what no longer holds.
    worn.retain(|entity, keys| {
        let keep = want.get(entity).copied().unwrap_or(0);
        keys.retain(|&(bit, key)| {
            if keep & bit != 0 {
                return true;
            }
            fx.write(SpellKitFx::Reap {
                entity: *entity,
                spell_id: key,
                class: FxClass::Hold,
            });
            false
        });
        !keys.is_empty()
    });
    // On what is new.
    for (entity, mask) in want {
        let keys = worn.entry(entity).or_default();
        for (bit, _) in STATE_AURAS {
            if mask & bit == 0 || keys.iter().any(|&(b, _)| b == bit) {
                continue;
            }
            let Some((spell, effects)) = state_aura(bit, spells, visuals) else {
                continue;
            };
            let key = STATE_KEY | spell;
            fx.write(SpellKitFx::Begin {
                entity,
                spell_id: key,
                persistent: true,
                class: FxClass::Hold,
                stage: FxStage::State,
                effects,
            });
            keys.push((bit, key));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_states_line_names_each_state_and_the_daze() {
        assert_eq!(states_text((1 << 7) | (1 << 0), 0), "Chilled \u{b7} Judged");
        assert_eq!(states_text(0, 45), "Daze 45%");
        assert_eq!(states_text(1 << 8, 0), "Dazed");
    }

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
