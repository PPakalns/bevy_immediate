use std::ops::DerefMut;

use bevy::{
    color::Hsva,
    feathers::{
        self, FeathersPlugins,
        controls::{ButtonProps, ButtonVariant, ColorSliderProps, SliderProps},
        dark_theme::create_dark_theme,
        rounded_corners::RoundedCorners,
        theme::UiTheme,
    },
    text::TextFont,
    utils::default,
};
use bevy_color::palettes::css::DARK_BLUE;
use bevy_ecs::{
    component::Component,
    hierarchy::Children,
    query::Without,
    resource::Resource,
    system::{Query, ResMut, SystemParam},
};
use bevy_immediate::{
    CapSet, Imm, ImmMarker,
    attach::{BevyImmediateAttachPlugin, ImmediateAttach},
    ui::{
        CapsUiFeathers, activated::ImmUiActivated, button_variant::ImmUiFeathersButtonVariant,
        checked::ImmUiChecked, disabled::ImmUiInteractionsDisabled, look::ImmUiLook,
        slider_base_color::ImmUiSliderBaseColor, slider_value::ImmUiSliderValue as _,
        text::ImmUiText,
    },
};
use bevy_ui::{BackgroundColor, Display, GridPlacement, Node, RepeatedGridTrack, Val};
use bevy_ui_widgets::SliderStep;

use crate::styles::title_text_style;

pub struct BevyWidgetExamplePlugin;

impl bevy_app::Plugin for BevyWidgetExamplePlugin {
    fn build(&self, app: &mut bevy_app::App) {
        // You will need bevy feature `experimental_bevy_feathers` and/or `experimental_bevy_ui_widgets`
        // For bevy feather prestyled bevy ui components
        app.add_plugins(FeathersPlugins)
            .insert_resource(UiTheme(create_dark_theme()));

        // Initialize plugin with your root component
        app.add_plugins(BevyImmediateAttachPlugin::<
            CapsUiFeathers,
            BevyWidgetExampleRoot,
        >::new());

        app.insert_resource(WidgetState::default());
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
    hsva: Hsva,
}

struct Checkbox {
    value: bool,
    disabled: bool,
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
            hsva: DARK_BLUE.into(),
        }
    }
}

#[derive(SystemParam)]
pub struct Params<'w, 's, Caps: CapSet> {
    state: ResMut<'w, WidgetState>,

    // Needed for color swatch
    children: Query<'w, 's, &'static Children>,
    background: Query<'w, 's, &'static mut BackgroundColor, Without<ImmMarker<Caps>>>,
}

impl ImmediateAttach<CapsUiFeathers> for BevyWidgetExampleRoot {
    type Params = Params<'static, 'static, CapsUiFeathers>;

    fn construct(ui: &mut Imm<CapsUiFeathers>, params: &mut Params<CapsUiFeathers>) {
        let Params {
            state,
            children,
            background,
        } = params;

        let WidgetState { values, hsva } = state.deref_mut();

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
                flex_direction: bevy_ui::FlexDirection::Row,
                column_gap: Val::Px(4.),
                justify_content: bevy_ui::JustifyContent::FlexStart,
                align_self: bevy_ui::AlignSelf::FlexStart,
                ..default()
            })
            .add(|ui| {
                ui.ch().on_spawn_text("Values:");

                let mut button = ui
                    .ch()
                    .on_spawn_insert(|| {
                        feathers::controls::button(
                            ButtonProps {
                                variant: ButtonVariant::Normal,
                                corners: RoundedCorners::Left,
                            },
                            (),
                            (),
                        )
                    })
                    .add(|ui| {
                        ui.ch().on_spawn_text("+");
                    });

                if button.activated() {
                    values.push(Checkbox {
                        value: false,
                        disabled: false,
                    });
                }

                let mut button = ui
                    .ch()
                    .on_spawn_insert(|| {
                        feathers::controls::button(
                            ButtonProps {
                                variant: ButtonVariant::Normal,
                                corners: RoundedCorners::Right,
                            },
                            (),
                            (),
                        )
                    })
                    .add(|ui| {
                        ui.ch().on_spawn_text("-");
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
                            .on_spawn_insert(|| {
                                feathers::controls::button(
                                    ButtonProps {
                                        variant: variant.clone(),
                                        corners: button_rounded_corners_row(idx, column_count),
                                    },
                                    (),
                                    (),
                                )
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
                        .on_spawn_insert(|| {
                            feathers::controls::button(
                                ButtonProps {
                                    variant: Default::default(),
                                    corners: button_rounded_corners_row(idx, column_count),
                                },
                                (),
                                (),
                            )
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
                        .on_spawn_insert(|| feathers::controls::checkbox((), ()))
                        .add(|ui| {
                            ui.ch().on_spawn_text_fn(|| format!("Checkbox {idx}"));
                        })
                        .interactions_disabled(state.disabled);
                    checkbox.checked(&mut state.value);
                }

                for (idx, state) in values.iter_mut().enumerate() {
                    ui.ch_id(("toggle", idx))
                        .on_spawn_insert(|| feathers::controls::toggle_switch(()))
                        .interactions_disabled(state.disabled)
                        .checked(&mut state.value);
                }

                for (idx, state) in values.iter_mut().enumerate() {
                    let mut radio_button = ui
                        .ch_id(("radio", idx))
                        .on_spawn_insert(|| feathers::controls::radio((), ()))
                        .add(|ui| {
                            ui.ch().on_spawn_text("Radio button");
                        })
                        .interactions_disabled(state.disabled)
                        .checked(&mut state.value);

                    // WARN: Widget doesn't update it's checked state. Need to check if widget was activated
                    // Will be fixed in future bevy release where `.checked` will work independently.
                    if radio_button.activated() {
                        state.value = true;
                    }
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
                        .on_spawn_insert(|| feathers::controls::toggle_switch(()))
                        .checked(&mut state.disabled);
                }
            });

        ui.ch().on_spawn_insert(|| Node {
            height: Val::Px(50.),
            ..default()
        });

        ui.ch().on_spawn_text("Color sliders");

        ui.ch().on_spawn_insert(Node::default).add(|ui| {
            ui.ch()
                .on_spawn_insert(|| {
                    feathers::controls::color_slider(
                        ColorSliderProps {
                            value: 0.,
                            channel: feathers::controls::ColorChannel::HslHue,
                        },
                        SliderStep(5.),
                    )
                })
                .slider(&mut hsva.hue);
        });
        ui.ch().on_spawn_insert(Node::default).add(|ui| {
            ui.ch()
                .on_spawn_insert(|| {
                    feathers::controls::color_slider(
                        ColorSliderProps {
                            value: 0.,
                            channel: feathers::controls::ColorChannel::HslSaturation,
                        },
                        SliderStep(0.05),
                    )
                })
                .slider_base_color(hsva.with_saturation(1.).with_value(1.).into())
                .slider(&mut hsva.saturation);
        });
        ui.ch().on_spawn_insert(Node::default).add(|ui| {
            ui.ch()
                .on_spawn_insert(|| {
                    feathers::controls::color_slider(
                        ColorSliderProps {
                            value: 0.,
                            channel: feathers::controls::ColorChannel::HslLightness,
                        },
                        SliderStep(0.05),
                    )
                })
                .slider(&mut hsva.value);
        });
        ui.ch().on_spawn_insert(Node::default).add(|ui| {
            ui.ch()
                .on_spawn_insert(|| {
                    feathers::controls::color_slider(
                        ColorSliderProps {
                            value: 0.,
                            channel: feathers::controls::ColorChannel::Alpha,
                        },
                        SliderStep(0.05),
                    )
                })
                .slider_base_color((*hsva).into())
                .slider(&mut hsva.alpha);
        });
        ui.ch().on_spawn_insert(Node::default).add(|ui| {
            ui.ch()
                .on_spawn_insert(|| {
                    feathers::controls::slider(
                        SliderProps {
                            value: 0.,
                            min: 0.,
                            max: 1.,
                        },
                        SliderStep(0.05),
                    )
                })
                .slider_base_color((*hsva).into())
                .slider(&mut hsva.alpha);
        });

        // WARN: Requires manual support
        // Background color must be updated not for entity, but for the first child
        let entity = ui
            .ch()
            .on_spawn_insert(|| feathers::controls::color_swatch(()))
            .background_color((*hsva).into());

        if let Some(mut background) = children
            .get(entity.entity())
            .ok()
            .and_then(|children| children.first())
            .and_then(|child| background.get_mut(*child).ok())
        {
            // Additional check to avoid unnecessary updates
            if *background != (*hsva).into() {
                *background = (*hsva).into();
            }
        }
    }
}
