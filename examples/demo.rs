use bevy::{color::palettes::basic::*, prelude::*, winit::WinitSettings};
use bevy_immediate::{
    BevyImmediatePlugin, ImmCtx, ImmImplCap,
    attach::{BevyImmediateAttachPlugin, ImmediateAttachRoot},
    ui::{ImmCapUi, picking::clicked::ImmUiClicked},
};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        //
        // If using immediate plugin with ui capabilities and "picking" feature
        // Make sure that picking plugin is enabled.
        // .add_plugins(
        //     bevy_picking:DefaultPickingPlugins
        // )
        //
        //
        // Add immediate mode with ui extensions for ergonomic API
        .add_plugins(BevyImmediatePlugin::<ImmCapUi>::default())
        .add_plugins(BevyImmediateAttachPlugin::<ImmCapUi, Tab3RootMarker>::default())
        //
        // Only run the app when there is user input. This will significantly reduce CPU/GPU use.
        .insert_resource(WinitSettings::desktop_app())
        //
        .insert_resource(State::default())
        .add_systems(Startup, setup)
        .add_systems(Update, button_system)
        .add_systems(Update, immediate_ui_demo)
        .run();
}

fn setup(mut commands: Commands) {
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

fn immediate_ui_demo(ctx: ImmCtx<ImmCapUi>, mut state: ResMut<State>) {
    ctx.build_immediate_root("main_ui")
        .child()
        .on_spawn_insert(|| Node {
            flex_direction: FlexDirection::Column,
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            row_gap: Val::Px(10.),
            ..default()
        })
        .add(|ui| {
            ui.child()
                .on_spawn_insert(|| Node {
                    flex_direction: FlexDirection::Row,
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    column_gap: Val::Px(10.),
                    ..default()
                })
                .add(|ui| {
                    let mut resp = ui
                        .child()
                        .on_spawn_insert(|| {
                            (
                                Button,
                                Node {
                                    width: Val::Px(150.0),
                                    height: Val::Px(65.0),
                                    border: UiRect::all(Val::Px(5.0)),
                                    // horizontally center child text
                                    justify_content: JustifyContent::Center,
                                    // vertically center child text
                                    align_items: AlignItems::Center,
                                    ..default()
                                },
                                BorderColor(Color::BLACK),
                                BorderRadius::MAX,
                                BackgroundColor(NORMAL_BUTTON),
                            )
                        })
                        .add(|ui| {
                            ui.child().on_spawn_insert(|| {
                                (
                                    Text::new("Button"),
                                    TextColor(Color::srgb(0.9, 0.9, 0.9)),
                                    TextShadow::default(),
                                )
                            });
                        });

                    if resp.clicked() {
                        state.clicked_times += 1;
                    }

                    ui.child()
                        .on_spawn_insert(|| {
                            (TextColor(Color::srgb(0.9, 0.9, 0.9)), TextShadow::default())
                        })
                        .on_change_insert(state.is_changed(), || {
                            Text::new(format!("Clicked: {}", state.clicked_times))
                        });
                });

            ui.child()
                .on_spawn_insert(|| Node {
                    flex_direction: FlexDirection::Row,
                    column_gap: Val::Px(10.),
                    ..default()
                })
                .add(|ui| {
                    for tab in [Tab::Tab1, Tab::Tab2, Tab::Tab3] {
                        let mut resp = ui
                            .child()
                            .on_spawn_insert(|| {
                                (
                                    Button,
                                    Node {
                                        width: Val::Px(65.0),
                                        height: Val::Px(65.0),
                                        border: UiRect::all(Val::Px(5.0)),
                                        // horizontally center child text
                                        justify_content: JustifyContent::Center,
                                        // vertically center child text
                                        align_items: AlignItems::Center,
                                        ..default()
                                    },
                                    BorderColor(Color::BLACK),
                                    BorderRadius::MAX,
                                    BackgroundColor(NORMAL_BUTTON),
                                )
                            })
                            .add(|ui| {
                                ui.child().on_spawn_insert(|| {
                                    (
                                        Text::new(format!("{:?}", tab)),
                                        TextColor(Color::srgb(0.9, 0.9, 0.9)),
                                        TextShadow::default(),
                                    )
                                });
                            });

                        if resp.clicked() {
                            state.tab = tab;
                        }
                    }
                });

            match state.tab {
                Tab::Tab1 => {
                    ui.child_with_id(state.tab)
                        .on_spawn_insert(|| {
                            (
                                Node {
                                    width: Val::Px(300.0),
                                    height: Val::Px(65.0),
                                    ..default()
                                },
                                BorderColor(Color::srgb(1., 0., 0.)),
                                BorderRadius::MAX,
                                BackgroundColor(NORMAL_BUTTON),
                            )
                        })
                        .add(|ui| {
                            ui.child().on_spawn_insert(|| {
                                (
                                    Text::new("Tab 1"),
                                    TextColor(Color::srgb(0.9, 0.9, 0.9)),
                                    TextShadow::default(),
                                )
                            });
                        });
                }
                Tab::Tab2 => {
                    ui.child_with_id(state.tab)
                        .on_spawn_insert(|| {
                            (
                                Node {
                                    width: Val::Px(150.0),
                                    height: Val::Px(65.0),
                                    ..default()
                                },
                                BorderColor(Color::srgb(0., 1., 0.)),
                                BackgroundColor(NORMAL_BUTTON),
                            )
                        })
                        .add(|ui| {
                            ui.child().on_spawn_insert(|| {
                                (
                                    Text::new("Tab 2"),
                                    TextColor(Color::srgb(0.9, 0.9, 0.9)),
                                    TextShadow::default(),
                                )
                            });
                        });
                }
                Tab::Tab3 => {
                    ui.child_with_id(state.tab)
                        .on_spawn_insert(|| (Node::default(), Tab3RootMarker));
                }
            }
        });
}

const NORMAL_BUTTON: Color = Color::srgb(0.15, 0.15, 0.15);
const HOVERED_BUTTON: Color = Color::srgb(0.25, 0.25, 0.25);
const PRESSED_BUTTON: Color = Color::srgb(0.35, 0.75, 0.35);

#[allow(clippy::type_complexity)]
fn button_system(
    mut interaction_query: Query<
        (&Interaction, &mut BackgroundColor, &mut BorderColor),
        (Changed<Interaction>, With<Button>),
    >,
) {
    for (interaction, mut color, mut border_color) in &mut interaction_query {
        match *interaction {
            Interaction::Pressed => {
                *color = PRESSED_BUTTON.into();
                border_color.0 = RED.into();
            }
            Interaction::Hovered => {
                *color = HOVERED_BUTTON.into();
                border_color.0 = Color::WHITE;
            }
            Interaction::None => {
                *color = NORMAL_BUTTON.into();
                border_color.0 = Color::BLACK;
            }
        }
    }
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

        ui.child()
            .on_spawn_insert(|| {
                (
                    Node {
                        width: Val::Px(75.0),
                        height: Val::Px(65.0),
                        ..default()
                    },
                    BorderColor(Color::srgb(0., 0., 1.)),
                    BorderRadius::MAX,
                    BackgroundColor(NORMAL_BUTTON),
                )
            })
            .add(|ui| {
                ui.child().on_spawn_insert(|| {
                    (
                        Text::new("Tab 3"),
                        TextColor(Color::srgb(0.9, 0.9, 0.9)),
                        TextShadow::default(),
                    )
                });
            });
    }
}
