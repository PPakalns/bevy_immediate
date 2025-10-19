use bevy::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.build().set(WindowPlugin {
            primary_window: Some(Window {
                fit_canvas_to_parent: true,
                ..default()
            }),
            ..default()
        }))
        //
        .add_plugins(bevy_framepace::FramepacePlugin) // Reduces input latency
        //
        // Reusable code for demo examples and effects
        .add_plugins(styles::DemoStylePlugin)
        // Setup camera
        .add_systems(Startup, setup_camera)
        //
        //
        // All examples are implemented in seperate modules
        //
        .add_plugins(hello_world::HelloWorldPlugin)
        .add_plugins(bevy_widgets::BevyWidgetExamplePlugin)
        .add_plugins(bevy_scrollarea::BevyScrollareaExamplePlugin)
        .add_plugins(plain_ui::PlainUiExamplePlugin)
        .add_plugins(widget_use::WidgetUseExamplePlugin)
        .add_plugins(widget_native::WidgetNativeExamplePlugin)
        .add_plugins(widget_functional::WidgetFunctionalExamplePlugin)
        .add_plugins(main_menu::MainMenuExamplePlugin)
        .add_plugins(extension::ExtensionExamplePlugin)
        .add_plugins(extension_use::ExtensionUseExamplePlugin)
        .add_plugins(power_user::PowerUserExamplePlugin)
        .add_plugins(hot_patching::HotPatchingExamplePlugin)
        .add_plugins(tooltip::TooltipExamplePlugin)
        .add_plugins(text_edit::TextEditExamplePlugin)
        .add_plugins(anchored::AnchoredUiExamplePlugin)
        .add_plugins(floating_window::FloatingWindowExamplePlugin)
        .run();
}

fn setup_camera(mut commands: Commands) {
    commands.spawn(Camera2d);
}

mod hello_world;

mod bevy_widgets;

mod bevy_scrollarea;

mod plain_ui;

mod widget_functional;

mod widget_native;

mod widget_use;

mod main_menu;

mod extension;

mod extension_use;

mod power_user;

mod hot_patching;

mod tooltip;

mod floating_window;

mod anchored;

mod text_edit;

mod styles;
