use bevy::prelude::*;
use bevy_immediate::{
    BevyImmediatePlugin, ImmCtx, ImmImplCap,
    attach::{BevyImmediateAttachPlugin, ImmediateAttachRoot},
    ui::{ImmCapUi, picking::clicked::ImmUiClicked, text::ImmUiText},
};

use crate::utils::my_text_style;

mod utils;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(utils::DemoUtilsPlugin)
        .add_systems(Startup, setup_camera)
        //
        //
        // Add immediate mode support with `ImmCapUi` capabilities
        .add_plugins(BevyImmediatePlugin::<ImmCapUi>::default())
        //
        // Create your Ui as a system
        .add_systems(Update, ui_as_system)
        .insert_resource(State::default())
        //
        // Attach your Ui to an entity with given marker
        .add_plugins(BevyImmediateAttachPlugin::<ImmCapUi, Tab3RootMarker>::default())
        .run();
}

fn setup_camera(mut commands: Commands) {
    commands.spawn(Camera2d);
}

#[derive(Default, Resource)]
pub struct State {
    clicked_times: usize,
    tab: Tab,
}

#[derive(Default, Debug, Hash, Clone, Copy)]
enum Tab {
    #[default]
    Tab1,
    Tab2,
    Tab3,
}

fn ui_as_system(ctx: ImmCtx<ImmCapUi>, mut state: ResMut<State>) {
    ctx.build_immediate_root("main_ui")
        .child()
        .on_spawn_insert(utils::node_full_screen_centered)
        .add(|ui| {
            ui.child()
                .on_spawn_insert(utils::container_with_background)
                .add(|ui| {
                    let mut resp = ui.child().on_spawn_insert(utils::button_bundle).add(|ui| {
                        ui.child()
                            .on_spawn_insert(utils::my_text_style)
                            .on_spawn_text("Button");
                    });

                    if resp.clicked() {
                        state.clicked_times += 1;
                    }

                    ui.child()
                        .on_spawn_insert(my_text_style)
                        .on_change_insert(state.is_changed(), || {
                            Text::new(format!("Clicked: {}", state.clicked_times))
                        });
                });

            ui.child()
                .on_spawn_insert(|| Node {
                    flex_direction: FlexDirection::Row,
                    ..utils::node_container()
                })
                .add(|ui| {
                    for tab in [Tab::Tab1, Tab::Tab2, Tab::Tab3] {
                        let mut entity =
                            ui.child().on_spawn_insert(utils::button_bundle).add(|ui| {
                                ui.child()
                                    .on_spawn_insert(utils::my_text_style)
                                    .on_spawn_text_fn(|| format!("{:?}", tab));
                            });

                        if entity.clicked() {
                            state.tab = tab;
                        }
                    }
                });

            match state.tab {
                Tab::Tab1 => {
                    ui.child_id(state.tab)
                        .on_spawn_insert(utils::container_with_background)
                        .add(|ui| {
                            ui.child()
                                .on_spawn_insert(utils::my_text_style)
                                .text("Tab1");
                        });
                }
                Tab::Tab2 => {
                    ui.child_id(state.tab)
                        .on_spawn_insert(utils::container_with_background)
                        .add(|ui| {
                            ui.child()
                                .on_spawn_insert(utils::my_text_style)
                                .text("Tab2");
                        });
                }
                Tab::Tab3 => {
                    ui.child_id(state.tab)
                        .on_spawn_insert(|| (Node::default(), Tab3RootMarker));
                }
            }
        });
}

#[derive(Component)]
#[component(storage = "SparseSet")]
struct Tab3RootMarker;

impl<Cap: ImmImplCap<ImmCapUi>> ImmediateAttachRoot<Cap> for Tab3RootMarker {
    type Params = ();

    fn execute(
        ui: &mut bevy_immediate::Imm<'_, '_, Cap>,
        params: &mut <Self::Params as bevy_ecs::system::SystemParam>::Item<'_, '_>,
    ) {
        let _ = params;

        ui.child().on_spawn_insert(utils::button_bundle).add(|ui| {
            ui.child()
                .on_spawn_insert(utils::my_text_style)
                .on_spawn_text("Tab 3");

            for i in 0..4 {
                ui.child_id(("ch", i)).text(i.to_string());
            }
        });
    }
}
