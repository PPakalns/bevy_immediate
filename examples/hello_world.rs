use bevy::{color::Color, text::TextColor};
use bevy_ecs::component::Component;
use bevy_immediate::{
    Imm,
    attach::{BevyImmediateAttachPlugin, ImmediateAttach},
    ui::{CapUi, text::ImmUiText},
};
use bevy_ui::TextShadow;

pub struct HelloWorldPlugin;

impl bevy_app::Plugin for HelloWorldPlugin {
    fn build(&self, app: &mut bevy_app::App) {
        app.add_plugins(BevyImmediateAttachPlugin::<CapUi, HelloWorldRoot>::new());
    }
}

#[derive(Component)]
pub struct HelloWorldRoot;

impl ImmediateAttach<CapUi> for HelloWorldRoot {
    type Params = ();

    fn construct(ui: &mut Imm<CapUi>, _: &mut ()) {
        ui.ch()
            .on_spawn_insert(|| (TextColor(Color::srgb(0.9, 0.9, 0.9)), TextShadow::default())) // Add bundle to entity
            .on_spawn_text("Hello world"); // You can use extensions supported by `CapUi` 
    }
}
