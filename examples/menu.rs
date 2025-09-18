use bevy::utils::default;
use bevy_ecs::{
    component::Component,
    resource::Resource,
    system::{ResMut, SystemParam},
};
use bevy_immediate::{
    Imm,
    attach::{BevyImmediateAttachPlugin, ImmediateAttach},
    ui::{CapUi, picking::clicked::ImmUiClicked, text::ImmUiText},
};
use bevy_ui::{FlexDirection, Node, UiRect, Val};

use crate::utils;

pub struct ExampleMenuPlugin;

impl bevy_app::Plugin for ExampleMenuPlugin {
    fn build(&self, app: &mut bevy_app::App) {
        app.insert_resource(CurrentExample::Basic);

        app.add_plugins(BevyImmediateAttachPlugin::<CapUi, MenuUiRoot>::new());
    }
}

#[derive(Component)]
pub struct MenuUiRoot;

#[derive(SystemParam)]
pub struct Params<'w> {
    current_example: ResMut<'w, CurrentExample>,
}

impl ImmediateAttach<CapUi> for MenuUiRoot {
    type Params = Params<'static>;

    fn construct(ui: &mut Imm<CapUi>, params: &mut Params) {
        ui.ch()
            .on_spawn_insert(|| Node {
                flex_direction: FlexDirection::Column,
                align_items: bevy_ui::AlignItems::Stretch,
                ..default()
            })
            .add(|ui| {
                ui.ch()
                    .on_spawn_insert(utils::title_text_style)
                    .on_spawn_text("Demo");
                ui.ch()
                    .on_spawn_insert(|| Node {
                        margin: UiRect {
                            bottom: Val::Px(10.),
                            ..default()
                        },
                        ..default()
                    })
                    .on_spawn_insert(utils::text_style)
                    .on_spawn_text("bevy_immediate");

                for (example, title) in MENU_VARIANTS {
                    let mut button = ui.ch().on_spawn_insert(utils::button_bundle).add(|ui| {
                        ui.ch()
                            .on_spawn_insert(utils::text_style)
                            .on_spawn_text(title);
                    });

                    if button.clicked() {
                        *params.current_example = example;
                    }
                }
            });
    }
}

pub const MENU_VARIANTS: [(CurrentExample, &str); 2] = [
    (CurrentExample::Basic, "Basic"),
    (CurrentExample::HelloWorld, "Hello World"),
];

#[derive(Resource, Hash, Clone, Copy)]
pub enum CurrentExample {
    Basic,
    HelloWorld,
}
