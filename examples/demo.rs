use bevy::{prelude::*, winit::WinitSettings};
mod utils;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        // Update only on user input
        .insert_resource(WinitSettings::desktop_app())
        // Reusable code for demo examples and effects
        .add_plugins(utils::DemoUtilsPlugin)
        // Setup camera
        .add_systems(Startup, setup_camera)
        //
        //
        // All examples are implemented in seperate modules
        //
        .add_plugins(hello_world::HelloWorldPlugin)
        .add_plugins(menu::ExampleMenuPlugin)
        .add_plugins(independent_root::IndependentRootPlugin)
        .add_plugins(widget_use_preview::WidgetUseExamplePlugin)
        .add_plugins(widget_native::WidgetNativePlugin)
        .add_plugins(widget_functional::WidgetFunctionalPlugin)
        .run();
}

fn setup_camera(mut commands: Commands) {
    commands.spawn(Camera2d);
}

mod hello_world;

mod independent_root;

mod widget_functional;

mod widget_native;

mod widget_use_preview;

mod menu;
