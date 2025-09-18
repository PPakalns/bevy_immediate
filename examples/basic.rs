use bevy_ecs::{
    component::Component,
    resource::Resource,
    system::{ResMut, SystemParam},
};
use bevy_immediate::{
    Imm,
    attach::{BevyImmediateAttachPlugin, ImmediateAttach},
    ui::{CapUi, text::ImmUiText},
};

use crate::{
    utils::{self, node_container},
    widget_functional::{WidgetParams, my_functional_widget},
    widget_native::NativeWidgetComp,
};

pub struct BasicExamplePlugin;

impl bevy_app::Plugin for BasicExamplePlugin {
    fn build(&self, app: &mut bevy_app::App) {
        // Initialize plugin with your root component
        app.add_plugins(BevyImmediateAttachPlugin::<CapUi, BasicExampleRoot>::new());

        // Used inside ui
        app.insert_resource(FunctionalCounterValues::default());
    }
}

#[derive(Component)]
pub struct BasicExampleRoot;

#[derive(SystemParam)]
pub struct Params<'w> {
    functional_counter: ResMut<'w, FunctionalCounterValues>,
}

impl ImmediateAttach<CapUi> for BasicExampleRoot {
    type Params = Params<'static>;

    fn construct(ui: &mut Imm<CapUi>, params: &mut Params) {
        ui.ch()
            .on_spawn_insert(utils::title_text_style)
            .on_spawn_text("Basic example");

        ui.ch()
            .on_spawn_insert(utils::text_style)
            .on_spawn_text("Native widgets");

        for i in 0..2 {
            ui.ch_id(("native", i))
                .on_spawn_insert(node_container)
                .on_spawn_insert(|| NativeWidgetComp {
                    title: "Count".into(),
                    counter: 10 + i * 2,
                });
        }

        ui.ch()
            .on_spawn_insert(utils::text_style)
            .on_spawn_text("Functional widgets");

        for (idx, counter) in params.functional_counter.values.iter_mut().enumerate() {
            ui.ch_id(("functional", idx))
                .on_spawn_insert(node_container)
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
