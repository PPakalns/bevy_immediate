use bevy::{color::Color, text::TextColor, utils::default};
use bevy_ecs::component::Component;
use bevy_immediate::{
    Imm,
    attach::{BevyImmediateAttachPlugin, ImmediateAttach},
    ui::{CapUi, text::ImmUiText},
};
use bevy_ui::{
    BackgroundColor, BorderColor, BorderRadius, FlexDirection, Node, TextShadow, UiRect, Val,
};

use crate::utils::{BACKGROUND, MyStyleBundle};

pub struct HelloWorldPlugin;

impl bevy_app::Plugin for HelloWorldPlugin {
    fn build(&self, app: &mut bevy_app::App) {
        // Add plugin
        app.add_plugins(BevyImmediateAttachPlugin::<CapUi, HelloWorldRoot>::new());
    }
}

#[derive(Component)]
pub struct HelloWorldRoot;

impl ImmediateAttach<CapUi> for HelloWorldRoot {
    type Params = ();

    fn construct(ui: &mut Imm<CapUi>, _: &mut ()) {
        // Construct entity hierarchies
        // and attach necessary components
        ui.ch()
            .on_spawn_insert(|| MyStyleBundle {
                node: Node {
                    flex_direction: FlexDirection::Column,
                    border: UiRect::all(Val::Px(10.)),
                    padding: UiRect::all(Val::Px(10.)),
                    column_gap: Val::Px(10.),
                    row_gap: Val::Px(10.),
                    ..default()
                },
                border_color: BorderColor(Color::srgb(1., 0., 0.)),
                border_radius: BorderRadius::all(Val::Px(5.)),
                background_color: BackgroundColor(BACKGROUND),
            })
            .add(|ui| {
                ui.ch()
                    .on_spawn_insert(|| {
                        (TextColor(Color::srgb(0.9, 0.9, 0.9)), TextShadow::default())
                    })
                    // You can use extensions supported by `CapUi`
                    // See [`bevy_immediate::ui::CapUi`]
                    .on_spawn_text("Hello world");
            });
    }
}
