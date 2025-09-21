use bevy_ecs::{
    component::Component,
    system::{Query, SystemParam},
};
use bevy_immediate::{
    Imm,
    attach::{BevyImmediateAttachPlugin, ImmediateAttach},
    ui::{CapsUi, picking::clicked::ImmUiClicked, text::ImmUiText},
};
use bevy_ui::{AlignItems, FlexDirection, Node, UiRect};

use crate::styles;

pub struct WidgetNativeExamplePlugin;

impl bevy_app::Plugin for WidgetNativeExamplePlugin {
    fn build(&self, app: &mut bevy_app::App) {
        app.add_plugins(BevyImmediateAttachPlugin::<CapsUi, NativeWidgetComp>::new());
    }
}

#[derive(Component)]
pub struct NativeWidgetComp {
    pub title: String,
    pub counter: usize,
}

#[derive(SystemParam)]
pub struct Params<'w, 's> {
    query: Query<'w, 's, &'static mut NativeWidgetComp>,
}

impl ImmediateAttach<CapsUi> for NativeWidgetComp {
    type Params = Params<'static, 'static>;

    fn construct(ui: &mut Imm<CapsUi>, params: &mut Params) {
        let entity = ui.current_entity().unwrap();

        // Value can be stored inside component
        let mut value = params.query.get_mut(entity).unwrap();

        ui.ch()
            .on_spawn_insert(|| Node {
                flex_direction: FlexDirection::Row,
                align_items: AlignItems::Center,
                padding: UiRect::ZERO,
                ..styles::node_container()
            })
            .add(|ui| {
                let change_detector = ui.change_detector();

                ui.ch()
                    .on_spawn_insert(styles::text_style)
                    // Change detection can be used to
                    // update bevy only when something has changed.
                    .on_change_text_fn(change_detector.has_changed(&value), || {
                        format!("{}: {}", value.title, value.counter)
                    });

                let mut button = ui.ch().on_spawn_insert(styles::button_bundle).add(|ui| {
                    ui.ch()
                        .on_spawn_insert(styles::text_style)
                        .on_spawn_text("-");
                });
                if button.clicked() {
                    value.counter = value.counter.saturating_sub(1);
                }

                let mut button = ui.ch().on_spawn_insert(styles::button_bundle).add(|ui| {
                    ui.ch()
                        .on_spawn_insert(styles::text_style)
                        .on_spawn_text("+");
                });
                if button.clicked() {
                    value.counter = value.counter.saturating_add(1);
                }
            });
    }
}
