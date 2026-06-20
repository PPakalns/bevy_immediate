use bevy::app::PluginGroup;
use bevy::color::palettes;
use bevy::color::{Alpha, Color, Srgba};
use bevy::ecs::{
    component::Component,
    hierarchy::Children,
    resource::Resource,
    system::{ResMut, SystemParam},
};
use bevy::feathers::{
    constants::fonts,
    containers::{group, group_body, group_header, pane, pane_body, pane_header},
    controls::{
        ColorChannel, FeathersButton, FeathersCheckbox, FeathersColorPlane, FeathersColorSlider,
        FeathersColorSwatch, FeathersNumberInput, FeathersRadio, FeathersTextInput,
        FeathersTextInputContainer, FeathersToggleSwitch,
    },
    display::{label, label_small},
    font_styles::InheritableFont,
    tokens,
};
use bevy::math::Vec3;
use bevy::scene::bsn;
use bevy::ui::widget::Text;
use bevy::ui::{
    AlignItems, Display, FlexDirection, GridPlacement, JustifyContent, Node, RepeatedGridTrack,
    Val, px,
};
use bevy::ui_widgets::RadioGroup;
use bevy::{
    feathers::{
        FeathersPlugins, controls::ButtonVariant, dark_theme::create_dark_theme,
        rounded_corners::RoundedCorners, theme::UiTheme,
    },
    utils::default,
};
use bevy_immediate::{
    Imm, ImmEntity,
    attach::{BevyImmediateAttachPlugin, ImmediateAttach},
    ui::{
        CapsUiFeathers, activated::ImmUiActivated, button_variant::ImmUiFeathersButtonVariant,
        checked::ImmUiChecked, color_plane::ImmUiColorPlane, color_swatch::ImmUiColorSwatch,
        disabled::ImmUiInteractionsDisabled, look::ImmUiLook, number_input::ImmUiNumberInput,
        slider_base_color::ImmUiSliderBaseColor, slider_value::ImmUiSliderValue as _,
        text::ImmUiText, text_input::ImmUiTextInput,
    },
};
use std::ops::DerefMut;

use crate::styles::{self, title_text_style};

pub struct BevyWidgetExamplePlugin;

impl bevy::app::Plugin for BevyWidgetExamplePlugin {
    fn build(&self, app: &mut bevy::app::App) {
        // You will need bevy features `experimental_bevy_feathers` and/or `experimental_bevy_ui_widgets`
        // For bevy feather prestyled bevy UI components

        // You will need bevy_immediate features `bevy_feathers` and/or `bevy_ui_widgets`

        app.add_plugins(FeathersPlugins.build())
            .insert_resource(UiTheme(create_dark_theme()));

        // Initialize plugin with your root component
        app.add_plugins(BevyImmediateAttachPlugin::<
            CapsUiFeathers,
            BevyWidgetExampleRoot,
        >::new());

        app.insert_resource(WidgetState::default());
        app.insert_resource(RadioState::default());
    }
}

// See bevy example `bevy/examples/ui/feathers.rs`
// https://github.com/bevyengine/bevy/tree/main/examples
// for more information

#[derive(Component)]
pub struct BevyWidgetExampleRoot;

#[derive(Resource)]
struct WidgetState {
    values: Vec<Checkbox>,
    rgb_color: Srgba,
    vec3_prop: Vec3,
}

struct Checkbox {
    value: bool,
    disabled: bool,
}

#[derive(Default, Resource, Debug, Hash, PartialEq, Clone, Copy)]
pub enum RadioState {
    Dog,
    #[default]
    Cat,
    Horse,
}

impl Default for WidgetState {
    fn default() -> Self {
        Self {
            values: [false, true]
                .into_iter()
                .flat_map(|disabled| {
                    [false, true]
                        .into_iter()
                        .map(move |value| Checkbox { value, disabled })
                })
                .collect(),
            rgb_color: palettes::tailwind::EMERALD_800.with_alpha(0.7),
            vec3_prop: Vec3::new(10.1, 7.124, 100.0),
        }
    }
}

#[derive(SystemParam)]
pub struct Params<'w> {
    state: ResMut<'w, WidgetState>,
    radio_state: ResMut<'w, RadioState>,
}

impl ImmediateAttach<CapsUiFeathers> for BevyWidgetExampleRoot {
    type Params = Params<'static>;

    fn construct(ui: &mut Imm<CapsUiFeathers>, params: &mut Params) {
        let WidgetState {
            values,
            rgb_color,
            vec3_prop,
        } = params.state.deref_mut();

        fn button_rounded_corners_row(idx: usize, count: usize) -> RoundedCorners {
            if idx == 0 {
                RoundedCorners::Left
            } else if idx + 1 == count {
                RoundedCorners::Right
            } else {
                RoundedCorners::None
            }
        }

        ui.ch()
            .on_spawn_insert(title_text_style)
            .on_spawn_text("Bevy widgets");
        ui.ch()
            .on_spawn_text("Using bevy_feathers styling, based on bevy_ui_widgets.");

        ui.ch()
            .on_spawn_insert(|| Node {
                flex_direction: bevy::ui::FlexDirection::Row,
                column_gap: Val::Px(4.),
                justify_content: bevy::ui::JustifyContent::FlexStart,
                align_self: bevy::ui::AlignSelf::FlexStart,
                ..default()
            })
            .add(|ui| {
                ui.ch().on_spawn_text("Values:");

                let mut button = ui.ch().on_spawn_apply_scene(|| {
                    bsn! {
                        @FeathersButton {
                            @variant: ButtonVariant::Normal,
                            @corners: RoundedCorners::Left,
                        } Children [
                            (Text("+"))
                        ]
                    }
                });

                if button.activated() {
                    values.push(Checkbox {
                        value: false,
                        disabled: false,
                    });
                }

                let mut button = ui
                    .ch()
                    .on_spawn_apply_scene(|| {
                        bsn! {
                            @FeathersButton {
                                @variant: ButtonVariant::Normal,
                                @corners: RoundedCorners::Left,
                            } Children [
                                (Text("-"))
                            ]
                        }
                    })
                    .interactions_disabled(values.len() == 1);
                if button.activated() {
                    values.pop();
                }
            });

        let column_count = values.len().max(1);
        ui.ch()
            .on_spawn_insert(|| Node {
                display: Display::Grid,
                grid_template_columns: RepeatedGridTrack::auto(column_count as u16),
                row_gap: Val::Px(4.),
                column_gap: Val::Px(4.),
                ..default()
            })
            .node_mut(|node| {
                // Check if column count needs change
                let value = RepeatedGridTrack::auto(column_count as u16);
                if node.grid_template_columns != value {
                    node.grid_template_columns = value;
                }
            })
            .add(|ui| {
                for (variant_idx, variant) in [ButtonVariant::Primary, ButtonVariant::Normal]
                    .into_iter()
                    .enumerate()
                {
                    for (idx, state) in values.iter_mut().enumerate() {
                        let mut button = ui
                            .ch_id(("variant", variant_idx, idx))
                            .on_spawn_apply_scene(|| {
                                bsn! {
                                    @FeathersButton {
                                        @variant: {variant.clone()},
                                        @corners: button_rounded_corners_row(idx, column_count),
                                    }
                                }
                            })
                            .add(|ui| {
                                ui.ch().on_spawn_text_fn(|| idx.to_string());
                            })
                            .interactions_disabled(state.disabled);

                        if button.activated() {
                            state.value = !state.value;
                        }
                    }
                }

                for (idx, state) in values.iter_mut().enumerate() {
                    let mut button = ui
                        .ch_id(("primary_change", idx))
                        .on_spawn_apply_scene(|| {
                            bsn! {
                                @FeathersButton {
                                    @corners: button_rounded_corners_row(idx, column_count),
                                }
                            }
                        })
                        .add(|ui| {
                            ui.ch().on_spawn_text_fn(|| idx.to_string());
                        })
                        .primary_button(state.value)
                        .interactions_disabled(state.disabled);

                    if button.activated() {
                        state.value = !state.value;
                    }
                }

                for (idx, state) in values.iter_mut().enumerate() {
                    let checkbox = ui
                        .ch_id(("checkbox", idx))
                        .on_spawn_apply_scene(|| {
                            bsn! { @FeathersCheckbox { @caption: {bsn!{ Text({format!("Checkbox {idx}")})} }}}
                        })
                        .interactions_disabled(state.disabled);
                    checkbox.checked(&mut state.value);
                }

                for (idx, state) in values.iter_mut().enumerate() {
                    ui.ch_id(("toggle", idx))
                        .on_spawn_apply_scene(|| bsn! {@FeathersToggleSwitch})
                        .interactions_disabled(state.disabled)
                        .checked(&mut state.value);
                }

                for (idx, state) in values.iter_mut().enumerate() {
                    ui.ch_id(("radio", idx))
                        .on_spawn_apply_scene(|| bsn! {@FeathersRadio})
                        .add(|ui| {
                            ui.ch().on_spawn_text("Radio button");
                        })
                        .interactions_disabled(state.disabled)
                        .checked_if_eq(true, &mut state.value);
                }

                for (idx, state) in values.iter_mut().enumerate() {
                    ui.ch_id(("value", idx))
                        .text(format!("{idx}: {:6?}", state.value));
                }

                for (idx, state) in values.iter_mut().enumerate() {
                    let text = if state.disabled {
                        "disabled"
                    } else {
                        "enabled"
                    };
                    ui.ch_id(("disabled_text", idx)).text(format!("{:8}", text));
                }

                ui.ch()
                    .on_spawn_insert(|| Node {
                        // GridPlacement::start_end(1, -1) doesn't work correctly
                        grid_column: GridPlacement::span(column_count as u16),
                        ..default()
                    })
                    .node_mut(|node| {
                        let span = GridPlacement::span(column_count as u16);
                        if node.grid_column != span {
                            node.grid_column = span;
                        }
                    })
                    .on_spawn_text("Interactions disabled:");

                for (idx, state) in values.iter_mut().enumerate() {
                    ui.ch_id(("disabled", idx))
                        .on_spawn_apply_scene(|| bsn! {@FeathersToggleSwitch})
                        .checked(&mut state.disabled);
                }
            });

        ui.ch().on_spawn_apply_scene(pane).add(|ui| {
            ui.ch().on_spawn_apply_scene(pane_header).add(|ui| {
                ui.ch().on_spawn_apply_scene(|| label("Pane"));
            });

            ui.ch().on_spawn_apply_scene(pane_body).add(|ui| {
                ui.ch().on_spawn_apply_scene(group).add(|ui| {
                    ui.ch().on_spawn_apply_scene(group_header).add(|ui| {
                        ui.ch().on_spawn_apply_scene(|| label("Group1"));
                    });

                    ui.ch().on_spawn_apply_scene(group_body).add(|ui| {
                        ui.ch().on_spawn_insert(row_between_node).add(|ui| {
                            ui.ch().on_spawn_text("Srgba");

                            ui.ch().on_spawn_insert(|| Node {
                                flex_grow: 1.0,
                                ..default()
                            });

                            ui.ch()
                                .on_spawn_apply_scene(|| {
                                    bsn! {
                                        @FeathersTextInputContainer
                                        Node {
                                            flex_grow: 0.,
                                            padding: { px(4).left() },
                                        }
                                    }
                                })
                                .add(|ui| {
                                    ui.ch()
                                        .on_spawn_apply_scene(|| {
                                            bsn! {
                                                @FeathersTextInput {
                                                    @visible_width: 10f32,
                                                    @max_characters: 9usize,
                                                }
                                                InheritableFont {
                                                    font: fonts::MONO,
                                                }
                                            }
                                        })
                                        .input_get_set(|value| match value {
                                            None => rgb_color.to_hex(),
                                            Some(hex) => {
                                                if let Ok(color) = Srgba::hex(hex) {
                                                    *rgb_color = color;
                                                }
                                                rgb_color.to_hex()
                                            }
                                        });
                                });

                            ui.ch()
                                .on_spawn_apply_scene(|| bsn! { @FeathersColorSwatch })
                                .color_swatch((*rgb_color).into());
                        });

                        ui.ch()
                            .on_spawn_apply_scene(|| bsn! { @FeathersColorPlane::RedBlue })
                            .color_plane_get_set(|value| match value {
                                None => Vec3::new(rgb_color.red, rgb_color.blue, rgb_color.green),
                                Some(xy) => {
                                    rgb_color.red = xy.x;
                                    rgb_color.blue = xy.y;
                                    Vec3::new(rgb_color.red, rgb_color.blue, rgb_color.green)
                                }
                            });

                        rgba_color_slider(
                            ui.ch(),
                            ColorChannel::Red,
                            Color::from(*rgb_color),
                            |value| match value {
                                None => rgb_color.red,
                                Some(red) => {
                                    rgb_color.red = red;
                                    red
                                }
                            },
                        );
                        rgba_color_slider(
                            ui.ch(),
                            ColorChannel::Green,
                            Color::from(*rgb_color),
                            |value| match value {
                                None => rgb_color.green,
                                Some(green) => {
                                    rgb_color.green = green;
                                    green
                                }
                            },
                        );
                        rgba_color_slider(
                            ui.ch(),
                            ColorChannel::Blue,
                            Color::from(*rgb_color),
                            |value| match value {
                                None => rgb_color.blue,
                                Some(blue) => {
                                    rgb_color.blue = blue;
                                    blue
                                }
                            },
                        );
                        rgba_color_slider(
                            ui.ch(),
                            ColorChannel::Alpha,
                            Color::from(*rgb_color),
                            |value| match value {
                                None => rgb_color.alpha,
                                Some(alpha) => {
                                    rgb_color.alpha = alpha;
                                    alpha
                                }
                            },
                        );
                    });
                });

                ui.ch().on_spawn_apply_scene(group).add(|ui| {
                    ui.ch().on_spawn_apply_scene(group_header).add(|ui| {
                        ui.ch().on_spawn_apply_scene(|| label("Group2"));
                    });

                    ui.ch().on_spawn_apply_scene(group_body).add(|ui| {
                        ui.ch()
                            .on_spawn_apply_scene(|| label_small("Vec3 property"));
                        ui.ch().on_spawn_insert(vec3_row_node).add(|ui| {
                            vec3_number_input(
                                ui.ch(),
                                &mut vec3_prop.x,
                                tokens::TEXT_INPUT_X_AXIS,
                                "X",
                            );
                            vec3_number_input(
                                ui.ch(),
                                &mut vec3_prop.y,
                                tokens::TEXT_INPUT_Y_AXIS,
                                "Y",
                            );
                            vec3_number_input(
                                ui.ch(),
                                &mut vec3_prop.z,
                                tokens::TEXT_INPUT_Z_AXIS,
                                "Z",
                            );
                        });
                    });
                });

                ui.ch().on_spawn_apply_scene(group).add(|ui| {
                    ui.ch().on_spawn_apply_scene(group_header).add(|ui| {
                        ui.ch()
                            .text(format!("Radio group: {:?}", *params.radio_state));
                    });

                    ui.ch().on_spawn_apply_scene(group_body).add(|ui| {
                        ui.ch()
                            .on_spawn_insert(styles::row_node_container)
                            .on_spawn_insert(|| RadioGroup)
                            .add(|ui| {
                                for (name, state) in [
                                    ("Dog", RadioState::Dog),
                                    ("Cat", RadioState::Cat),
                                    ("Horse", RadioState::Horse),
                                ] {
                                    ui.ch_id(("pane_radio", state))
                                        .on_spawn_apply_scene(|| bsn! {@FeathersRadio})
                                        .add(|ui| {
                                            ui.ch().on_spawn_text(name);
                                        })
                                        .checked_if_eq(state, &mut params.radio_state);
                                }
                            });
                    });
                });
            });
        });
    }
}

fn vec3_number_input(
    ui: ImmEntity<CapsUiFeathers>,
    value: &mut f32,
    sigil_color: bevy::feathers::theme::ThemeToken,
    label_text: &'static str,
) {
    ui.on_spawn_apply_scene(move || {
        bsn! {
            @FeathersNumberInput {
                @sigil_color: sigil_color,
                @label_text: label_text,
            }
            Node {
                flex_grow: 1.0,
            }
        }
    })
    .number_input(value);
}

fn rgba_color_slider(
    ui: ImmEntity<CapsUiFeathers>,
    channel: ColorChannel,
    base_color: Color,
    value_get_set: impl FnMut(Option<f32>) -> f32,
) {
    ui.on_spawn_apply_scene(move || {
        bsn! {
            @FeathersColorSlider {
                @channel: channel,
            }
        }
    })
    .slider_base_color(base_color)
    .slider_get_set(value_get_set);
}

fn vec3_row_node() -> Node {
    Node {
        display: Display::Flex,
        flex_direction: FlexDirection::Row,
        column_gap: px(6),
        align_items: AlignItems::Center,
        justify_content: JustifyContent::SpaceBetween,
        ..default()
    }
}

fn row_between_node() -> Node {
    Node {
        display: Display::Flex,
        flex_direction: FlexDirection::Row,
        align_items: AlignItems::Center,
        justify_content: JustifyContent::SpaceBetween,
        column_gap: px(4),
        ..default()
    }
}
