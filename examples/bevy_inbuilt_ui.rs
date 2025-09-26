use bevy::{
    feathers::{
        self, FeathersPlugins,
        controls::{ButtonProps, ButtonVariant},
        rounded_corners::RoundedCorners,
    },
    ui_widgets::Callback,
    utils::default,
};
use bevy_ecs::{component::Component, system::SystemParam};
use bevy_immediate::{
    Imm,
    attach::{BevyImmediateAttachPlugin, ImmediateAttach},
    ui::{
        CapsUi, interaction::ImmUiInteraction as _, picking::clicked::ImmUiClicked, text::ImmUiText,
    },
};
use bevy_ui::{FlexDirection, Node};

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
                ..default()
            })
            .add(|ui| {
                let mut button = ui
                    .ch()
                    .on_spawn_insert(|| {
                        feathers::controls::button(
                            ButtonProps {
                                variant: ButtonVariant::Normal,
                                corners: RoundedCorners::Left,
                                on_click: Callback::Ignore,
                            },
                            (),
                            (),
                        )
                    })
                    .add(|ui| {
                        ui.ch().on_spawn_text("Test");
                    });

                if button.clicked() {
                    log::info!("Clicked");
                }
                if button.primary_clicked() {
                    log::info!("Clicked");
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
            });
    }
}
