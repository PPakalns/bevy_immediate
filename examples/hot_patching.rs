use bevy::utils::default;
use bevy_ecs::component::Component;
use bevy_immediate::{
    Imm,
    attach::{BevyImmediateAttachPlugin, ImmediateAttach},
    lch, lid,
    ui::{CapsUi, text::ImmUiText},
};
use bevy_ui::{Node, px};

use crate::styles::{self, title_text_style};

pub struct HotPatchingExamplePlugin;

impl bevy_app::Plugin for HotPatchingExamplePlugin {
    fn build(&self, app: &mut bevy_app::App) {
        // Add bevy immediate plugin with UI support which will construct UI
        // rooted at entity with `HelloWorldRoot` component
        app.add_plugins(BevyImmediateAttachPlugin::<CapsUi, HotPatchingRoot>::new());
    }
}

#[derive(Component)]
pub struct HotPatchingRoot;

impl ImmediateAttach<CapsUi> for HotPatchingRoot {
    type Params = (); // Access data from World using SystemParam

    fn construct(ui: &mut Imm<CapsUi>, _: &mut ()) {
        // There are two possible workflows:

        // 1.
        // If `hotpatching` feature is enabled for `bevy_immediate` and `bevy` crates
        //
        // it will trigger recreation of UI.

        // 2.
        // If `hotpatching` feature **is not** enabled for this crate.
        //
        // Hot patching will try to reuse already existing nodes
        //
        // This may not reload changes that are marked as `on_spawn`
        //
        // To force recreation of node:
        //
        // * Change id value passed to current `.ch_id(__)` or to any ancestor element.
        // * Comment out all code and then uncomment it.
        // * Open, close current UI,
        // * You can use `lid!()` macro which will use line and column number for id.
        //   Then node recreation will be based on source code location.
        // This library will try to develop a better way in future:
        // * like inserting compilation time as id :)

        // To execute demo with hotpatching: See bevy_immediate README.md "Hotpatching" section.

        lch!(ui)
            .on_spawn_insert(title_text_style)
            .on_spawn_text("Hot patching ");

        ui.ch_id(lid!())
            .on_spawn_text("Install dx tool and launch demo example with hotpatching");

        ui.ch_id(lid!())
            .on_spawn_text("(See bevy_immediate README.md \"Hotpatching\" section)");

        lch!(ui).on_spawn_insert(|| Node {
            height: px(50.),
            ..default()
        });

        lch!(ui).on_spawn_text("COMMENT ME in code!");

        // lch!(ui).on_spawn_text("UNCOMMENT ME!");

        lch!(ui).on_spawn_insert(|| Node {
            height: px(50.),
            ..default()
        });

        for idx in 0..3 {
            ui.ch_id((lid!(), idx))
                .on_spawn_insert(styles::button_bundle)
                .add(|ui| {
                    ui.ch().on_spawn_text("Example 2");
                });
        }
    }
}
