use bevy::{
    color::{Hsva, palettes::css::RED},
    feathers::{
        self, FeathersPlugins,
        controls::{
            ButtonProps, ButtonVariant, CheckboxProps, ColorSliderProps, SliderProps,
            ToggleSwitchProps,
        },
        dark_theme::create_dark_theme,
        rounded_corners::RoundedCorners,
        theme::UiTheme,
    },
    text::TextFont,
    ui_widgets::Callback,
    utils::default,
};
use bevy_ecs::{
    component::Component,
    resource::Resource,
    system::{ResMut, SystemParam},
};
use bevy_immediate::{
    Imm,
    attach::{BevyImmediateAttachPlugin, ImmediateAttach},
    ui::{
        CapsUiFeathers, activated::ImmUiActivated, checked::ImmUiChecked,
        slider_base_color::ImmUiSliderBaseColor, slider_value::ImmUiSliderValue as _,
        text::ImmUiText, ui_look::ImmUiLook,
    },
};
use bevy_ui::{FlexDirection, JustifyContent, Node};
use bevy_ui_widgets::RadioGroup;

pub struct BevyInbuiltUiExamplePlugin;

impl bevy_app::Plugin for BevyInbuiltUiExamplePlugin {
    fn build(&self, app: &mut bevy_app::App) {
        // You will need bevy feature `experimental_bevy_feathers` and/or `experimental_bevy_ui_widgets`
        // For bevy feather prestyled bevy ui components
        app.add_plugins(FeathersPlugins)
            .insert_resource(UiTheme(create_dark_theme()));

        // Initialize plugin with your root component
        app.add_plugins(BevyImmediateAttachPlugin::<
            CapsUiFeathers,
            BevyInbuiltUiExampleRoot,
        >::new());

        app.insert_resource(WidgetState::default());
    }
}

#[derive(Component)]
pub struct BevyInbuiltUiExampleRoot;

#[derive(Resource)]
struct WidgetState {
    checkboxes: Vec<bool>,
    hsva: Hsva,
}

impl Default for WidgetState {
    fn default() -> Self {
        Self {
            checkboxes: vec![true, false, true],
            hsva: RED.into(),
        }
    }
}

#[derive(SystemParam)]
pub struct Params<'w> {
    state: ResMut<'w, WidgetState>,
}

impl ImmediateAttach<CapsUiFeathers> for BevyInbuiltUiExampleRoot {
    type Params = Params<'static>;

    fn construct(ui: &mut Imm<CapsUiFeathers>, params: &mut Params) {
        ui.ch().on_spawn_insert(|| Node {
            flex_grow: 1.,
            ..default()
        });

        ui.ch()
            .on_spawn_insert(|| {
                let mut font = TextFont::default();
                font.font_size *= 1.4;
                font
            })
            .on_spawn_text("Supported");

        ui.ch()
            .on_spawn_insert(|| Node {
                flex_direction: FlexDirection::Row,
                column_gap: bevy_ui::Val::Px(4.),
                ..default()
            })
            .add(|ui| {
                let state_len = params.state.checkboxes.len();
                for (idx, state) in params.state.checkboxes.iter_mut().enumerate() {
                    let mut button = ui
                        .ch_id(idx)
                        .on_spawn_insert(|| {
                            feathers::controls::button(
                                ButtonProps {
                                    variant: ButtonVariant::Normal,
                                    corners: if idx == 0 {
                                        RoundedCorners::Left
                                    } else if idx + 1 == state_len {
                                        RoundedCorners::Right
                                    } else {
                                        RoundedCorners::None
                                    },
                                    on_click: Callback::Ignore,
                                },
                                (),
                                (),
                            )
                        })
                        .add(|ui| {
                            ui.ch().on_spawn_text_fn(|| idx.to_string());
                        });

                    if button.activated() {
                        *state = !*state;
                    }
                }
            });
        ui.ch()
            .on_spawn_insert(|| Node {
                flex_direction: FlexDirection::Row,
                justify_content: JustifyContent::SpaceBetween,
                column_gap: bevy_ui::Val::Px(10.),
                ..default()
            })
            .add(|ui| {
                for (idx, state) in params.state.checkboxes.iter_mut().enumerate() {
                    let checkbox = ui
                        .ch_id(idx)
                        .on_spawn_insert(|| {
                            feathers::controls::checkbox(
                                CheckboxProps {
                                    on_change: Callback::Ignore,
                                },
                                (),
                                (),
                            )
                        })
                        .add(|ui| {
                            ui.ch().on_spawn_text_fn(|| format!("Checkbox {idx}"));
                        });
                    checkbox.checked(state);
                }
            });

        ui.ch()
            .on_spawn_insert(|| Node {
                flex_direction: FlexDirection::Row,
                justify_content: JustifyContent::SpaceBetween,
                column_gap: bevy_ui::Val::Px(10.),
                ..default()
            })
            .add(|ui| {
                for (idx, state) in params.state.checkboxes.iter_mut().enumerate() {
                    ui.ch_id(idx)
                        .on_spawn_insert(|| {
                            feathers::controls::toggle_switch(
                                ToggleSwitchProps {
                                    on_change: Callback::Ignore,
                                },
                                (),
                            )
                        })
                        .checked(state);
                }
            });
        ui.ch()
            .on_spawn_insert(|| Node {
                flex_direction: FlexDirection::Row,
                justify_content: JustifyContent::SpaceBetween,
                column_gap: bevy_ui::Val::Px(10.),
                ..default()
            })
            .add(|ui| {
                for (idx, state) in params.state.checkboxes.iter_mut().enumerate() {
                    ui.ch_id(idx).text(format!("{idx}: {:?}", state));
                }
            });

        ui.ch().on_spawn_insert(|| Node { ..default() }).add(|ui| {
            ui.ch()
                .on_spawn_insert(|| {
                    feathers::controls::color_slider(
                        ColorSliderProps {
                            value: 0.,
                            on_change: Callback::Ignore,
                            channel: feathers::controls::ColorChannel::HslHue,
                        },
                        (),
                    )
                })
                .slider(&mut params.state.hsva.hue);
        });
        ui.ch().on_spawn_insert(|| Node { ..default() }).add(|ui| {
            ui.ch()
                .on_spawn_insert(|| {
                    feathers::controls::color_slider(
                        ColorSliderProps {
                            value: 0.,
                            on_change: Callback::Ignore,
                            channel: feathers::controls::ColorChannel::HslSaturation,
                        },
                        (),
                    )
                })
                .slider_base_color(params.state.hsva.with_saturation(1.).with_value(1.).into())
                .slider(&mut params.state.hsva.saturation);
        });
        ui.ch().on_spawn_insert(|| Node { ..default() }).add(|ui| {
            ui.ch()
                .on_spawn_insert(|| {
                    feathers::controls::color_slider(
                        ColorSliderProps {
                            value: 0.,
                            on_change: Callback::Ignore,
                            channel: feathers::controls::ColorChannel::HslLightness,
                        },
                        (),
                    )
                })
                .slider(&mut params.state.hsva.value);
        });
        ui.ch().on_spawn_insert(|| Node { ..default() }).add(|ui| {
            ui.ch()
                .on_spawn_insert(|| {
                    feathers::controls::color_slider(
                        ColorSliderProps {
                            value: 0.,
                            on_change: Callback::Ignore,
                            channel: feathers::controls::ColorChannel::Alpha,
                        },
                        (),
                    )
                })
                .slider_base_color(params.state.hsva.into())
                .slider(&mut params.state.hsva.alpha);
        });
        ui.ch().on_spawn_insert(|| Node { ..default() }).add(|ui| {
            ui.ch()
                .on_spawn_insert(|| {
                    feathers::controls::slider(
                        SliderProps {
                            value: 3.,
                            min: 0.,
                            max: 1.,
                            on_change: Callback::Ignore,
                        },
                        (),
                    )
                })
                .slider_base_color(params.state.hsva.into())
                .slider(&mut params.state.hsva.alpha);
        });

        ui.ch().on_spawn_insert(|| Node {
            flex_grow: 1.,
            ..default()
        });

        ui.ch()
            .on_spawn_insert(|| {
                let mut font = TextFont::default();
                font.font_size *= 1.4;
                font
            })
            .on_spawn_text("Does not support interactive updates (for now)");

        ui.ch().on_spawn_text("Color swatch:");

        // Background color must be updated for not for entity, but for first child element
        // And currently used assets/materials are private
        //
        // Need to udpate color swatch on the bevy side
        ui.ch()
            .on_spawn_insert(|| feathers::controls::color_swatch(()))
            .background_color(params.state.hsva.into());

        ui.ch().on_spawn_text("Radio group:");
        // Radio group currently is unusable due to
        // UI doing more than necessary.
        //
        // It updates the value of radio buttons only through callback
        ui.ch()
            .on_spawn_insert(|| {
                (
                    Node {
                        flex_direction: FlexDirection::Row,
                        justify_content: JustifyContent::SpaceBetween,
                        column_gap: bevy_ui::Val::Px(10.),
                        ..default()
                    },
                    // !!!!!! Needs to be added
                    RadioGroup {
                        on_change: Callback::Ignore,
                    },
                )
            })
            .add(|ui| {
                for (idx, state) in params.state.checkboxes.iter_mut().enumerate() {
                    ui.ch_id(idx)
                        .on_spawn_insert(|| feathers::controls::radio((), ()))
                        .add(|ui| {
                            ui.ch().on_spawn_text("Radio button");
                        })
                        .checked(state);
                }
            });

        ui.ch().on_spawn_insert(|| Node {
            flex_grow: 1.,
            ..default()
        });
    }
}
