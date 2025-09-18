use bevy::prelude::*;
mod utils;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        // Reusable code for demo examples and effects
        .add_plugins(utils::DemoUtilsPlugin)
        // Setup camera
        .add_systems(Startup, setup_camera)
        //
        //
        // All examples are implemented in seperate modules
        //
        .add_plugins(hello_world::HelloWorldPlugin)
        .add_plugins(plain_ui::PlainUiExamplePlugin)
        .add_plugins(widget_use::WidgetUseExamplePlugin)
        .add_plugins(widget_native::WidgetNativeExamplePlugin)
        .add_plugins(widget_functional::WidgetFunctionalExamplePlugin)
        .add_plugins(menu::MenuExamplePlugin)
        .add_plugins(extension::ExtensionExamplePlugin)
        .add_plugins(extension_use::ExtensionUseExamplePlugin)
        .run();
}

fn setup_camera(mut commands: Commands) {
    commands.spawn(Camera2d);
}

mod hello_world;

mod plain_ui;

mod widget_functional;

mod widget_native;

mod widget_use;

mod menu;

mod extension;

mod extension_use;

// mod power_user;
