use bevy_ecs::system::ResMut;
use bevy_immediate::{BevyImmediatePlugin, ImmCtx, ui::CapUi};
use bevy_ui::Node;

use crate::{
    hello_world::HelloWorldRoot,
    menu::{CurrentExample, MenuUiRoot},
    utils,
    widget_use_preview::WidgetUseExampleRoot,
};

/// Example showcases how to create
/// UI that is not attached to anything
pub struct IndependentRootPlugin;

impl bevy_app::Plugin for IndependentRootPlugin {
    fn build(&self, app: &mut bevy_app::App) {
        // Initialize plugin (it can be initialized multiple times)
        app.add_plugins(BevyImmediatePlugin::<CapUi>::new());

        // Build your ui as a simple system
        app.add_systems(bevy_app::Update, ui);
    }
}

fn ui(ctx: ImmCtx<CapUi>, example: ResMut<CurrentExample>) {
    // Build your ui
    ctx.build_immediate_root("unique_id")
        .ch()
        .on_spawn_insert(|| Node {
            flex_direction: bevy_ui::FlexDirection::Row,
            column_gap: bevy_ui::Val::Px(10.),
            ..utils::fill_parent_node()
        })
        .add(|ui| {
            // Menu container
            ui.ch()
                .on_spawn_insert(utils::container_with_background)
                .on_spawn_insert(|| MenuUiRoot);

            // Content container
            let content = ui
                .ch_id(*example) // Changing id creates a new entity
                .on_spawn_insert(|| {
                    let mut bundle = utils::container_with_background();
                    bundle.node.flex_grow = 1.; // Fill remaining space
                    bundle
                });

            match *example {
                CurrentExample::WidgetPreview => {
                    content.on_spawn_insert(|| WidgetUseExampleRoot);
                }
                CurrentExample::HelloWorld => {
                    content.on_spawn_insert(|| HelloWorldRoot);
                }
            }
        });
}
