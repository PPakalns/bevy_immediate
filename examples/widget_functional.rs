use bevy_immediate::{
    Imm,
    ui::{CapsUi, ImplCapsUi, picking::clicked::ImmUiClicked, text::ImmUiText},
};
use bevy_ui::{AlignItems, FlexDirection, Node, UiRect};

use crate::styles;

pub struct WidgetFunctionalExamplePlugin;

impl bevy_app::Plugin for WidgetFunctionalExamplePlugin {
    fn build(&self, _app: &mut bevy_app::App) {
        // No need to even create a plugin
    }
}

pub struct WidgetParams<'a> {
    pub title: &'a str,
    pub counter: &'a mut usize,
}

/// You can implement your functional widget as a simple function with arbitrary parameters
pub fn my_functional_widget(ui: &mut Imm<CapsUi>, value: WidgetParams) {
    // This example avoids duplciated implementation by calling generic one
    my_functional_widget_generic(ui, value);
}

/// If you develop a library, use generic functional widget variant so that users can use your widget
/// with `Caps` in which they could have additional capability support
///
/// In this case we require for at least CapsUi capabilities to be implemented
pub fn my_functional_widget_generic<Caps: ImplCapsUi>(ui: &mut Imm<Caps>, value: WidgetParams) {
    ui.ch()
        .on_spawn_insert(|| Node {
            flex_direction: FlexDirection::Row,
            align_items: AlignItems::Center,
            padding: UiRect::ZERO,
            ..styles::node_container()
        })
        .add(|ui| {
            ui.ch()
                .on_spawn_insert(styles::text_style)
                .text(format!("{}: {}", value.title, value.counter));

            let mut button = ui.ch().on_spawn_insert(styles::button_bundle).add(|ui| {
                ui.ch()
                    .on_spawn_insert(styles::text_style)
                    .on_spawn_text("-");
            });

            if button.clicked() {
                *value.counter = value.counter.saturating_sub(1);
            }

            let mut button = ui.ch().on_spawn_insert(styles::button_bundle).add(|ui| {
                ui.ch()
                    .on_spawn_insert(styles::text_style)
                    .on_spawn_text("+");
            });

            if button.clicked() {
                *value.counter = value.counter.saturating_add(1);
            }
        });
}
