use bevy::ecs::{
    component::Component,
    resource::Resource,
    system::{ResMut, SystemParam},
};
use bevy::ui::{FlexDirection, Node, Val};
use bevy_immediate::{
    Imm, ImmEntity,
    attach::{BevyImmediateAttachPlugin, ImmediateAttach},
    ui::{CapsUi, text::ImmUiText},
};
use bevy_immediate_ui::activated::ImmUiActivated;

use crate::styles;

pub struct EguiStyleApiExamplePlugin;

impl bevy::app::Plugin for EguiStyleApiExamplePlugin {
    fn build(&self, app: &mut bevy::app::App) {
        app.add_plugins(BevyImmediateAttachPlugin::<CapsUi, EguiStyleApiExampleRoot>::new());
        app.insert_resource(EguiStyleApiState { click_count: 0 });
    }
}

#[derive(Resource, Default)]
struct EguiStyleApiState {
    click_count: u32,
}

#[derive(Component)]
pub struct EguiStyleApiExampleRoot;

#[derive(SystemParam)]
pub struct Params<'w> {
    state: ResMut<'w, EguiStyleApiState>,
}

impl ImmediateAttach<CapsUi> for EguiStyleApiExampleRoot {
    type Params = Params<'static>;

    fn construct(ui: &mut Imm<CapsUi>, params: &mut Params) {
        // egui-style ui implementation
        //
        // Inside loops, use [`Imm::with_add_id_pref`] before calling these methods.

        ui.label("Egui-style API");

        ui.column(|ui| {
            ui.row(|ui| {
                ui.column(|ui| {
                    ui.label("Labels (loop)");
                    for i in 0..5 {
                        let mut ui = ui.with_add_id_pref(i);
                        ui.label(&format!("Row item {i}"));
                    }
                });

                ui.column(|ui| {
                    ui.label("Buttons (loop)");
                    for i in 0..5 {
                        let mut ui = ui.with_add_id_pref(i);
                        if ui.button(&format!("Button {i}")).activated() {
                            params.state.click_count += 1;
                        }
                    }

                    ui.label(&format!("Clicks: {}", params.state.click_count));
                });
            });
        });
    }
}

/// Egui-style helpers using call-site ids
///
/// Use [`Imm::tch`] and implementation needs #[track_caller] annotation
pub trait EguiStyleApi<'w, 's> {
    fn label(&mut self, text: &str);
    fn button(&mut self, text: &str) -> ImmEntity<'_, 'w, 's, CapsUi>;
    fn row(&mut self, f: impl FnOnce(&mut Imm<'w, 's, CapsUi>));
    fn column(&mut self, f: impl FnOnce(&mut Imm<'w, 's, CapsUi>));
}

impl<'w, 's> EguiStyleApi<'w, 's> for Imm<'w, 's, CapsUi> {
    #[track_caller]
    fn label(&mut self, text: &str) {
        self.tch().on_spawn_insert(styles::text_style).text(text);
    }

    #[track_caller]
    fn button(&mut self, text: &str) -> ImmEntity<'_, 'w, 's, CapsUi> {
        self.tch()
            .on_spawn_insert(styles::button_bundle)
            .add(move |ui| {
                ui.tch().on_spawn_insert(styles::text_style).text(text);
            })
    }

    #[track_caller]
    fn row(&mut self, f: impl FnOnce(&mut Imm<'w, 's, CapsUi>)) {
        self.tch().on_spawn_insert(row_layout_node).add(f);
    }

    #[track_caller]
    fn column(&mut self, f: impl FnOnce(&mut Imm<'w, 's, CapsUi>)) {
        self.tch().on_spawn_insert(column_layout_node).add(f);
    }
}

fn row_layout_node() -> Node {
    let mut node = styles::node_container();
    node.flex_direction = FlexDirection::Row;
    node.column_gap = Val::Px(12.);
    node
}

fn column_layout_node() -> Node {
    let mut node = styles::node_container();
    node.flex_direction = FlexDirection::Column;
    node.flex_grow = 1.;
    node
}
