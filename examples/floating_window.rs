use bevy::prelude::SpawnRelated;
use bevy::utils::default;
use bevy_color::{
    Srgba,
    palettes::css::{BLACK, LIGHT_GRAY},
};
use bevy_ecs::{
    bundle::Bundle,
    children,
    component::Component,
    resource::Resource,
    system::{ResMut, SystemParam},
};
use bevy_immediate::{
    Imm, ImmCtx, ImmEntity,
    attach::{BevyImmediateAttachPlugin, ImmediateAttach},
    ui::{
        CapsUi,
        activated::ImmUiActivated,
        floating_window_plugin::{
            self, FloatingWindow, FloatingWindowPlugin, FloatingWindowStoreLocationId,
        },
        selected::ImmUiSelectable,
        text::ImmUiText,
    },
    utils::ImmLocalHashMemoryHelper,
};
use bevy_picking::hover::Hovered;
use bevy_platform::collections::{HashMap, HashSet, hash_set::Entry};
use bevy_ui::{
    AlignItems, BackgroundColor, BorderColor, FlexDirection, JustifyContent, Node, Overflow,
    percent, px, vh, vw,
    widget::{Button, Text},
};
use strum::IntoEnumIterator;

use crate::{
    anchored::AnchoredUiExampleRoot,
    bevy_scrollarea::BevyScrollareaExampleRoot,
    bevy_widgets::BevyWidgetExampleRoot,
    extension_use::ExtensionUseExampleRoot,
    hello_world::HelloWorldRoot,
    hot_patching::HotPatchingRoot,
    main_menu::CurrentExample,
    power_user::PowerUserExampleRoot,
    styles::{self, MyStyle, button_bundle, node_container},
    tooltip::TooltipExampleRoot,
    widget_use::WidgetUseExampleRoot,
};

pub struct FloatingWindowExamplePlugin;

impl bevy_app::Plugin for FloatingWindowExamplePlugin {
    fn build(&self, app: &mut bevy_app::App) {
        app.add_plugins(FloatingWindowPlugin);

        app.add_plugins(BevyImmediateAttachPlugin::<CapsUi, FloatingWindowRoot>::new());

        app.add_systems(bevy_app::Update, spawn_windows_from_system);

        app.insert_resource(FloatingWindowState::default());
    }
}

#[derive(Resource, Default)]
pub struct FloatingWindowState {
    /// These windows will be unique, newly spawned
    windows: HashMap<CurrentExample, Vec<i32>>,
    unique_window_id_counter: i32, // Counter to generate unique ids

    /// These windows will remember they location
    windows_with_memory: HashSet<CurrentExample>,
}

#[derive(Component)]
pub struct FloatingWindowRoot;

#[derive(SystemParam)]
pub struct Params<'w> {
    state: ResMut<'w, FloatingWindowState>,
}

impl ImmediateAttach<CapsUi> for FloatingWindowRoot {
    type Params = Params<'static>; // Access data from World using SystemParam

    fn construct(ui: &mut Imm<CapsUi>, params: &mut Params) {
        ui.ch()
            .on_spawn_text("Toggle floating windows for examples (will remember their location)");

        ui.ch()
            .on_spawn_insert(|| Node {
                flex_wrap: bevy_ui::FlexWrap::Wrap,
                ..styles::row_node_container()
            })
            .add(|ui| {
                for example in CurrentExample::iter() {
                    let mut button = ui
                        .ch()
                        .on_spawn_insert(styles::button_bundle)
                        .add(|ui| {
                            ui.ch().on_spawn_text_fn(|| example.title().to_string());
                        })
                        .selected(params.state.windows_with_memory.contains(&example));

                    if button.activated() {
                        match params.state.windows_with_memory.entry(example) {
                            Entry::Occupied(entry) => {
                                entry.remove();
                            }
                            Entry::Vacant(entry) => {
                                entry.insert();
                            }
                        }
                    }
                }
            });

        ui.ch().on_spawn_text("Add floating windows for examples");

        ui.ch()
            .on_spawn_insert(|| Node {
                flex_wrap: bevy_ui::FlexWrap::Wrap,
                ..styles::row_node_container()
            })
            .add(|ui| {
                for example in CurrentExample::iter() {
                    let mut button = ui.ch().on_spawn_insert(styles::button_bundle).add(|ui| {
                        ui.ch().on_spawn_text_fn(|| example.title().to_string());
                    });
                    if button.activated() {
                        let unique_id = params.state.unique_window_id_counter;
                        params.state.unique_window_id_counter += 1;
                        params
                            .state
                            .windows
                            .entry(example)
                            .or_default()
                            .push(unique_id);
                    }
                }
            });

        // Showcases how floating windows can be spawned directly from UI
        ui.ch()
            .on_spawn_text("Spawn floating windows as \"popup\" directly:");

        ui.ch()
            .on_spawn_insert(styles::row_node_container)
            .add(|ui| {
                let mut button = ui.ch().on_spawn_insert(button_bundle).add(|ui| {
                    ui.ch().on_spawn_text("Popup");
                });

                let mut is_popup_open_local =
                    ImmLocalHashMemoryHelper::new(&mut button, "is_popup_open", &false);

                if button.activated() {
                    is_popup_open_local.store(&true);
                }

                let is_open = is_popup_open_local.is_stored(&true);

                button = button.selected(is_open);

                if is_open {
                    // Manage new entity tree
                    button = button.unrooted("my_ui", |ui| {
                        my_window(
                            ui.ch(),
                            true,
                            |fw| fw,
                            || "Popup".into(),
                            || is_popup_open_local.store(&false),
                            |ui| {
                                ui.ch().on_spawn_insert(node_container).add(|ui| {
                                    ui.ch().on_spawn_text("Popup text");
                                });
                            },
                        );
                    });
                }

                is_popup_open_local.finalize(&mut button);
            });
    }
}

fn spawn_windows_from_system(ctx: ImmCtx<CapsUi>, mut state: ResMut<FloatingWindowState>) {
    // P.S. It is possible to spawn floating windows directly
    // See how `.unrooted()` is used previously in this file.

    let mut ui_root = ctx.build_immediate_root("example_windows");

    for (example, ids) in state.windows.iter_mut() {
        ids.retain(|id| {
            let mut open = true;

            show_example_window(
                ui_root.ch_id(("non_unique", example, id)),
                false,
                example,
                &mut open,
            );

            open
        });
    }

    state.windows_with_memory.retain(|example| {
        let mut open = true;

        show_example_window(ui_root.ch_id(("memory", example)), true, example, &mut open);

        open
    });
}

fn show_example_window(
    imm_entity: ImmEntity<CapsUi>,
    memorize_location: bool,
    example: &CurrentExample,
    open: &mut bool,
) {
    my_window(
        imm_entity,
        memorize_location,
        |fw| fw,
        || example.title().into(),
        || *open = false,
        |ui| {
            // Content
            let content = ui.ch().on_spawn_insert(|| {
                (Node {
                    flex_direction: FlexDirection::Column,
                    ..node_container()
                },)
            });
            match *example {
                CurrentExample::WidgetUse => {
                    content.on_spawn_insert(|| WidgetUseExampleRoot);
                }
                CurrentExample::HelloWorld => {
                    content.on_spawn_insert(|| HelloWorldRoot);
                }
                CurrentExample::ExtensionUse => {
                    content.on_spawn_insert(|| ExtensionUseExampleRoot);
                }
                CurrentExample::PowerUser => {
                    content.on_spawn_insert(|| PowerUserExampleRoot);
                }
                CurrentExample::BevyWidgets => {
                    content.on_spawn_insert(|| BevyWidgetExampleRoot);
                }
                CurrentExample::BevyScrollbar => {
                    content.on_spawn_insert(|| BevyScrollareaExampleRoot);
                }
                CurrentExample::HotPatching => {
                    content.on_spawn_insert(|| HotPatchingRoot);
                }
                CurrentExample::Tooltip => {
                    content.on_spawn_insert(|| TooltipExampleRoot);
                }
                CurrentExample::FloatingWindows => {
                    content.on_spawn_insert(|| FloatingWindowRoot);
                }
                CurrentExample::Anchored => {
                    content.on_spawn_insert(|| AnchoredUiExampleRoot);
                }
            }
        },
    );
}

fn my_window(
    mut ui_root: ImmEntity<CapsUi>,
    memorize_location: bool,
    window_params: impl FnOnce(FloatingWindow) -> FloatingWindow,
    title: impl FnOnce() -> String,
    on_close: impl FnOnce(),
    content: impl FnOnce(&mut Imm<CapsUi>),
) {
    ui_root = ui_root.on_spawn_insert(|| {
        (
            Node {
                border: px(2.).into(),
                ..default()
            },
            window_params(FloatingWindow {
                max_width: vw(95.),
                max_height: vh(95.),
                min_width: px(100.),
                min_height: px(100.),
                ..Default::default()
            }),
            BackgroundColor(BLACK.into()),
            BorderColor::all(LIGHT_GRAY),
            // Add borders for window resizing
            floating_window_plugin::resizable_borders(8., ()),
        )
    });

    if memorize_location {
        // Add component which will store window location
        // When window is reopened, it should open at the same location.
        let id = ui_root.imm_id();
        ui_root = ui_root.on_spawn_insert(|| FloatingWindowStoreLocationId(id));
    }

    ui_root.add(|ui| {
        // Inner node required for overflow clipping
        ui.ch()
            .on_spawn_insert(|| Node {
                flex_direction: FlexDirection::Column,
                width: percent(100.),
                height: percent(100.),
                max_width: percent(100.),
                max_height: percent(100.),
                overflow: Overflow::clip(),
                ..default()
            })
            .add(|ui| {
                ui.ch()
                    .on_spawn_insert(|| {
                        (
                            Node {
                                flex_direction: FlexDirection::RowReverse,
                                align_items: AlignItems::Stretch,
                                ..default()
                            },
                            BackgroundColor(Srgba::new(0.363, 0.363, 0.363, 1.0).into()),
                        )
                    })
                    .add(|ui| {
                        let mut close = ui.ch().on_spawn_insert(close_button_bundle);
                        if close.activated() {
                            on_close();
                        }

                        ui.ch()
                            .on_spawn_insert(|| Node {
                                flex_grow: 1.,
                                justify_content: JustifyContent::Center,
                                align_items: AlignItems::Center,
                                ..default()
                            })
                            .add(|ui| {
                                ui.ch().on_spawn_text_fn(title);
                            });
                    });

                content(ui);
            });
    });
}

fn close_button_bundle() -> impl Bundle {
    (
        Node {
            flex_grow: 0.,
            aspect_ratio: Some(1.),
            padding: px(4.).into(),
            ..default()
        },
        Button,
        Hovered::default(),
        MyStyle,
        children![Text("X".into())],
    )
}
