use bevy::{
    feathers::{
        self, FeathersPlugins,
        controls::{
            ButtonProps, ButtonVariant, CheckboxProps, ColorSliderProps, SliderProps,
            ToggleSwitchProps,
        },
        rounded_corners::RoundedCorners,
    },
    ui_widgets::Callback,
    utils::default,
};
use bevy_ecs::{
    component::Component,
    system::{SystemId, SystemParam},
};
use bevy_immediate::{
    Imm,
    attach::{BevyImmediateAttachPlugin, ImmediateAttach},
    ui::{
        CapsUi, activated::ImmUiActivated, clicked::ImmUiClicked,
        interaction::ImmUiInteraction as _, text::ImmUiText,
    },
};
use bevy_ui::{Checked, FlexDirection, Node};

pub struct BevyInbuiltUiExamplePlugin;

impl bevy_app::Plugin for BevyInbuiltUiExamplePlugin {
    fn build(&self, app: &mut bevy_app::App) {
        // You will need bevy feature `experimental_bevy_feathers` and/or `experimental_bevy_ui_widgets`
        // For bevy feather prestyled bevy ui components
        app.add_plugins(FeathersPlugins);

        // Initialize plugin with your root component
        app.add_plugins(BevyImmediateAttachPlugin::<CapsUi, BevyInbuiltUiExampleRoot>::new());
    }
}

#[derive(Component)]
pub struct BevyInbuiltUiExampleRoot;

#[derive(SystemParam)]
pub struct Params {}

impl ImmediateAttach<CapsUi> for BevyInbuiltUiExampleRoot {
    type Params = Params;

    fn construct(ui: &mut Imm<CapsUi>, _params: &mut Params) {
        ui.ch()
            .on_spawn_insert(|| Node {
                flex_direction: FlexDirection::Row,
                column_gap: bevy_ui::Val::Px(1.),
                ..default()
            })
            .add(|ui| {
                let variants = ["A", "B", "C", "D"];
                for (idx, title) in variants.iter().enumerate() {
                    let mut button = ui
                        .ch()
                        .on_spawn_insert(|| {
                            feathers::controls::button(
                                ButtonProps {
                                    variant: ButtonVariant::Normal,
                                    corners: if idx == 0 {
                                        RoundedCorners::Left
                                    } else if idx + 1 == variants.len() {
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
                            ui.ch().on_spawn_text(title);
                        });

                    if button.clicked() {
                        log::info!("Clicked");
                    }
                    if button.primary_clicked() {
                        log::info!("Primary clicked");
                    }
                    if button.secondary_clicked() {
                        log::info!("Secondary clicked");
                    }
                    if button.middle_clicked() {
                        log::info!("Middle clicked");
                    }
                    if button.pressed() {
                        log::info!("Pressed");
                    }
                    if button.activated() {
                        log::info!("Activated");
                    }
                }
            });
        ui.ch()
            .on_spawn_insert(|| {
                feathers::controls::checkbox(
                    CheckboxProps {
                        on_change: Callback::Ignore,
                    },
                    Checked,
                    (),
                )
            })
            .add(|ui| {
                ui.ch().on_spawn_text("Checkbox");
            });
        ui.ch().on_spawn_insert(|| {
            feathers::controls::color_slider(
                ColorSliderProps {
                    value: 0.,
                    on_change: Callback::Ignore,
                    channel: feathers::controls::ColorChannel::HslHue,
                },
                (),
            )
        });
        ui.ch()
            .on_spawn_insert(|| feathers::controls::color_swatch(()));
        ui.ch()
            .on_spawn_insert(|| feathers::controls::radio((), ()))
            .add(|ui| {
                ui.ch().on_spawn_text("Radio button");
            });
        ui.ch().on_spawn_insert(|| {
            feathers::controls::slider(
                SliderProps {
                    value: 3.,
                    min: 0.,
                    max: 10.,
                    on_change: Callback::Ignore,
                },
                (),
            )
        });
        ui.ch().on_spawn_insert(|| {
            feathers::controls::toggle_switch(
                ToggleSwitchProps {
                    on_change: Callback::Ignore,
                },
                (),
            )
        });
    }
}
