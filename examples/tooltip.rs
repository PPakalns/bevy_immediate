use bevy::{
    color::Color,
    math::Vec2,
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
    system::{Query, Single},
};
use bevy_feathers::{
    controls::{ButtonProps, ButtonVariant, button},
    rounded_corners::RoundedCorners,
};
use bevy_immediate::{
    CapSet, Imm, ImmEntity,
    attach::{BevyImmediateAttachPlugin, ImmediateAttach},
    ui::{CapsUiFeathers, ImplCapsUi, interaction::ImmUiInteraction, text::ImmUiText},
};
use bevy_ui::{
    BackgroundColor, BorderColor, ComputedNode, ComputedUiRenderTargetInfo, Node,
    RepeatedGridTrack, UiGlobalTransform, UiRect, UiSystems, Val, px,
    widget::{Text, TextShadow},
};
use itertools::Itertools;

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

        ui.ch().on_spawn_insert(|| {
            Text("Anchoring: Element (TT) positioning against target element (T)".into())
        });

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

                ui.ch().on_spawn_insert(|| Text("TT \\ T".into()));

                ui.ch().on_spawn_insert(|| Text(format!("Cursor")));

                for (ty, tx) in anchors() {
                    ui.ch()
                        .on_spawn_insert(|| Text(format!("{:?} {:?}", tx.sign(), ty.sign())));
                }

                let button = |ui: &mut Imm<_>, x, y, tx, ty, cursor| {
                    ui.ch_id((x, y, tx, ty, cursor))
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
                                    Text("T".into()),
                                )
                            });
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
                        .on_spawn_insert(|| Text(format!("{:?} {:?}", x.sign(), y.sign())));

                    button(ui, x, y, Anchor::Middle, Anchor::Middle, true);

                    for (ty, tx) in anchors() {
                        button(ui, x, y, tx, ty, false);
                    }
                }
            });
    }
}

pub trait WithEntity<'w, 's, Caps: CapSet> {
    fn with_tooltip(self, f: impl FnOnce(&mut Imm<'w, 's, Caps>)) -> Self;
    fn with_tooltip_container(self, f: impl FnOnce(ImmEntity<'_, 'w, 's, Caps>)) -> Self;
}

impl<'w, 's, Caps> WithEntity<'w, 's, Caps> for ImmEntity<'_, 'w, 's, Caps>
where
    Caps: ImplCapsUi,
{
    fn with_tooltip(self, f: impl FnOnce(&mut Imm<'w, 's, Caps>)) -> Self {
        self.with_tooltip_container(|entity| {
            entity.add(|ui| {
                f(ui);
            });
        })
    }

    fn with_tooltip_container(mut self, f: impl FnOnce(ImmEntity<'_, 'w, 's, Caps>)) -> Self {
        if self.hovered() {
            let entity = self.entity();
            self = self.add(|ui| {
                ui.unrooted("with_tooltip", |ui| {
                    let entity = ui.ch().on_spawn_insert(|| {
                        (
                            Node {
                                position_type: bevy_ui::PositionType::Absolute,
                                ..default()
                            },
                            AnchorTarget::Entity(entity),
                        )
                    });
                    f(entity);
                });
            });
        }
        self
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub enum Anchor {
    Start,
    Middle,
    End,
}

impl Anchor {
    fn sign(&self) -> i32 {
        match self {
            Anchor::Start => -1,
            Anchor::Middle => 0,
            Anchor::End => 1,
        }
    }
}

#[derive(Component, Clone, Copy, PartialEq)]
pub struct AnchorOption {
    /// Anchor location for element to place
    anchor: Direction<Anchor>,
    /// Anchor location for element that this element is placed against
    target_anchor: Direction<Anchor>,
    /// Additional padding to location where element will be placed
    /// Padding is ignored for Middle anchor locations
    padding: Direction<Val>,
}

#[derive(Component)]
pub struct AnchorMouse;

impl Default for AnchorOption {
    fn default() -> Self {
        Self {
            anchor: Direction {
                x: Anchor::Start,
                y: Anchor::Start,
            },
            target_anchor: Direction {
                x: Anchor::Start,
                y: Anchor::End,
            },
            padding: Direction {
                x: Val::ZERO,
                y: Val::ZERO,
            },
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
struct Direction<T> {
    x: T,
    y: T,
}

impl<T> Direction<T> {
    pub fn map<O>(&self, f: impl Fn(&T) -> O) -> Direction<O> {
        Direction {
            x: f(&self.x),
            y: f(&self.y),
        }
    }
}

impl Direction<Anchor> {
    fn sign_vec(&self) -> Vec2 {
        self.map(|v| v.sign() as f32).into()
    }
}

impl From<Direction<f32>> for Vec2 {
    fn from(value: Direction<f32>) -> Self {
        Vec2 {
            x: value.x,
            y: value.y,
        }
    }
}

#[derive(Component)]
#[require(PlacementCache, AnchorOption)]
pub enum AnchorTarget {
    Entity(Entity),
    Cursor,
    PhysicalPosition(Vec2),
}

#[derive(Component, Default)]
struct PlacementCache {
    last_offset: Option<Vec2>,
}

fn position_tooltip(
    tooltip: Query<(
        Entity,
        &AnchorTarget,
        &mut PlacementCache,
        &AnchorOption,
        &ComputedNode,
        &ComputedUiRenderTargetInfo,
        &mut Node,
    )>,
    computed_nodes: Query<&ComputedNode>,
    mut global_transform: Query<&mut UiGlobalTransform>,
    children: Query<&Children>,
    window: Single<&Window, With<PrimaryWindow>>,
) {
    for (
        entity,
        target,
        mut placement_cache,
        anchor_option,
        tooltip_computed,
        tooltip_target_info,
        mut node,
    ) in tooltip
    {
        let cursor = window.physical_cursor_position();

        let target_position: Vec2 = match &*target {
            AnchorTarget::Entity(entity) => (|| -> _ {
                let target_compute = computed_nodes.get(*entity).ok()?;
                let target_global = global_transform.get(*entity).ok()?;

                let anchor_offset = Vec2::from(anchor_option.target_anchor.sign_vec());
                let target_pos =
                    target_global.translation + anchor_offset * 0.5 * target_compute.size;

                Some(target_pos)
            })()
            .unwrap_or(Vec2::ZERO),
            AnchorTarget::Cursor => cursor.unwrap_or(Vec2::ZERO),
            AnchorTarget::PhysicalPosition(pos) => *pos,
        };
        let target_position = target_position.round();

        let tooltip_anchor_offset = {
            let anchor_sign_vec = anchor_option.anchor.sign_vec();

            let anchor_offset = anchor_sign_vec * 0.5 * tooltip_computed.size;

            let x = anchor_option
                .padding
                .x
                .resolve(
                    tooltip_target_info.scale_factor(),
                    tooltip_target_info.physical_size().x as f32,
                    tooltip_target_info.physical_size().as_vec2(),
                )
                .unwrap_or(0.);

            let y = anchor_option
                .padding
                .y
                .resolve(
                    tooltip_target_info.scale_factor(),
                    tooltip_target_info.physical_size().y as f32,
                    tooltip_target_info.physical_size().as_vec2(),
                )
                .unwrap_or(0.);

            anchor_offset + anchor_sign_vec * Vec2 { x, y }
        };

        let final_position = target_position - tooltip_anchor_offset;

        if placement_cache.last_offset == Some(final_position) {
            continue;
        }
        placement_cache.last_offset = Some(final_position);

        {
            let offset =
                (final_position - tooltip_computed.size * 0.5) / tooltip_target_info.scale_factor();
            node.left = px(offset.x);
            node.top = px(offset.y);
        }

        let Ok(current) = global_transform.get(entity) else {
            continue;
        };

        // Logic to avoid 1 frame delay
        // Global transform update is done immediatelly
        let delta = final_position - current.translation;

        update_global_transforms(entity, delta, &children, &mut global_transform);
    }
}

fn update_global_transforms(
    current: Entity,
    delta: Vec2,
    children: &Query<&Children>,
    query: &mut Query<&mut UiGlobalTransform>,
) {
    if let Ok(mut global) = query.get_mut(current) {
        let mut transformation = **global;
        transformation.translation += delta;
        *global = transformation.into();
    }

    let Ok(current_children) = children.get(current) else {
        return;
    };

    for &child in current_children {
        update_global_transforms(child, delta, children, query);
    }
}
