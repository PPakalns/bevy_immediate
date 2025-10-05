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
    GlobalZIndex, Node, RepeatedGridTrack, UiGlobalTransform, UiRect, UiScale, UiSystems, Val, px,
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
            position_tooltip
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
            update_window_order.before(UiSystems::Prepare),
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
                                    FloatingWindow { focus: true },
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

                let trigger_dropdown = button.activated();
                button = button.with_dropdown_container(trigger_dropdown, |container| {
                    container
                        .on_spawn_insert(|| AnchorOption {
                            anchor: Direction { x, y },
                            target_anchor: Direction { x: tx, y: ty },
                            ..default()
                        })
                        .add(|ui| {
                            dropdown_content(ui);
                        });
                });
                let is_shown = button.dropdown_is_shown();
                button.primary_button(is_shown);
            }
        });
    }
}

fn dropdown_content(ui: &mut Imm<'_, '_, CapsUiFeathers>) {
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
                BackgroundColor(bevy_color::palettes::css::BLACK.into()),
                BorderColor::all(bevy_color::palettes::css::DARK_GRAY),
            )
        })
        .add(|ui| {
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

                let trigger_dropdown = button.activated();
                button = button.with_dropdown_container(trigger_dropdown, |container| {
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
                                        BackgroundColor(bevy_color::palettes::css::BLACK.into()),
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
                });
                let is_shown = button.dropdown_is_shown();
                button.primary_button(is_shown);
            }
        });
}

pub trait ImmUiDropdown<'w, 's, Caps: CapSet> {
    fn dropdown_is_shown(&self) -> bool;
    fn with_dropdown(self, triggered: bool, f: impl FnOnce(&mut Imm<'w, 's, Caps>)) -> Self;
    fn with_dropdown_container(
        self,
        triggered: bool,
        f: impl FnOnce(ImmEntity<'_, 'w, 's, Caps>),
    ) -> Self;
}

struct DropdownHash;

impl<'w, 's, Caps> ImmUiDropdown<'w, 's, Caps> for ImmEntity<'_, 'w, 's, Caps>
where
    Caps: ImplCapsUi,
{
    fn dropdown_is_shown(&self) -> bool {
        self.hash_get_typ::<DropdownHash>().is_some()
    }

    fn with_dropdown(self, triggered: bool, f: impl FnOnce(&mut Imm<'w, 's, Caps>)) -> Self {
        self.with_dropdown_container(triggered, |entity| {
            entity.add(|ui| {
                f(ui);
            });
        })
    }

    fn with_dropdown_container(
        mut self,
        triggered: bool,
        f: impl FnOnce(ImmEntity<'_, 'w, 's, Caps>),
    ) -> Self {
        let mut show = self.dropdown_is_shown();

        if triggered {
            show = !show;
            if show {
                self.hash_store_typ::<DropdownHash>(imm_id(()));
            } else {
                self.hash_remove_typ::<DropdownHash>();
            }
        }

        if show {
            let entity = self.entity();
            self = self.add(|ui| {
                ui.unrooted("with_dropdown", |ui| {
                    let entity = ui.ch().on_spawn_insert(|| {
                        (
                            Node {
                                position_type: bevy_ui::PositionType::Absolute,
                                ..default()
                            },
                            AnchorTarget::Entity(entity),
                            FocusParent(entity),
                            FocusDetectShouldClose,
                        )
                    });

                    if entity.cap_entity_contains::<FocusShouldClose>() {
                        show = false;
                    }

                    f(entity);
                });
            });

            if !show {
                self.hash_remove_typ::<DropdownHash>();
            }
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
            let offset = ((final_position - tooltip_computed.size * 0.5)
                / tooltip_target_info.scale_factor())
            .round();
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

#[derive(Component)]
#[require(FloatingWindowDrag, GlobalZIndex)]
pub struct FloatingWindow {
    pub focus: bool,
}

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
        (&mut FloatingWindowDrag, &mut Node, &ComputedNode),
        With<FloatingWindowDrag>,
    >,
    ui_scale: Res<UiScale>,
    mut global_transform: Query<&mut UiGlobalTransform>,
    children: Query<&Children>,
) {
    let Ok((mut state, mut node, comp_node)) = scroll_position_query.get_mut(drag.entity) else {
        return;
    };

    drag.propagate(false);

    let distance = drag.distance / (comp_node.inverse_scale_factor * ui_scale.0);
    let target_position = state.initial_pos + distance;

    if state.last_offset == Some(target_position) {
        return;
    }
    state.last_offset = Some(target_position);

    {
        let offset =
            ((target_position - comp_node.size * 0.5) * comp_node.inverse_scale_factor).round();
        node.left = px(offset.x);
        node.top = px(offset.y);
    }

    let Ok(current) = global_transform.get(drag.entity) else {
        return;
    };

    // Logic to avoid 1 frame delay
    // Global transform update is done immediatelly
    let delta = target_position - current.translation;

    update_global_transforms(drag.entity, delta, &children, &mut global_transform);
}

fn window_on_focus(
    pointer: On<Pointer<Press>>,
    mut windows: Query<&mut FloatingWindow>,
    child_of: Query<&ChildOf>,
) {
    if pointer.original_event_target() != pointer.entity {
        return;
    }

    let root_entity = child_of.root_ancestor(pointer.entity);

    let Ok(mut window) = windows.get_mut(root_entity) else {
        return;
    };

    window.focus = true;
}

fn update_window_order(mut windows: Query<(Entity, &mut FloatingWindow, &mut GlobalZIndex)>) {
    let mut process = false;
    for window in windows.iter_mut() {
        if window.1.focus {
            process = true;
        }
    }
    if !process {
        return;
    }

    let mut back = vec![];
    let mut front = vec![];
    for mut window in windows.iter_mut() {
        if window.1.focus {
            front.push(window.0);
            window.1.focus = false;
        } else {
            back.push((window.2.0, window.0));
        }
    }
    back.sort();
    let mut z_index = WINDOW_Z_INDEX_BASE;
    for entity in back.into_iter().map(|item| item.1).chain(front.into_iter()) {
        let (_, _, mut global_z) = windows.get_mut(entity).unwrap();

        if global_z.0 != z_index {
            global_z.0 = z_index;
        }

        z_index += 1;
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
