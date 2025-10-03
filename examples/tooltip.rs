use bevy::{
    color::Color,
    math::{Affine2, Vec2},
    text::TextColor,
    transform::TransformSystems,
    utils::default,
    window::{PrimaryWindow, Window},
};
use bevy_ecs::{
    component::Component,
    entity::Entity,
    hierarchy::Children,
    query::With,
    schedule::IntoScheduleConfigs,
    system::{Commands, Query, Res, Single},
};
use bevy_feathers::{
    controls::{ButtonProps, ButtonVariant, button},
    palette::WHITE,
    rounded_corners::RoundedCorners,
};
use bevy_immediate::{
    CapSet, Imm, ImmEntity, ImmId,
    attach::{BevyImmediateAttachPlugin, ImmediateAttach},
    ui::{CapsUiFeathers, ImplCapsUi, interaction::ImmUiInteraction, text::ImmUiText},
};
use bevy_input::ButtonInput;
use bevy_ui::{
    BackgroundColor, BorderColor, ComputedNode, ComputedUiRenderTargetInfo, ComputedUiTargetCamera,
    Node, UiGlobalTransform, UiRect, UiScale, UiSystems, UiTargetCamera, UiTransform, Val, px,
    ui_layout_system,
    widget::{Text, TextShadow},
};

use crate::styles::node_container;

pub struct TooltipExamplePlugin;

impl bevy_app::Plugin for TooltipExamplePlugin {
    fn build(&self, app: &mut bevy_app::App) {
        // Add bevy immediate plugin with UI support which will construct UI
        // rooted at entity with `HelloWorldRoot` component
        app.add_plugins(BevyImmediateAttachPlugin::<
            CapsUiFeathers,
            TooltipExampleRoot,
        >::new());

        app.add_systems(
            bevy_app::PostUpdate,
            position_tooltip
                .after(UiSystems::Layout)
                .before(TransformSystems::Propagate),
        );
    }
}

#[derive(Component)]
pub struct TooltipExampleRoot;

impl ImmediateAttach<CapsUiFeathers> for TooltipExampleRoot {
    type Params = (); // Access data from World using SystemParam

    fn construct(ui: &mut Imm<CapsUiFeathers>, _: &mut ()) {
        // Construct entity hierarchies
        // and attach necessary components
        ui.ch().on_spawn_insert(node_container).add(|ui| {
            for idx in 0..20 {
                ui.ch_id(idx)
                    .on_spawn_insert(|| {
                        button(
                            ButtonProps {
                                variant: ButtonVariant::Normal,
                                corners: RoundedCorners::All,
                            },
                            (),
                            (),
                        )
                    })
                    .add(|ui| {
                        ui.ch().on_spawn_insert(|| {
                            (
                                TextColor(Color::srgb(0.9, 0.9, 0.9)),
                                TextShadow::default(),
                                Text("Hello world".into()),
                            )
                        });
                    })
                    .with_tooltip(|ui| {
                        ui.ch_id(idx)
                            .on_spawn_insert(|| {
                                (
                                    Node {
                                        border: UiRect::all(px(2.)),
                                        ..default()
                                    },
                                    BackgroundColor(bevy_color::palettes::css::DARK_GRAY.into()),
                                    BorderColor::all(bevy_color::palettes::css::WHITE),
                                )
                            })
                            .add(|ui| {
                                ui.ch().on_spawn_text("Example");
                            });
                    });
            }
        });
    }
}

pub trait WithEntity<Caps: CapSet> {
    fn with_tooltip(self, f: impl FnOnce(&mut Imm<'_, '_, Caps>)) -> Self;
}

impl<Caps> WithEntity<Caps> for ImmEntity<'_, '_, '_, Caps>
where
    Caps: ImplCapsUi,
{
    fn with_tooltip(mut self, f: impl FnOnce(&mut Imm<'_, '_, Caps>)) -> Self {
        if self.hovered() {
            let entity = self.entity();
            self = self.add(|ui| {
                ui.unrooted("with_tooltip", |ui| {
                    ui.ch()
                        .on_spawn_insert(|| {
                            (
                                Node {
                                    position_type: bevy_ui::PositionType::Absolute,
                                    ..default()
                                },
                                TooltipPosition {
                                    entity,
                                    last_offset: Vec2::ZERO,
                                },
                                BackgroundColor(WHITE),
                            )
                        })
                        .add(|ui| {
                            f(ui);
                        });
                });
            });
        }
        self
    }
}

#[derive(Component)]
struct TooltipPosition {
    entity: Entity,
    last_offset: Vec2,
}

fn position_tooltip(
    tooltip: Query<(
        Entity,
        &mut TooltipPosition,
        &ComputedNode,
        &ComputedUiRenderTargetInfo,
        &mut Node,
    )>,
    computed_nodes: Query<&ComputedNode>,
    mut global_transform: Query<&mut UiGlobalTransform>,
    children: Query<&Children>,
    window: Single<&Window, With<PrimaryWindow>>,
) {
    let Some(cursor) = window.cursor_position() else {
        return;
    };

    for (entity, mut tooltip, tooltip_computed, target_info, mut node) in tooltip {
        let Ok(target_compute) = computed_nodes.get(tooltip.entity) else {
            continue;
        };
        let Ok(target_global) = global_transform.get(tooltip.entity) else {
            continue;
        };

        let target_pos = target_global.translation - target_compute.size * 0.5;

        {
            let offset = (target_pos) / target_info.scale_factor();

            if offset == tooltip.last_offset {
                continue;
            }

            tooltip.last_offset = offset;
            node.left = px(offset.x);
            node.top = px(offset.y);
        }

        let Ok(current) = global_transform.get(entity) else {
            continue;
        };

        // Logic to avoid 1 frame delay
        // Global transform update is done immediatelly
        let delta =
            Affine2::from_translation(target_pos + tooltip_computed.size * 0.5) * current.inverse();

        update_global_transforms(entity, delta, &children, &mut global_transform);
    }
}

fn update_global_transforms(
    current: Entity,
    delta: Affine2,
    children: &Query<&Children>,
    query: &mut Query<&mut UiGlobalTransform>,
) {
    if let Ok(mut global) = query.get_mut(current) {
        *global = (**global * delta).into();
    }

    let Ok(current_children) = children.get(current) else {
        return;
    };

    for &child in current_children {
        update_global_transforms(child, delta, children, query);
    }
}
