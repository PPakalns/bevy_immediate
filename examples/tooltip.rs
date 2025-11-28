use bevy::utils::default;
use bevy_color::palettes::css::DARK_GRAY;
use bevy_ecs::{
    component::Component,
    system::{Res, SystemParam},
};
use bevy_immediate::{
    Imm,
    attach::{BevyImmediateAttachPlugin, ImmediateAttach},
    ui::{
        CapsUi,
        anchored::ImmUiAnchored,
        anchored_ui_plugin::{Anchor, AnchorOption, AnchorTarget, Direction},
        text::ImmUiText,
        tooltip_plugin::TooltipGlobalSettings,
    },
};
use bevy_ui::{BackgroundColor, BorderColor, Node, RepeatedGridTrack, UiRect, px};
use itertools::Itertools;

use crate::styles::compact_button_bundle;

pub struct TooltipExamplePlugin;

impl bevy_app::Plugin for TooltipExamplePlugin {
    fn build(&self, app: &mut bevy_app::App) {
        app.add_plugins(BevyImmediateAttachPlugin::<CapsUi, TooltipExampleRoot>::new());
    }
}

#[derive(Component)]
pub struct TooltipExampleRoot;

#[derive(SystemParam)]
pub struct Params<'w> {
    tooltip: Res<'w, TooltipGlobalSettings>,
}

impl ImmediateAttach<CapsUi> for TooltipExampleRoot {
    type Params = Params<'static>;

    fn construct(ui: &mut Imm<CapsUi>, params: &mut Params) {
        ui.ch().on_spawn_text("Hover button too see tooltip!");
        ui.ch()
            .on_spawn_insert(compact_button_bundle)
            .add(|ui| {
                ui.ch().on_spawn_text("Button");
            })
            .with_tooltip_container(|ui| {
                ui.on_spawn_insert(|| AnchorTarget::Cursor).add(|ui| {
                    ui.ch()
                        .on_spawn_insert(|| (Node::DEFAULT, BackgroundColor(DARK_GRAY.into())))
                        .add(|ui| {
                            ui.ch().on_spawn_text("Tooltip!");
                        });
                });
            });

        ui.ch().on_spawn_insert(|| Node {
            height: px(40.),
            ..default()
        });

        ui.ch().on_spawn_text(
            "Possible anchor locations: Element (TT) placed against target element (T)",
        );
        ui.ch()
            .on_spawn_insert(|| Node {
                display: bevy_ui::Display::Grid,
                grid_template_columns: vec![
                    RepeatedGridTrack::auto(1),
                    RepeatedGridTrack::fr(10, 1.),
                ],
                column_gap: px(8.),
                row_gap: px(8.),
                ..default()
            })
            .add(|ui| {
                const ANCHORS: [Anchor; 3] = [Anchor::Start, Anchor::Middle, Anchor::End];
                let anchors = || ANCHORS.into_iter().cartesian_product(ANCHORS);

                ui.ch().on_spawn_text("TT \\ T");

                ui.ch().on_spawn_text("Cursor");

                for (ty, tx) in anchors() {
                    ui.ch()
                        .on_spawn_text_fn(|| format!("{:?} {:?}", tx.sign(), ty.sign()));
                }

                let button = |ui: &mut Imm<_>, x, y, tx, ty, cursor| {
                    ui.ch_id((x, y, tx, ty, cursor))
                        .on_spawn_insert(compact_button_bundle)
                        .add(|ui| {
                            ui.ch().on_spawn_text("T");
                        })
                        .with_tooltip_container(|mut container| {
                            if cursor {
                                container = container.on_spawn_insert(|| AnchorTarget::Cursor);
                            }

                            container
                                .on_spawn_insert(|| AnchorOption {
                                    anchor: Direction { x, y },
                                    target_anchor: Direction { x: tx, y: ty },
                                    // padding: Direction {
                                    //     x: px(10.),
                                    //     y: px(10.),
                                    // },
                                    ..default()
                                })
                                .add(|ui| {
                                    ui.ch()
                                        .on_spawn_insert(|| {
                                            (
                                                Node {
                                                    border: UiRect::all(px(2.)),
                                                    ..default()
                                                },
                                                BackgroundColor(
                                                    bevy_color::palettes::css::DARK_GRAY.into(),
                                                ),
                                                BorderColor::all(bevy_color::palettes::css::WHITE),
                                            )
                                        })
                                        .add(|ui| {
                                            ui.ch().on_spawn_text("TT");
                                        });
                                });
                        });
                };

                for (y, x) in anchors() {
                    ui.ch()
                        .on_spawn_text_fn(|| format!("{:?} {:?}", x.sign(), y.sign()));

                    button(ui, x, y, Anchor::Middle, Anchor::Middle, true);

                    for (ty, tx) in anchors() {
                        button(ui, x, y, tx, ty, false);
                    }
                }
            });

        ui.ch().on_spawn_insert(|| Node {
            height: px(40.),
            ..default()
        });

        ui.ch().on_spawn_text("Tooltip global settings:");

        ui.ch()
            .text(format!("Delay: {:.2}s", params.tooltip.tooltip_delay));
        ui.ch().text(format!(
            "Pointer delay: {:.2}s",
            params.tooltip.pointer_changed_delay
        ));
        ui.ch()
            .text(format!("Reset: {:.2}s", params.tooltip.reset_delay));
    }
}
