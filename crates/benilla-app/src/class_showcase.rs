//! Fork-only, not 1.12.1: the ARPG class screen. In an ARPG session, Create New Character on the
//! select screen opens this screen first: the window split into five slices, left to right
//! Paladin, Mage, Druid, Warlock and Shaman, each a live stock glue scene with the server's
//! showcase character for that class standing in it, in its tier 2 set and an epic weapon
//! (`SMSG_ARPG_SHOWCASE`, cmangos `Arpg/ArpgShowcase.cpp`). Hovering a slice lights it and dims
//! the rest; clicking one opens the create screen locked to that class, every race open to it
//! (the ARPG server lets any race be any class). Back or Escape returns to select, and the
//! create screen's Back returns here.
//!
//! The stages are [`crate::portrait`]'s showcase booths ([`ShowcasePreview`]); this module asks
//! the server for the characters, hands their looks to the booths and draws the slices.

use bevy::prelude::*;
use bevy::window::PrimaryWindow;

use benilla_assets::WorldAssets;
use benilla_protocol::messages::arpg::ArpgShowcaseChar;
use benilla_protocol::{SessionEvent, SessionEventKind};

use crate::char_select::{class_name, wow_font, ClientState};
use crate::glue::art::{GlueArt, BACKDROP, GOLD};
use crate::glue::widgets::{glue_button, outlined_text, GlueBtnKind, GlueText};
use crate::glue_strings::GlueStrings;
use crate::net::{CharListMessage, CharPick, CharRequest, NetHandlerApp};
use crate::portrait::{
    PortraitImages, PortraitSource, SelectLook, ShowcasePreview, SHOWCASE, SHOWCASE_TOKENS,
};
use crate::sound::GlueSound;

/// The screen's z, as the other glue screens'.
const SCREEN_Z: i32 = 1100;
/// The facing the five characters share: the create screen's −15°.
const SHOWCASE_YAW: f32 = -15.0 * std::f32::consts::PI / 180.0;
/// A slice's tint: lit under the cursor, dimmed beside a lit one, and in between with none lit.
const LIT: Color = Color::WHITE;
const DIMMED: Color = Color::srgb(0.42, 0.42, 0.42);
const RESTING: Color = Color::srgb(0.85, 0.85, 0.85);

/// The server's showcase characters, as last answered; `None` until it answers (a stock server
/// never does, and the slices stand empty).
#[derive(Resource, Default)]
pub(crate) struct ShowcaseRoster {
    chars: Option<Vec<ArpgShowcaseChar>>,
    /// The question is out, unanswered: not asked again until a roster lands or the screen
    /// reopens.
    asked: bool,
    /// The realm `chars` came from, off the character list; another realm's list forgets them.
    realm: Option<String>,
}

/// One clickable control on the screen.
#[derive(Component, Clone, Copy, PartialEq, Eq)]
enum ShowcaseAction {
    /// A slice, by its index in [`SHOWCASE`].
    Slot(usize),
    Back,
}

/// Root of the screen, with the glue scale it was built at.
#[derive(Component)]
struct ClassShowcaseUi {
    s: f32,
}

/// The gold bar under a lit slice.
#[derive(Component)]
struct SliceLight(usize);

pub(crate) struct ClassShowcasePlugin;

impl Plugin for ClassShowcasePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ShowcaseRoster>()
            // Whatever the view, so every session event kind has its owner.
            .net_handler(SessionEventKind::ArpgShowcase, on_showcase)
            .add_systems(OnEnter(ClientState::ClassShowcase), enter_showcase)
            .add_systems(OnExit(ClientState::ClassShowcase), exit_showcase)
            // Back at select, the create screen is no longer the class screen's.
            .add_systems(OnEnter(ClientState::CharSelect), free_class_lock)
            // Back at login, the next realm may be another server's: ask it afresh.
            .add_systems(OnEnter(ClientState::Login), forget_showcase)
            .add_systems(
                Update,
                forget_other_realm
                    .after(benilla_world::schedule::WorldStage::Net)
                    .before(ask_for_showcase),
            )
            .add_systems(
                Update,
                (
                    ask_for_showcase,
                    feed_looks,
                    materialize_screen,
                    showcase_input,
                    hover_visuals,
                )
                    .chain()
                    .before(crate::glue::GlueVisuals)
                    .after(benilla_world::schedule::WorldStage::Net)
                    .run_if(in_state(ClientState::ClassShowcase)),
            );
    }
}

fn on_showcase(In(ev): In<SessionEvent>, mut roster: ResMut<ShowcaseRoster>) {
    if let SessionEvent::ArpgShowcase { chars } = ev {
        info!(
            "class screen: the server's showcase — {} character(s)",
            chars.len()
        );
        roster.chars = Some(chars);
        roster.asked = false;
    }
}

fn forget_showcase(mut roster: ResMut<ShowcaseRoster>) {
    *roster = ShowcaseRoster::default();
}

/// A character list from another realm than the showcase's came from forgets the showcase, so
/// Change Realm never shows the last realm's characters.
fn forget_other_realm(
    mut roster: ResMut<ShowcaseRoster>,
    mut lists: MessageReader<CharListMessage>,
) {
    let Some(realm) = lists
        .read()
        .last()
        .map(|l| l.realm.as_ref().map(|r| r.name.clone()))
    else {
        return;
    };
    if roster.realm != realm {
        *roster = ShowcaseRoster { realm, ..default() };
    }
}

/// A showcase character's look, as a select roster entry's.
fn look_of(c: &ArpgShowcaseChar) -> SelectLook {
    SelectLook::from(c)
}

fn enter_showcase(mut preview: ResMut<ShowcasePreview>, mut roster: ResMut<ShowcaseRoster>) {
    preview.active = true;
    preview.yaw = SHOWCASE_YAW;
    // A reopen may ask again if the last question went unanswered.
    roster.asked = false;
}

fn exit_showcase(
    mut commands: Commands,
    roots: Query<Entity, With<ClassShowcaseUi>>,
    mut preview: ResMut<ShowcasePreview>,
) {
    for e in &roots {
        commands.entity(e).despawn();
    }
    preview.active = false;
}

fn free_class_lock(mut sel: ResMut<crate::char_create::CreateSelection>) {
    sel.set_class_lock(None);
}

/// Ask the server for the characters while none have come: on entry, and again after a fresh
/// roster, since the park serves a question only once the roster it was sending is out.
fn ask_for_showcase(
    mut roster: ResMut<ShowcaseRoster>,
    pick: Res<CharPick>,
    mut lists: MessageReader<CharListMessage>,
) {
    if lists.read().count() > 0 {
        roster.asked = false;
    }
    if roster.chars.is_some() || roster.asked {
        return;
    }
    roster.asked = true;
    let _ = pick.0.send(CharRequest::Showcase);
}

/// Hand each slice its class's look whenever the server's characters change.
fn feed_looks(roster: Res<ShowcaseRoster>, mut preview: ResMut<ShowcasePreview>) {
    if !roster.is_changed() && !preview.is_added() {
        return;
    }
    let chars = roster.chars.as_deref().unwrap_or(&[]);
    let looks: [Option<SelectLook>; SHOWCASE.len()] =
        std::array::from_fn(|i| chars.iter().find(|c| c.class == SHOWCASE[i].0).map(look_of));
    if preview.looks != looks {
        preview.looks = looks;
    }
}

/// Spawn the slices once the glue art is loaded (or artless after a second), and rebuild them
/// when the glue scale changes.
#[allow(clippy::too_many_arguments)]
fn materialize_screen(
    mut commands: Commands,
    existing: Query<(Entity, &ClassShowcaseUi)>,
    assets: Res<AssetServer>,
    portraits: Res<PortraitImages>,
    mut art: ResMut<GlueArt>,
    world_assets: Option<ResMut<WorldAssets>>,
    mut images: ResMut<Assets<Image>>,
    mut add_mats: ResMut<Assets<crate::glue::add_material::AddUiMaterial>>,
    strings: Option<Res<GlueStrings>>,
    window: Query<&Window, With<PrimaryWindow>>,
) {
    if let Some(mut wa) = world_assets {
        art.ensure_loaded(&mut wa, &mut images, &mut add_mats);
    }
    let s = crate::glue::screen_scale(window.single().ok());
    if let Ok((root, ui)) = existing.single() {
        if ui.s == s {
            return;
        }
        commands.entity(root).despawn();
    }
    spawn_screen(
        &mut commands,
        &assets,
        &portraits,
        &art,
        strings.as_deref(),
        s,
    );
}

fn spawn_screen(
    commands: &mut Commands,
    assets: &AssetServer,
    portraits: &PortraitImages,
    art: &GlueArt,
    strings: Option<&GlueStrings>,
    s: f32,
) {
    let font = wow_font(assets);
    let px = |v: f32| Val::Px(v * s);
    let empty = GlueStrings::default();
    let strings = strings.unwrap_or(&empty);
    let root = commands
        .spawn((
            ClassShowcaseUi { s },
            GlobalZIndex(SCREEN_Z),
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Row,
                ..default()
            },
            BackgroundColor(BACKDROP),
        ))
        .with_children(|ui| {
            for (i, (class, _)) in SHOWCASE.iter().enumerate() {
                let mut slice = ui.spawn((
                    ShowcaseAction::Slot(i),
                    Button,
                    Node {
                        width: Val::Percent(100.0 / SHOWCASE.len() as f32),
                        height: Val::Percent(100.0),
                        ..default()
                    },
                ));
                if let Some(PortraitSource::Live(image)) = portraits.0.get(SHOWCASE_TOKENS[i]) {
                    slice.insert(ImageNode::new(image.clone()).with_color(RESTING));
                }
                slice.with_children(|c| {
                    // The class name, gold, near the slice's foot.
                    c.spawn(Node {
                        position_type: PositionType::Absolute,
                        bottom: px(110.0),
                        width: Val::Percent(100.0),
                        justify_content: JustifyContent::Center,
                        ..default()
                    })
                    .with_children(|t| {
                        outlined_text(
                            t,
                            Node::default(),
                            (),
                            (),
                            GlueText {
                                text: class_name(*class),
                                size: 26.0,
                                color: GOLD,
                                wrap: false,
                            },
                            &font,
                            s,
                        );
                    });
                    // The lit slice's gold bar along its foot.
                    c.spawn((
                        SliceLight(i),
                        Node {
                            position_type: PositionType::Absolute,
                            bottom: px(96.0),
                            left: Val::Percent(20.0),
                            width: Val::Percent(60.0),
                            height: px(3.0),
                            ..default()
                        },
                        BackgroundColor(GOLD),
                        Visibility::Hidden,
                    ));
                });
            }
        })
        .id();
    // Back, as select's: 100×35 at BOTTOMRIGHT (-30,25) of the glue canvas.
    let mut canvas = commands.spawn((crate::glue::glue_canvas(), ChildOf(root)));
    canvas.with_children(|ui| {
        ui.spawn(Node {
            position_type: PositionType::Absolute,
            right: px(30.0),
            bottom: px(25.0),
            ..default()
        })
        .with_children(|actions| {
            glue_button(
                actions,
                art,
                &font,
                ShowcaseAction::Back,
                strings.text("BACK", "Back"),
                100.0,
                35.0,
                GlueBtnKind::Small,
                s,
            );
        });
    });
}

/// A slice's click opens the create screen locked to its class; Back and Escape return to select.
fn showcase_input(
    buttons: Query<(Entity, &ShowcaseAction)>,
    clicks: Res<crate::glue::GlueClicks>,
    keys: Res<ButtonInput<KeyCode>>,
    mut sel: ResMut<crate::char_create::CreateSelection>,
    mut next: ResMut<NextState<ClientState>>,
    mut sounds: MessageWriter<GlueSound>,
) {
    for (entity, action) in &buttons {
        if !clicks.hit(entity) {
            continue;
        }
        match *action {
            ShowcaseAction::Slot(i) => {
                sounds.write(GlueSound("gsCharacterSelectionCreateNew"));
                sel.set_class_lock(Some(SHOWCASE[i].0));
                next.set(ClientState::CharCreate);
                return;
            }
            ShowcaseAction::Back => {
                sounds.write(GlueSound("gsCharacterCreationCancel"));
                next.set(ClientState::CharSelect);
                return;
            }
        }
    }
    if keys.just_pressed(KeyCode::Escape) {
        sounds.write(GlueSound("gsCharacterCreationCancel"));
        next.set(ClientState::CharSelect);
    }
}

/// Light the slice under the cursor and dim the rest; with none under it, all rest alike.
fn hover_visuals(
    mut slices: Query<(&ShowcaseAction, &Interaction, &mut ImageNode)>,
    mut lights: Query<(&SliceLight, &mut Visibility)>,
) {
    let lit = slices
        .iter()
        .find_map(|(action, interaction, _)| match (action, interaction) {
            (ShowcaseAction::Slot(i), Interaction::Hovered | Interaction::Pressed) => Some(*i),
            _ => None,
        });
    for (action, _, mut image) in &mut slices {
        let ShowcaseAction::Slot(i) = *action else {
            continue;
        };
        let want = match lit {
            Some(l) if l == i => LIT,
            Some(_) => DIMMED,
            None => RESTING,
        };
        if image.color != want {
            image.color = want;
        }
    }
    for (light, mut vis) in &mut lights {
        let want = if lit == Some(light.0) {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        };
        if *vis != want {
            *vis = want;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Each slice takes its own class's character from the server's set, whatever its order,
    /// and a class the server sent none for stands empty.
    #[test]
    fn each_slice_takes_its_own_classs_character() {
        let ch = |class: u8, race: u8| ArpgShowcaseChar {
            class,
            race,
            ..Default::default()
        };
        let mut app = App::new();
        app.init_resource::<ShowcasePreview>()
            .insert_resource(ShowcaseRoster {
                chars: Some(vec![ch(7, 6), ch(2, 1), ch(9, 2)]),
                asked: false,
                realm: None,
            })
            .add_systems(Update, feed_looks);
        app.update();
        let looks = app.world().resource::<ShowcasePreview>().looks;
        let races: Vec<Option<u8>> = looks.iter().map(|l| l.map(|l| l.race)).collect();
        // Paladin, Mage, Druid, Warlock, Shaman.
        assert_eq!(races, [Some(1), None, None, Some(2), Some(6)]);
    }

    fn list_from(realm: &str) -> CharListMessage {
        CharListMessage {
            characters: Vec::new(),
            realm: Some(benilla_protocol::RealmInfo {
                name: realm.into(),
                address: "127.0.0.1:8085".into(),
                population: 1.0,
                characters: 0,
                realm_type: 0,
                flags: 0,
                category: 0,
                id: 0,
            }),
        }
    }

    #[test]
    fn another_realms_list_forgets_the_showcase_and_the_same_realms_keeps_it() {
        let mut app = App::new();
        app.add_message::<CharListMessage>()
            .insert_resource(ShowcaseRoster {
                chars: Some(Vec::new()),
                asked: false,
                realm: Some("Alpha".into()),
            })
            .add_systems(Update, forget_other_realm);
        app.world_mut().write_message(list_from("Alpha"));
        app.update();
        assert!(app.world().resource::<ShowcaseRoster>().chars.is_some());
        app.world_mut().write_message(list_from("Beta"));
        app.update();
        let roster = app.world().resource::<ShowcaseRoster>();
        assert!(roster.chars.is_none());
        assert_eq!(roster.realm.as_deref(), Some("Beta"));
    }
}
