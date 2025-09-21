use bevy_ecs::{
    component::Component,
    resource::Resource,
    system::{ResMut, SystemParam},
};
use bevy_immediate::{
    Imm,
    attach::{BevyImmediateAttachPlugin, ImmediateAttach},
    ui::{CapsUi, text::ImmUiText},
};
use bevy_ui::Node;

use crate::{
    styles::{self},
    widget_functional::{WidgetParams, my_functional_widget},
    widget_native::NativeWidgetComp,
};

pub struct WidgetUseExamplePlugin;

impl bevy_app::Plugin for WidgetUseExamplePlugin {
    fn build(&self, app: &mut bevy_app::App) {
        // Initialize plugin with your root component
        app.add_plugins(BevyImmediateAttachPlugin::<CapsUi, WidgetUseExampleRoot>::new());

        // Used inside ui
        app.insert_resource(FunctionalCounterValues::default());
    }
}

#[derive(Component)]
pub struct WidgetUseExampleRoot;

#[derive(SystemParam)]
pub struct Params<'w> {
    functional_counter: ResMut<'w, FunctionalCounterValues>,
}

impl ImmediateAttach<CapsUi> for WidgetUseExampleRoot {
    type Params = Params<'static>;

    fn construct(ui: &mut Imm<CapsUi>, params: &mut Params) {
        ui.ch()
            .on_spawn_insert(styles::title_text_style)
            .on_spawn_text("Widget preview");

        ui.ch()
            .on_spawn_insert(styles::text_style)
            .on_spawn_text("Embed native widgets");

        for i in 0..2 {
            ui.ch_id(("native", i))
                .on_spawn_insert(|| Node::DEFAULT)
                .on_spawn_insert(|| NativeWidgetComp {
                    title: "Count".into(),
                    counter: 45 + i * 14,
                });
        }

        ui.ch()
            .on_spawn_insert(styles::text_style)
            .on_spawn_text("Embed functional widgets");

        for (idx, counter) in params.functional_counter.values.iter_mut().enumerate() {
            ui.ch_id(("functional", idx))
                .on_spawn_insert(|| Node::DEFAULT)
                .add(|ui| {
                    my_functional_widget(
                        ui,
                        WidgetParams {
                            title: "Count",
                            counter,
                        },
                    );
                });
        }
    }
}

#[derive(Resource)]
struct FunctionalCounterValues {
    values: Vec<usize>,
}

impl Default for FunctionalCounterValues {
    fn default() -> Self {
        Self {
            values: vec![23, 555],
        }
    }
}
