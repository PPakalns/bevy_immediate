use std::alloc::Layout;

use bevy::{
    color::Color,
    math::Vec2,
    render::render_resource::encase::private::StructMetadata,
    text::TextColor,
    transform::TransformSystems,
    utils::default,
    window::{PrimaryWindow, Window},
};
use bevy_app::{HierarchyPropagatePlugin, PostUpdate, Propagate};
use bevy_color::{
    Srgba,
    palettes::css::{DARK_GRAY, LIGHT_GRAY},
};
use bevy_ecs::{
    component::Component,
    entity::Entity,
    event::PropagateEntityTrigger,
    hierarchy::{ChildOf, Children},
    observer::On,
    query::With,
    resource::Resource,
    schedule::IntoScheduleConfigs,
    system::{
        Commands, Query, Res, ResMut, Single,
        lifetimeless::{SRes, SResMut},
    },
};
use bevy_feathers::{
    controls::{self, ButtonProps, ButtonVariant, button},
    rounded_corners::RoundedCorners,
};
use bevy_immediate::{
    CapSet, Imm, ImmEntity,
    attach::{BevyImmediateAttachPlugin, ImmediateAttach},
    imm_id,
    ui::{
        CapsUiFeathers, ImplCapsUi, activated::ImmUiActivated,
        button_variant::ImmUiFeathersButtonVariant, interaction::ImmUiInteraction, text::ImmUiText,
    },
};
use bevy_picking::{
    Pickable,
    events::{Drag, DragStart, Pointer, Press},
};
use bevy_platform::collections::HashSet;
use bevy_ui::{
    BackgroundColor, BorderColor, ComputedNode, ComputedUiRenderTargetInfo, FlexDirection,
    GlobalZIndex, LayoutConfig, Node, RepeatedGridTrack, UiGlobalTransform, UiRect, UiScale,
    UiSystems, Val, px, ui_layout_system,
    widget::{Text, TextShadow},
};
use itertools::Itertools;

use crate::{
    bevy_widgets::BevyWidgetExampleRoot,
    styles::{node_container, row_node_container},
};

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
            position_anchor
                .after(UiSystems::Layout)
                .before(TransformSystems::Propagate),
        );

        app.add_plugins(HierarchyPropagatePlugin::<Pickable>::new(
            bevy_app::PostUpdate,
        ));

        app.insert_resource(State {
            open: vec![false, false, false, false],
        });

        app.add_observer(window_on_drag_start)
            .add_observer(window_on_drag)
            .add_observer(window_on_focus);
        app.add_systems(
            bevy_app::PostUpdate,
            update_ui_layer_order.before(UiSystems::Prepare),
        );
        app.add_observer(update_should_close);
    }
}

#[derive(Resource)]
pub struct State {
    open: Vec<bool>,
}

#[derive(Component)]
pub struct TooltipExampleRoot;

impl ImmediateAttach<CapsUiFeathers> for TooltipExampleRoot {
    type Params = SResMut<State>; // Access data from World using SystemParam

    fn construct(ui: &mut Imm<CapsUiFeathers>, state: &mut ResMut<State>) {
        // Construct entity hierarchies
        // and attach necessary components

        ui.ch().on_spawn_insert(|| Text("Floating windows".into()));

        ui.ch().on_spawn_insert(row_node_container).add(|ui| {
            for open in state.open.iter_mut() {
                let mut button = ui
                    .ch()
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
                    .primary_button(*open)
                    .add(|ui| {
                        ui.ch()
                            .on_spawn_insert(|| {
                                (TextColor(Color::srgb(0.9, 0.9, 0.9)), TextShadow::default())
                            })
                            .on_spawn_text("Window");
                    });
                if button.activated() {
                    *open = !*open;
                }

                if *open {
                    button.unrooted("my_ui", |ui| {
                        ui.ch()
                            .on_spawn_insert(|| {
                                (
                                    Node {
                                        flex_direction: FlexDirection::Column,
                                        border: px(2.).into(),
                                        ..default()
                                    },
                                    FloatingWindow,
                                    BackgroundColor(bevy_feathers::palette::BLACK),
                                    BorderColor::all(LIGHT_GRAY),
                                )
                            })
                            .add(|ui| {
                                ui.ch()
                                    .on_spawn_insert(|| {
                                        (
                                            Node {
                                                flex_direction: FlexDirection::Row,
                                                justify_content:
                                                    bevy_ui::JustifyContent::SpaceBetween,
                                                align_items: bevy_ui::AlignItems::Stretch,
                                                ..default()
                                            },
                                            BackgroundColor(
                                                Srgba::new(0.363, 0.363, 0.363, 1.0).into(),
                                            ),
                                        )
                                    })
                                    .add(|ui| {
                                        ui.ch()
                                            .on_spawn_insert(|| Node {
                                                flex_grow: 1.,
                                                justify_content: bevy_ui::JustifyContent::Center,
                                                ..default()
                                            })
                                            .add(|ui| {
                                                ui.ch().on_spawn_text("Title");
                                            });

                                        ui.ch().on_spawn_insert(|| Node::DEFAULT).add(|ui| {
                                            let mut close = ui
                                                .ch()
                                                .on_spawn_insert(|| {
                                                    controls::button(
                                                        ButtonProps {
                                                            variant: ButtonVariant::Primary,
                                                            corners: RoundedCorners::None,
                                                        },
                                                        (),
                                                        (),
                                                    )
                                                })
                                                .add(|ui| {
                                                    ui.ch().on_spawn_text("X");
                                                });
                                            if close.activated() {
                                                *open = !*open;
                                            }
                                        });
                                    });

                                ui.ch().on_spawn_insert(|| {
                                    (
                                        Node {
                                            flex_direction: FlexDirection::Column,
                                            ..node_container()
                                        },
                                        BevyWidgetExampleRoot,
                                    )
                                });
                            });
                    });
                }
            }
        });

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
                            controls::button(
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

        ui.ch().on_spawn_text("Dropdown");

        ui.ch().on_spawn_insert(row_node_container).add(|ui| {
            for (x, y, tx, ty) in [
                (Anchor::Start, Anchor::End, Anchor::Start, Anchor::Start),
                (Anchor::Start, Anchor::Start, Anchor::Start, Anchor::End),
            ] {
                let mut button = ui
                    .ch_id((x, y, tx, ty))
                    .on_spawn_insert(|| {
                        controls::button(
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
                                Text("Menu".into()),
                            )
                        });
                    });

                // Dropdown state management using on entity stored value
                let menu_open_id = imm_id("activated");
                if button.activated() {
                    button.hash_set(menu_open_id, imm_id(()));
                }

                let mut is_open_value = button.hash_get(menu_open_id);

                button = button.primary_button(is_open_value.is_some());

                if is_open_value.is_some() {
                    button = button.with_dropdown_container(
                        || {
                            is_open_value = None;
                        },
                        |container| {
                            container
                                .on_spawn_insert(|| AnchorOption {
                                    anchor: Direction { x, y },
                                    target_anchor: Direction { x: tx, y: ty },
                                    ..default()
                                })
                                .add(|ui| {
                                    dropdown_content(ui);
                                });
                        },
                    );
                }

                button.hash_update(menu_open_id, is_open_value);
            }
        });
    }
}

fn dropdown_content(ui: &mut Imm<'_, '_, CapsUiFeathers>) {
    let mut menu_container = ui.ch().on_spawn_insert(|| {
        (
            Node {
                flex_direction: FlexDirection::Column,
                border: UiRect::all(px(2.)),
                padding: px(4.).into(),
                row_gap: px(4.),
                ..default()
            },
            BackgroundColor(bevy_color::palettes::css::BLACK.into()),
            BorderColor::all(bevy_color::palettes::css::DARK_GRAY),
        )
    });

    // On ImmEntity for the lifetime of entity custom hashed values can be stored
    //
    // We will use locally stored hash value to store current menu choice
    let menu_hash_id = imm_id("menu_value");
    let mut stored_hash = menu_container.hash_get(menu_hash_id);

    menu_container = menu_container.add(|ui| {
        for _ in 0..10 {
            let mut button = ui
                .ch()
                .on_spawn_insert(|| {
                    controls::button(
                        ButtonProps {
                            variant: ButtonVariant::Normal,
                            corners: RoundedCorners::None,
                        },
                        (),
                        (),
                    )
                })
                .add(|ui| {
                    ui.ch().on_spawn_text("Example");
                });

            let button_id = button.imm_id();
            if button.hovered() {
                stored_hash = Some(button_id);
            }

            button = button.primary_button(stored_hash == Some(button_id));

            if stored_hash == Some(button_id) {
                button.with_dropdown_container(
                    || {
                        stored_hash = None;
                    },
                    |container| {
                        container
                            .on_spawn_insert(|| AnchorOption {
                                anchor: Direction {
                                    x: Anchor::Start,
                                    y: Anchor::Start,
                                },
                                target_anchor: Direction {
                                    x: Anchor::End,
                                    y: Anchor::Start,
                                },
                                ..default()
                            })
                            .add(|ui| {
                                ui.ch()
                                    .on_spawn_insert(|| {
                                        (
                                            Node {
                                                flex_direction: FlexDirection::Column,
                                                border: UiRect::all(px(2.)),
                                                padding: px(4.).into(),
                                                row_gap: px(4.),
                                                ..default()
                                            },
                                            BackgroundColor(
                                                bevy_color::palettes::css::BLACK.into(),
                                            ),
                                            BorderColor::all(bevy_color::palettes::css::DARK_GRAY),
                                        )
                                    })
                                    .add(|ui| {
                                        for _ in 0..3 {
                                            ui.ch()
                                                .on_spawn_insert(|| {
                                                    controls::button(
                                                        ButtonProps {
                                                            variant: ButtonVariant::Normal,
                                                            corners: RoundedCorners::None,
                                                        },
                                                        (),
                                                        (),
                                                    )
                                                })
                                                .add(|ui| {
                                                    ui.ch().on_spawn_text("Final button");
                                                });
                                        }
                                    });
                            });
                    },
                );
            }
        }
    });

    menu_container.hash_update(menu_hash_id, stored_hash);
}

pub trait ImmUiDropdown<'w, 's, Caps: CapSet> {
    fn with_dropdown(self, on_close: impl FnOnce(), f: impl FnOnce(&mut Imm<'w, 's, Caps>))
    -> Self;
    fn with_dropdown_container(
        self,
        on_close: impl FnOnce(),
        f: impl FnOnce(ImmEntity<'_, 'w, 's, Caps>),
    ) -> Self;
}

impl<'w, 's, Caps> ImmUiDropdown<'w, 's, Caps> for ImmEntity<'_, 'w, 's, Caps>
where
    Caps: ImplCapsUi,
{
    fn with_dropdown(
        self,
        on_close: impl FnOnce(),
        f: impl FnOnce(&mut Imm<'w, 's, Caps>),
    ) -> Self {
        self.with_dropdown_container(on_close, |entity| {
            entity.add(|ui| {
                f(ui);
            });
        })
    }

    fn with_dropdown_container(
        mut self,
        on_close: impl FnOnce(),
        f: impl FnOnce(ImmEntity<'_, 'w, 's, Caps>),
    ) -> Self {
        let entity = self.entity();
        let mut should_close = false;
        self = self.add(|ui| {
            ui.unrooted("with_dropdown", |ui| {
                let entity = ui.ch().on_spawn_insert(|| {
                    (
                        Node {
                            position_type: bevy_ui::PositionType::Absolute,
                            ..default()
                        },
                        UiZOrderLayer::Dropdown,
                        AnchorTarget::Entity(entity),
                        FocusParent(entity),
                        FocusDetectShouldClose,
                    )
                });

                if entity.cap_entity_contains::<FocusShouldClose>() {
                    should_close = true;
                }

                f(entity);
            });
        });

        // Should close is called at the end (1 frame delay)
        // to process possible updates in dropdown that could
        // have called dropdown to close.
        if should_close {
            on_close();
        }

        self
    }
}

pub trait ImmUiTooltip<'w, 's, Caps: CapSet> {
    fn with_tooltip(self, f: impl FnOnce(&mut Imm<'w, 's, Caps>)) -> Self;
    fn with_tooltip_container(self, f: impl FnOnce(ImmEntity<'_, 'w, 's, Caps>)) -> Self;
}

impl<'w, 's, Caps> ImmUiTooltip<'w, 's, Caps> for ImmEntity<'_, 'w, 's, Caps>
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
                            UiZOrderLayer::Tooltip,
                            FocusParent(entity),
                            Propagate(Pickable {
                                should_block_lower: false,
                                is_hoverable: false,
                            }),
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

impl Default for AnchorOption {
    fn default() -> Self {
        Self {
            anchor: Direction {
                x: Anchor::Start,
                y: Anchor::End,
            },
            target_anchor: Direction {
                x: Anchor::Middle,
                y: Anchor::Start,
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

fn position_anchor(
    elements_to_anchor: Query<(
        Entity,
        &AnchorTarget,
        &mut PlacementCache,
        Option<&LayoutConfig>,
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
        layout_config,
        anchor_option,
        comp_node,
        comp_target_info,
        mut node,
    ) in elements_to_anchor
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

        let element_anchor_offset = {
            let anchor_sign_vec = anchor_option.anchor.sign_vec();

            let anchor_offset = anchor_sign_vec * 0.5 * comp_node.size;

            let x = anchor_option
                .padding
                .x
                .resolve(
                    comp_target_info.scale_factor(),
                    comp_target_info.physical_size().x as f32,
                    comp_target_info.physical_size().as_vec2(),
                )
                .unwrap_or(0.);

            let y = anchor_option
                .padding
                .y
                .resolve(
                    comp_target_info.scale_factor(),
                    comp_target_info.physical_size().y as f32,
                    comp_target_info.physical_size().as_vec2(),
                )
                .unwrap_or(0.);

            anchor_offset + anchor_sign_vec * Vec2 { x, y }
        };

        let final_position = target_position - element_anchor_offset;

        if placement_cache.last_offset == Some(final_position) {
            continue;
        }
        placement_cache.last_offset = Some(final_position);

        apply_position(
            entity,
            final_position,
            &mut node,
            comp_node,
            layout_config,
            &children,
            &mut global_transform,
        );
    }
}

fn apply_position(
    entity: Entity,
    mut final_position: Vec2,
    node: &mut Node,
    comp_node: &ComputedNode,
    layout_config: Option<&LayoutConfig>,
    children: &Query<&Children>,
    mut global_transform: &mut Query<&mut UiGlobalTransform>,
) {
    {
        let mut offset = final_position - comp_node.size * 0.5;

        // This is needed to avoid 1px broken layouts where something doesn't align up correctly
        if layout_config.map(|lc| lc.use_rounding).unwrap_or(true) {
            let offset_rounded = offset.round();
            final_position += offset_rounded - offset; // Get final position in correct place
            offset = offset_rounded;
        }

        let offset_px = offset * comp_node.inverse_scale_factor;
        node.left = px(offset_px.x);
        node.top = px(offset_px.y);
    }

    let Ok(current) = global_transform.get(entity) else {
        return;
    };

    // Logic to avoid 1 frame delay
    // Global transform update is done immediatelly
    let delta = final_position - current.translation;

    update_global_transforms(entity, delta, &children, &mut global_transform);
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

#[derive(Component)]
#[require(FloatingWindowDrag, UiZOrderLayer::Window)]
pub struct FloatingWindow;

#[derive(Component, Default)]
struct FloatingWindowDrag {
    initial_pos: Vec2,
    last_offset: Option<Vec2>,
}

fn window_on_drag_start(
    mut drag_start: On<Pointer<DragStart>>,
    mut scroll_position_query: Query<
        (&UiGlobalTransform, &mut FloatingWindowDrag),
        With<FloatingWindowDrag>,
    >,
) {
    if let Ok((transform, mut state)) = scroll_position_query.get_mut(drag_start.entity) {
        state.initial_pos = transform.translation;
        state.last_offset = None;

        drag_start.propagate(false);
    }
}

fn window_on_drag(
    mut drag: On<Pointer<Drag>>,
    mut scroll_position_query: Query<
        (
            &mut FloatingWindowDrag,
            &mut Node,
            &ComputedNode,
            Option<&LayoutConfig>,
        ),
        With<FloatingWindowDrag>,
    >,
    ui_scale: Res<UiScale>,
    mut global_transform: Query<&mut UiGlobalTransform>,
    children: Query<&Children>,
) {
    let Ok((mut state, mut node, comp_node, layout_config)) =
        scroll_position_query.get_mut(drag.entity)
    else {
        return;
    };

    drag.propagate(false);

    let distance = drag.distance / (comp_node.inverse_scale_factor * ui_scale.0);
    let target_position = state.initial_pos + distance;

    if state.last_offset == Some(target_position) {
        return;
    }
    state.last_offset = Some(target_position);

    apply_position(
        drag.entity,
        target_position,
        &mut node,
        comp_node,
        layout_config,
        &children,
        &mut global_transform,
    );
}

fn window_on_focus(
    pointer: On<Pointer<Press>>,
    mut windows: Query<&mut UiBringForward, With<FloatingWindow>>,
    child_of: Query<&ChildOf>,
) {
    if pointer.original_event_target() != pointer.entity {
        return;
    }

    let root_entity = child_of.root_ancestor(pointer.entity);

    let Ok(mut window) = windows.get_mut(root_entity) else {
        return;
    };

    window.forward = true;
}

#[derive(Component)]
pub struct UiBringForward {
    forward: bool,
}

#[derive(Component, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
#[require(UiBringForward { forward: true }, GlobalZIndex)]
pub enum UiZOrderLayer {
    Window,
    Dropdown,
    Popup,
    Tooltip,
}

impl UiZOrderLayer {
    pub fn base(&self) -> i32 {
        match self {
            UiZOrderLayer::Window => 1000,
            UiZOrderLayer::Dropdown => 2000,
            UiZOrderLayer::Popup => 3000,
            UiZOrderLayer::Tooltip => 4000,
        }
    }
}

fn update_ui_layer_order(
    mut layer_roots: Query<(
        Entity,
        &mut UiBringForward,
        &mut GlobalZIndex,
        &UiZOrderLayer,
    )>,
) {
    let mut process = false;
    for layer in layer_roots.iter_mut() {
        if layer.1.forward {
            process = true;
            break;
        }
    }

    // Everything in correct order
    if !process {
        return;
    }

    // Sort layers by
    // UiZOrderLayer, (false - keep order, true - bring forward), current global z index, entity
    let mut layers: Vec<(UiZOrderLayer, bool, i32, Entity)> = vec![];

    for mut layer in layer_roots.iter_mut() {
        if layer.1.forward {
            layers.push((*layer.3, true, layer.2.0, layer.0));
            layer.1.forward = false;
        } else {
            layers.push((*layer.3, false, layer.2.0, layer.0));
        }
    }

    layers.sort_unstable();

    let mut current_layer = None;
    let mut current_z_index = 0;

    for (layer, _, _, entity) in layers.into_iter() {
        let (_, _, mut global_z, _) = layer_roots.get_mut(entity).unwrap();

        if current_layer != Some(layer) {
            current_layer = Some(layer);
            current_z_index = layer.base();
        }

        if global_z.0 != current_z_index {
            global_z.0 = current_z_index;
        }

        current_z_index += 1;
    }
}

const WINDOW_Z_INDEX_BASE: i32 = 1000;

/// If contents of entity are interacted with, focus should be kept
/// for given focus parent entity and its parents recursively
///
/// Component should be attached to root entity
#[derive(Component)]
pub struct FocusParent(pub Entity);

/// Add component to Ui tree root entity
///
/// This component will track if UI should be closed
/// in situations where something else is pressed on screen.
///
/// [`FocusShouldClose`]  will be inserted if ui should close.
#[derive(Component)]
pub struct FocusDetectShouldClose;

/// Informs that something else was focused and view should close
#[derive(Component)]
pub struct FocusShouldClose;

fn update_should_close(
    pointer: On<Pointer<Press>>,
    should_close: Query<Entity, With<FocusDetectShouldClose>>,
    focus_parents: Query<&FocusParent>,
    child_of: Query<&ChildOf>,
    mut commands: Commands,
) {
    if pointer.original_event_target() != pointer.entity {
        return;
    }

    // No elements to close
    if should_close.is_empty() {
        return;
    }

    let mut keep_open = HashSet::new();

    let mut current_entity = Some(pointer.entity);
    while let Some(entity) = current_entity.take() {
        let root_entity = child_of.root_ancestor(entity);

        if should_close.contains(root_entity) {
            keep_open.insert(root_entity);
        }

        if let Ok(focus_parent) = focus_parents.get(root_entity) {
            current_entity = Some(focus_parent.0);
        }
    }

    for entity in should_close.iter() {
        if keep_open.contains(&entity) {
            continue;
        }
        commands.entity(entity).insert(FocusShouldClose);
    }
}
