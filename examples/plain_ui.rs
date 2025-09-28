use bevy_ecs::system::ResMut;
use bevy_immediate::{BevyImmediatePlugin, ImmCtx, ui::CapsUi};
use bevy_input_focus::tab_navigation::TabGroup;
use bevy_ui::Node;

use crate::{
    bevy_scrollarea::BevyScrollareaExampleRoot,
    bevy_widgets::BevyWidgetExampleRoot,
    extension_use::ExtensionUseExampleRoot,
    hello_world::HelloWorldRoot,
    menu::{CurrentExample, MenuUiRoot},
    power_user::PowerUserExampleRoot,
    styles,
    widget_use::WidgetUseExampleRoot,
};

/// Example showcases how to create
/// UI that is not attached to anything
pub struct PlainUiExamplePlugin;

impl bevy_app::Plugin for PlainUiExamplePlugin {
    fn build(&self, app: &mut bevy_app::App) {
        // Initialize plugin (it can be initialized multiple times)
        app.add_plugins(BevyImmediatePlugin::<CapsUi>::new());

        // Build your ui as a simple system
        app.add_systems(bevy_app::Update, ui_system);
    }
}

fn ui_system(ctx: ImmCtx<CapsUi>, example: ResMut<CurrentExample>) {
    // It is possible to access world data that is not
    // used by immediate mode.
    // See bevy_immediate documentation for what is and is not used.

    // Build your ui
    ctx.build_immediate_root("unique_id")
        .ch()
        .on_spawn_insert(|| {
            (
                Node {
                    flex_direction: bevy_ui::FlexDirection::Row,
                    column_gap: bevy_ui::Val::Px(10.),
                    ..styles::fill_parent_node()
                },
                TabGroup::new(0),
            )
        })
        .add(|ui| {
            // Menu container
            ui.ch()
                .on_spawn_insert(styles::container_with_background)
                .on_spawn_insert(|| MenuUiRoot);

            // Content container
            let content = ui
                .ch_id(*example) // Changing id creates a new entity
                .on_spawn_insert(|| {
                    let mut bundle = styles::container_with_background();
                    bundle.node.flex_grow = 1.; // Fill remaining space
                    bundle
                });

            match *example {
                CurrentExample::WidgetUse => {
                    // We insert UI widget as an entity with ui widget root component
                    content.on_spawn_insert(|| WidgetUseExampleRoot);
                }
                CurrentExample::HelloWorld => {
                    content.on_spawn_insert(|| HelloWorldRoot);
                }
                CurrentExample::ExtensionUse => {
                    content.on_spawn_insert(|| ExtensionUseExampleRoot);
                }
                CurrentExample::PowerUser => {
                    content.on_spawn_insert(|| PowerUserExampleRoot);
                }
                CurrentExample::BevyWidgets => {
                    content.on_spawn_insert(|| BevyWidgetExampleRoot);
                }
                CurrentExample::BevyScrollbar => {
                    content.on_spawn_insert(|| BevyScrollareaExampleRoot);
                }
            }
        });
}
