use bevy::utils::default;
use bevy_ecs::{
    component::Component,
    resource::Resource,
    system::{ResMut, SystemParam},
};
use bevy_immediate::{
    Imm,
    attach::{BevyImmediateAttachPlugin, ImmediateAttach},
    ui::{CapsUi, picking::clicked::ImmUiClicked, selected::ImmUiSelectable, text::ImmUiText},
};
use bevy_ui::{FlexDirection, Node, UiDebugOptions, Val};

use crate::styles::{self, button_bundle, fill_parent_node, text_style};

pub struct MenuExamplePlugin;

impl bevy_app::Plugin for MenuExamplePlugin {
    fn build(&self, app: &mut bevy_app::App) {
        app.insert_resource(CurrentExample::WidgetUse);

        app.add_plugins(BevyImmediateAttachPlugin::<CapsUi, MenuUiRoot>::new());
    }
}

#[derive(Component)]
pub struct MenuUiRoot;

#[derive(SystemParam)]
pub struct Params<'w> {
    current_example: ResMut<'w, CurrentExample>,
    debug_options: ResMut<'w, UiDebugOptions>,
}

impl ImmediateAttach<CapsUi> for MenuUiRoot {
    type Params = Params<'static>;

    fn construct(ui: &mut Imm<CapsUi>, params: &mut Params) {
        ui.ch()
            .on_spawn_insert(|| Node {
                flex_direction: FlexDirection::Column,
                align_items: bevy_ui::AlignItems::Stretch,
                ..fill_parent_node()
            })
            .add(|ui| {
                ui.ch()
                    .on_spawn_insert(styles::title_text_style)
                    .on_spawn_text("Demo");
                ui.ch()
                    .on_spawn_insert(styles::text_style)
                    .on_spawn_text("bevy_immediate");

                ui.ch().on_spawn_insert(|| Node {
                    height: Val::Px(10.),
                    ..default()
                });

                for (example, title) in MENU_VARIANTS {
                    let mut button = ui
                        .ch()
                        .on_spawn_insert(styles::button_bundle)
                        .selected(example == *params.current_example)
                        .add(|ui| {
                            ui.ch()
                                .on_spawn_insert(styles::text_style)
                                .on_spawn_text(title);
                        });

                    if button.clicked() {
                        *params.current_example = example;
                    }
                }

                ui.ch().on_spawn_insert(|| Node {
                    flex_grow: 1.,
                    ..default()
                });

                let mut button = ui
                    .ch()
                    .on_spawn_insert(button_bundle)
                    .selected(params.debug_options.enabled)
                    .add(|ui| {
                        ui.ch().on_spawn_insert(text_style).text("Debug");
                    });
                if button.clicked() {
                    params.debug_options.enabled = !params.debug_options.enabled;
                }
            });
    }
}

pub const MENU_VARIANTS: [(CurrentExample, &str); 4] = [
    (CurrentExample::HelloWorld, "Hello World"),
    (CurrentExample::WidgetUse, "Widget usage"),
    (CurrentExample::ExtensionUse, "Extension usage"),
    (CurrentExample::PowerUser, "Power user"),
];

#[derive(Resource, Hash, Clone, Copy, PartialEq, Eq)]
pub enum CurrentExample {
    WidgetUse,
    HelloWorld,
    ExtensionUse,
    PowerUser,
}
