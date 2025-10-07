use bevy::utils::default;
use bevy_ecs::component::Component;
use bevy_immediate::{
    Imm,
    attach::{BevyImmediateAttachPlugin, ImmediateAttach},
    ui::{
        CapsUi,
        activated::ImmUiActivated,
        anchored::ImmUiAnchored,
        anchored_ui_plugin::{Anchor, AnchorOption, Direction},
        floating_ui_focus_plugin::FocusCloseCurrentTree,
        interaction::ImmUiInteraction,
        selected::ImmUiSelectable,
        text::ImmUiText,
    },
    utils::ImmLocalHashMemoryHelper,
};
use bevy_ui::{
    AlignItems, AlignSelf, BackgroundColor, BorderColor, FlexDirection, JustifyContent,
    JustifySelf, Node, px,
};

use crate::styles::{compact_button_bundle, compact_node_container, row_node_container};

pub struct AnchoredUiExamplePlugin;

impl bevy_app::Plugin for AnchoredUiExamplePlugin {
    fn build(&self, app: &mut bevy_app::App) {
        app.add_plugins(BevyImmediateAttachPlugin::<CapsUi, AnchoredUiExampleRoot>::new());
    }
}

#[derive(Component)]
pub struct AnchoredUiExampleRoot;

impl ImmediateAttach<CapsUi> for AnchoredUiExampleRoot {
    type Params = ();

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

                        // Helper utility to store simple state that can be checked
                        let mut local_state =
                            ImmLocalHashMemoryHelper::new(&mut button, "is_activated", &false);

                        if button.activated() {
                            local_state.store(&true);
                        }

                        button = button.selected(local_state.is_stored(&true));

                        if local_state.is_stored(&true) {
                            button = button.add_dropdown_container(
                                || {
                                    local_state.store(&false);
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

                        local_state.finalize(&mut button);
                    }
                });
            });
    }
}

fn dropdown_content(ui: &mut Imm<'_, '_, CapsUi>) {
    let mut menu_container = ui.ch().on_spawn_insert(|| {
        (
            Node {
                border: px(1.).into(),
                ..compact_node_container()
            },
            BackgroundColor(bevy_color::palettes::css::BLACK.into()),
            BorderColor::all(bevy_color::palettes::css::WHITE),
        )
    });

    // On ImmEntity for the lifetime of entity custom hashed values can be stored
    //
    // We will use locally stored hash value to store current menu choice
    let mut local_state = ImmLocalHashMemoryHelper::new(&mut menu_container, "opened_menu", &None);

    menu_container = menu_container.add(|ui| {
        for _ in 0..10 {
            let mut button = ui.ch().on_spawn_insert(compact_button_bundle).add(|ui| {
                ui.ch().on_spawn_text("Example");
            });

            if button.hovered() {
                local_state.store(&Some(button.imm_id()));
            }

            let is_open = local_state.is_stored(&Some(button.imm_id()));

            button = button.selected(is_open);

            if is_open {
                button.add_dropdown_container(
                    || {
                        local_state.store(&None);
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
                                            Node {
                                                border: px(1.).into(),
                                                ..compact_node_container()
                                            },
                                            BackgroundColor(
                                                bevy_color::palettes::css::BLACK.into(),
                                            ),
                                            BorderColor::all(bevy_color::palettes::css::DARK_GRAY),
                                        )
                                    })
                                    .add(|ui| {
                                        for _ in 0..3 {
                                            let mut button = ui
                                                .ch()
                                                .on_spawn_insert(compact_button_bundle)
                                                .add(|ui| {
                                                    ui.ch().on_spawn_text("Final button");
                                                });
                                            if button.activated() {
                                                // Do something

                                                // Mark menu hierarchy to be closed
                                                let event = FocusCloseCurrentTree {
                                                    entity: button.entity(),
                                                };
                                                button.commands().trigger(event);
                                            }
                                        }
                                    });
                            });
                    },
                );
            }
        }
    });

    local_state.finalize(&mut menu_container);
}
