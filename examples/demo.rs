use bevy::{color::palettes::basic::*, prelude::*, winit::WinitSettings};
use bevy_immediate::{BevyImmediatePlugin, BevyImmediateUiExtensionPlugin, SimpleUiCtx, sid};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        //
        // Add immediate mode plugin and ui extensions for ergonomic API
        .add_plugins(BevyImmediatePlugin)
        .add_plugins(BevyImmediateUiExtensionPlugin)
        //
        // Only run the app when there is user input. This will significantly reduce CPU/GPU use.
        .insert_resource(WinitSettings::desktop_app())
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

fn immediate_ui_demo(ctx: SimpleUiCtx, mut state: ResMut<State>) {
    ctx.init_ui("main_ui")
        .build()
        .on_insert_add_bundle(|| Node {
            flex_direction: FlexDirection::Column,
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            row_gap: Val::Px(10.),
            ..default()
        })
        .add(|ui| {
            ui.build()
                .on_insert_add_bundle(|| Node {
                    flex_direction: FlexDirection::Row,
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    column_gap: Val::Px(10.),
                    ..default()
                })
                .add(|ui| {
                    let mut resp = ui
                        .build()
                        .on_insert_add_bundle(|| {
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
                            ui.build()
                                .on_insert_add_bundle(|| {
                                    (
                                        Text::new("Button"),
                                        TextColor(Color::srgb(0.9, 0.9, 0.9)),
                                        TextShadow::default(),
                                    )
                                })
                                .add_empty();
                        });

                    if resp.clicked() {
                        state.clicked_times += 1;
                    }

                    ui.build()
                        .on_insert_add_bundle(|| {
                            (TextColor(Color::srgb(0.9, 0.9, 0.9)), TextShadow::default())
                        })
                        .on_change_add_bundle(state.is_changed(), || {
                            Text::new(format!("Clicked: {}", state.clicked_times))
                        })
                        .add_empty();
                });

            ui.build()
                .on_insert_add_bundle(|| Node {
                    flex_direction: FlexDirection::Row,
                    column_gap: Val::Px(10.),
                    ..default()
                })
                .add(|ui| {
                    for tab in [Tab::Tab1, Tab::Tab2, Tab::Tab3] {
                        let mut resp = ui
                            .build()
                            .on_insert_add_bundle(|| {
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
                                ui.build()
                                    .on_insert_add_bundle(|| {
                                        (
                                            Text::new(format!("{:?}", tab)),
                                            TextColor(Color::srgb(0.9, 0.9, 0.9)),
                                            TextShadow::default(),
                                        )
                                    })
                                    .add_empty();
                            });

                        if resp.clicked() {
                            state.tab = tab;
                        }
                    }
                });

            match state.tab {
                Tab::Tab1 => {
                    ui.build_id(sid(state.tab))
                        .on_insert_add_bundle(|| {
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
                            ui.build()
                                .on_insert_add_bundle(|| {
                                    (
                                        Text::new("Tab 1"),
                                        TextColor(Color::srgb(0.9, 0.9, 0.9)),
                                        TextShadow::default(),
                                    )
                                })
                                .add_empty();
                        });
                }
                Tab::Tab2 => {
                    ui.build_id(sid(state.tab))
                        .on_insert_add_bundle(|| {
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
                            ui.build()
                                .on_insert_add_bundle(|| {
                                    (
                                        Text::new("Tab 2"),
                                        TextColor(Color::srgb(0.9, 0.9, 0.9)),
                                        TextShadow::default(),
                                    )
                                })
                                .add_empty();
                        });
                }
                Tab::Tab3 => {
                    ui.build_id(sid(state.tab))
                        .on_insert_add_bundle(|| {
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
                            ui.build()
                                .on_insert_add_bundle(|| {
                                    (
                                        Text::new("Tab 3"),
                                        TextColor(Color::srgb(0.9, 0.9, 0.9)),
                                        TextShadow::default(),
                                    )
                                })
                                .add_empty();
                        });
                }
            }
        });
}

const NORMAL_BUTTON: Color = Color::srgb(0.15, 0.15, 0.15);
const HOVERED_BUTTON: Color = Color::srgb(0.25, 0.25, 0.25);
const PRESSED_BUTTON: Color = Color::srgb(0.35, 0.75, 0.35);

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
