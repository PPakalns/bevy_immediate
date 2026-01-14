use bevy::ecs::system::ResMut;
use bevy::input_focus::tab_navigation::TabGroup;
use bevy::ui::Node;
use bevy_immediate::{BevyImmediatePlugin, ImmCtx, ui::CapsUi};
use bevy_immediate_ui::text::ImmUiText;

use crate::{
    anchored::AnchoredUiExampleRoot,
    bevy_scrollarea::BevyScrollareaExampleRoot,
    bevy_widgets::BevyWidgetExampleRoot,
    extension_use::ExtensionUseExampleRoot,
    floating_window::FloatingWindowRoot,
    hello_world::HelloWorldRoot,
    hot_patching::HotPatchingRoot,
    main_menu::{CurrentExample, MenuUiRoot},
    power_user::PowerUserExampleRoot,
    styles,
    // text_edit::TextEditExampleRoot,
    tooltip::TooltipExampleRoot,
    widget_use::WidgetUseExampleRoot,
};

/// Example showcases how to create
/// UI that is not attached to anything
pub struct PlainUiExamplePlugin;

impl bevy::app::Plugin for PlainUiExamplePlugin {
    fn build(&self, app: &mut bevy::app::App) {
        // Initialize plugin (it can be initialized multiple times)
        app.add_plugins(BevyImmediatePlugin::<CapsUi>::new());

        // Build your UI as a simple system
        app.add_systems(bevy::app::Update, ui_system);
    }
}

fn ui_system(ctx: ImmCtx<CapsUi>, example: ResMut<CurrentExample>) {
    // It is possible to access world data that is not
    // used by immediate mode.

    // Build your ui, provide unique id
    ctx.build_immediate_root("unique_id")
        .ch()
        .on_spawn_insert(|| {
            (
                Node {
                    flex_direction: bevy::ui::FlexDirection::Row,
                    column_gap: bevy::ui::Val::Px(10.),
                    ..styles::fill_parent_node()
                },
                TabGroup::new(0),
            )
        })
        .add(|ui| {
            // Menu container
            ui.ch_id("menu")
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
                    // We insert UI widget as an entity with UI widget root component
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
                CurrentExample::HotPatching => {
                    content.on_spawn_insert(|| HotPatchingRoot);
                }
                CurrentExample::Tooltip => {
                    content.on_spawn_insert(|| TooltipExampleRoot);
                }
                CurrentExample::FloatingWindows => {
                    content.on_spawn_insert(|| FloatingWindowRoot);
                }
                CurrentExample::Anchored => {
                    content.on_spawn_insert(|| AnchoredUiExampleRoot);
                }
                CurrentExample::TextEdit => {
                    // content.on_spawn_insert(|| TextEditExampleRoot);
                    content.text("bevy_ui_text_input not available for bevy 0.18 yet!");
                }
            }
        });
}
