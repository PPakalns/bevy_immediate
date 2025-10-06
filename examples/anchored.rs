use bevy::utils::default;
use bevy_ecs::component::Component;
use bevy_immediate::{
    Imm,
    attach::{BevyImmediateAttachPlugin, ImmediateAttach},
    imm_id,
    ui::{
        CapsUi,
        activated::ImmUiActivated,
        anchored::ImmUiAnchored,
        anchored_entity_plugin::{Anchor, AnchorOption, Direction},
        interaction::ImmUiInteraction,
        selected::ImmUiSelectable,
        text::ImmUiText,
    },
};
use bevy_ui::{
    AlignItems, AlignSelf, BackgroundColor, BorderColor, FlexDirection, JustifyContent,
    JustifyItems, JustifySelf, Node,
};

use crate::styles::{compact_button_bundle, compact_node_container, row_node_container};

pub struct AnchoredUiExamplePlugin;

impl bevy_app::Plugin for AnchoredUiExamplePlugin {
    fn build(&self, app: &mut bevy_app::App) {
        // Add bevy immediate plugin with UI support which will construct UI
        // rooted at entity with `HelloWorldRoot` component
        app.add_plugins(BevyImmediateAttachPlugin::<CapsUi, AnchoredUiExampleRoot>::new());
    }
}

#[derive(Component)]
pub struct AnchoredUiExampleRoot;

impl ImmediateAttach<CapsUi> for AnchoredUiExampleRoot {
    type Params = (); // Access data from World using SystemParam

    fn construct(ui: &mut Imm<CapsUi>, _: &mut ()) {
        ui.ch()
            .on_spawn_insert(|| Node {
                flex_direction: FlexDirection::Column,
                flex_grow: 1.,
                align_self: AlignSelf::Center,
                justify_self: JustifySelf::Center,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            })
            .add(|ui| {
                ui.ch().on_spawn_text("Dropdown");

                ui.ch().on_spawn_insert(row_node_container).add(|ui| {
                    for (x, y, tx, ty, dir_text) in [
                        (
                            Anchor::Start,
                            Anchor::End,
                            Anchor::Start,
                            Anchor::Start,
                            "Up",
                        ),
                        (
                            Anchor::Start,
                            Anchor::Start,
                            Anchor::Start,
                            Anchor::End,
                            "Down",
                        ),
                    ] {
                        let mut button = ui
                            .ch_id((x, y, tx, ty))
                            .on_spawn_insert(compact_button_bundle)
                            .add(|ui| {
                                ui.ch().on_spawn_text_fn(|| format!("Menu {}", dir_text));
                            });

                        // Dropdown state management using on entity stored value
                        let menu_open_id = imm_id("activated");
                        if button.activated() {
                            button.hash_set(menu_open_id, imm_id(()));
                        }

                        let mut is_open_value = button.hash_get(menu_open_id);

                        button = button.selected(is_open_value.is_some());

                        if is_open_value.is_some() {
                            button = button.with_dropdown_container(
                                || {
                                    is_open_value = None;
                                },
                                |container| {
                                    container
                                        .on_spawn_insert(|| AnchorOption {
                                            anchor: Direction { x, y },
                                            target_anchor: Direction { x: tx, y: ty },
                                            ..default()
                                        })
                                        .add(|ui| {
                                            dropdown_content(ui);
                                        });
                                },
                            );
                        }

                        button.hash_update(menu_open_id, is_open_value);
                    }
                });
            });
    }
}

fn dropdown_content(ui: &mut Imm<'_, '_, CapsUi>) {
    let mut menu_container = ui.ch().on_spawn_insert(|| {
        (
            compact_node_container(),
            BackgroundColor(bevy_color::palettes::css::BLACK.into()),
            BorderColor::all(bevy_color::palettes::css::DARK_GRAY),
        )
    });

    // On ImmEntity for the lifetime of entity custom hashed values can be stored
    //
    // We will use locally stored hash value to store current menu choice
    let menu_hash_id = imm_id("menu_value");
    let mut stored_hash = menu_container.hash_get(menu_hash_id);

    menu_container = menu_container.add(|ui| {
        for _ in 0..10 {
            let mut button = ui.ch().on_spawn_insert(compact_button_bundle).add(|ui| {
                ui.ch().on_spawn_text("Example");
            });

            let button_id = button.imm_id();
            if button.hovered() {
                stored_hash = Some(button_id);
            }

            button = button.selected(stored_hash == Some(button_id));

            if stored_hash == Some(button_id) {
                button.with_dropdown_container(
                    || {
                        stored_hash = None;
                    },
                    |container| {
                        container
                            .on_spawn_insert(|| AnchorOption {
                                anchor: Direction {
                                    x: Anchor::Start,
                                    y: Anchor::Start,
                                },
                                target_anchor: Direction {
                                    x: Anchor::End,
                                    y: Anchor::Start,
                                },
                                ..default()
                            })
                            .add(|ui| {
                                ui.ch()
                                    .on_spawn_insert(|| {
                                        (
                                            compact_node_container(),
                                            BackgroundColor(
                                                bevy_color::palettes::css::BLACK.into(),
                                            ),
                                            BorderColor::all(bevy_color::palettes::css::DARK_GRAY),
                                        )
                                    })
                                    .add(|ui| {
                                        for _ in 0..3 {
                                            ui.ch().on_spawn_insert(compact_button_bundle).add(
                                                |ui| {
                                                    ui.ch().on_spawn_text("Final button");
                                                },
                                            );
                                        }
                                    });
                            });
                    },
                );
            }
        }
    });

    menu_container.hash_update(menu_hash_id, stored_hash);
}
