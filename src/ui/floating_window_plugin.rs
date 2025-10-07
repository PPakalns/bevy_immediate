use ahash::HashMap;
use bevy_ecs::{
    bundle::Bundle,
    children,
    component::Component,
    hierarchy::{ChildOf, Children},
    observer::On,
    query::{Added, Changed, Or, With},
    resource::Resource,
    schedule::IntoScheduleConfigs,
    spawn::SpawnRelated,
    system::{Commands, Query, Res, ResMut},
};
use bevy_math::{I8Vec2, Vec2, bounding::Aabb2d};
use bevy_picking::{
    Pickable,
    events::{Drag, DragEnd, DragStart, Pointer},
    hover::Hovered,
};
use bevy_ui::{
    ComputedNode, ComputedUiRenderTargetInfo, LayoutConfig, Node, Pressed, RepeatedGridTrack,
    UiGlobalTransform, UiScale, UiSystems, Val, px,
};
use rand::Rng;

use crate::{
    ImmId,
    ui::floating_ui_ordering_plugin::{FloatingUiOrderingPlugin, UiBringForward, UiZOrderLayer},
};

#[cfg(feature = "bevy_feathers")]
use bevy_feathers::cursor;
#[cfg(feature = "bevy_feathers")]
use bevy_window::SystemCursorIcon;

/// Plugin implements floating windows
/// and such functionality as window
/// dragging, resizing, size restrictions
/// and keeping them always inside view.
pub struct FloatingWindowPlugin;

impl bevy_app::Plugin for FloatingWindowPlugin {
    fn build(&self, app: &mut bevy_app::App) {
        if !app.is_plugin_added::<FloatingUiOrderingPlugin>() {
            app.add_plugins(FloatingUiOrderingPlugin);
        }

        app.add_observer(window_on_drag_start)
            .add_observer(window_on_drag)
            .add_observer(window_on_drag_end);

        #[cfg(feature = "bevy_feathers")]
        app.insert_resource(WindowDragTmpCursor::default());

        app.add_observer(window_resize_drag_start)
            .add_observer(window_resize_drag)
            .add_observer(window_resize_drag_end);

        app.add_systems(
            bevy_app::PostUpdate,
            floating_window_node_update_system.before(UiSystems::Prepare),
        );

        app.add_systems(
            bevy_app::PostUpdate,
            floating_window_node_init_system.before(UiSystems::Prepare),
        );

        app.insert_resource(FloatingWindowLocationStore::default());

        app.add_systems(bevy_app::PreUpdate, floating_window_cache);
    }
}

/// Floating window and its sizing restrictions
///
/// These need to be specified separately because
/// [`Node`]` values will be overrided.
#[derive(Component)]
#[require(
    Node,
    FloatingWindowInteractionState,
    UiZOrderLayer::Window,
    UiBringForward
)]
pub struct FloatingWindow {
    /// See [`Node::initial_width`]
    pub initial_width: Val,
    /// See [`Node::initial_height`]
    pub initial_height: Val,
    /// See [`Node::min_width`]
    pub min_width: Val,
    /// See [`Node::min_height`]
    pub min_height: Val,
    /// See [`Node::max_width`]
    pub max_width: Val,
    /// See [`Node::max_height`]
    pub max_height: Val,

    /// Ratio of how much floating window should be inside
    /// window.
    ///
    /// Ration used to move windows inside screen when
    /// they are fully outside the view.
    pub overlap_ratio: Option<f32>,
}

impl Default for FloatingWindow {
    fn default() -> Self {
        Self {
            min_width: Val::Px(50.),
            min_height: Val::Px(50.),
            max_width: Val::Vw(90.),
            max_height: Val::Vh(90.),
            initial_width: Val::Auto,
            initial_height: Val::Auto,
            overlap_ratio: Some(0.075),
        }
    }
}

fn floating_window_node_init_system(
    mut query: Query<
        (
            &FloatingWindow,
            Option<&FloatingWindowStoreLocationId>,
            &mut Node,
        ),
        Added<FloatingWindow>,
    >,
    floating_window_location_store: Res<FloatingWindowLocationStore>,
) {
    let mut rng = rand::rng();
    for (floating, floating_window_location_id, mut node) in query.iter_mut() {
        if let Some(stored_location) = floating_window_location_id
            .and_then(|id| floating_window_location_store.stored.get(&id.0))
        {
            // Restore window location from memory
            node.min_width = px(stored_location.size_px.x);
            node.min_height = px(stored_location.size_px.y);
            node.max_width = px(stored_location.size_px.x);
            node.max_height = px(stored_location.size_px.y);

            node.left = px(stored_location.offset_px.x);
            node.top = px(stored_location.offset_px.y);
        } else {
            node.min_width = floating.initial_width;
            node.min_height = floating.initial_height;
            node.max_width = floating.max_width;
            node.max_height = floating.max_height;

            // TODO: Improve logic to decide window start location
            node.left = px(rng.random_range(10..500i32) as f32);
            node.top = px(rng.random_range(10..500i32) as f32);
        }
    }
}

#[allow(clippy::type_complexity)]
fn floating_window_node_update_system(
    mut query: Query<
        (
            &FloatingWindow,
            &mut Node,
            &FloatingWindowInteractionState,
            &ComputedNode,
            &ComputedUiRenderTargetInfo,
            &UiGlobalTransform,
        ),
        (
            Or<(
                Changed<FloatingWindow>,
                Changed<FloatingWindowInteractionState>,
                Changed<ComputedNode>,
                Changed<ComputedUiRenderTargetInfo>,
                Changed<UiGlobalTransform>,
            )>,
        ),
    >,
) {
    for (
        floating_window,
        mut node,
        interaction_state,
        comp_node,
        comp_target_info,
        global_transform,
    ) in query.iter_mut()
    {
        let Some(overlap_ratio) = floating_window.overlap_ratio else {
            continue;
        };

        if interaction_state.currently_drag || interaction_state.currently_resize {
            continue;
        }

        if (node.width == Val::Auto || node.height == Val::Auto)
            // Detect if layout calculations have been done
            && comp_node.size.min_element() > 10.
        {
            // Fixed size is needed to force overflowing elements to not overflow
            // See `Bevy Scrollareas` bevy_immediate example floating window behaviour.
            node.min_width = px(comp_node.size.x * comp_node.inverse_scale_factor);
            node.min_height = px(comp_node.size.y * comp_node.inverse_scale_factor);
        }

        let window_node = Aabb2d::new(global_transform.translation, comp_node.size * 0.5);
        let camera = Aabb2d {
            max: comp_target_info.physical_size().as_vec2(),
            min: Vec2::ZERO,
        };

        if camera.min.x <= window_node.min.x
            && window_node.max.x <= camera.max.x
            && camera.min.y <= window_node.min.y
            && window_node.max.y <= camera.max.y
        {
            // Fully inside, no need to check overlap
            continue;
        }

        let int_min = window_node.min.max(camera.min);
        let int_max = window_node.max.min(camera.max);

        let overlap = (int_max - int_min).max(Vec2::ZERO);
        let needed_overlap = comp_target_info.physical_size().as_vec2() * overlap_ratio;

        if overlap.x >= needed_overlap.x && overlap.y >= needed_overlap.y {
            continue;
        }

        let mut offset_to_add = (needed_overlap - overlap).max(Vec2::ZERO);
        if window_node.min.x > 0. {
            offset_to_add.x *= -1.;
        }
        if window_node.min.y > 0. {
            offset_to_add.y *= -1.;
        }

        let left = resolve_x(node.left, comp_target_info).unwrap_or(0.) + offset_to_add.x;
        let top = resolve_y(node.top, comp_target_info).unwrap_or(0.) + offset_to_add.y;

        node.left = px(left * comp_node.inverse_scale_factor);
        node.top = px(top * comp_node.inverse_scale_factor);
    }
}

#[derive(Component, Default)]
struct FloatingWindowInteractionState {
    // For dragging
    currently_drag: bool,
    initial_drag_pos: Vec2,
    drag_last_offset: Option<Vec2>,

    // For resize
    currently_resize: bool,
    initial_resize_size: Vec2,
    initial_resize_offset: Vec2,
}

fn window_on_drag_start(
    mut drag_start: On<Pointer<DragStart>>,
    mut scroll_position_query: Query<
        (&UiGlobalTransform, &mut FloatingWindowInteractionState),
        With<FloatingWindowInteractionState>,
    >,
) {
    if let Ok((transform, mut state)) = scroll_position_query.get_mut(drag_start.entity) {
        state.initial_drag_pos = transform.translation;
        state.drag_last_offset = None;
        state.currently_drag = true;

        drag_start.propagate(false);
    }
}

fn window_on_drag(
    mut drag: On<Pointer<Drag>>,
    mut scroll_position_query: Query<
        (
            &mut FloatingWindowInteractionState,
            &mut Node,
            &ComputedNode,
            Option<&LayoutConfig>,
        ),
        With<FloatingWindowInteractionState>,
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
    let target_position = state.initial_drag_pos + distance;

    if state.drag_last_offset == Some(target_position) {
        return;
    }
    state.drag_last_offset = Some(target_position);

    super::anchored_ui_plugin::apply_position(
        drag.entity,
        target_position,
        &mut node,
        comp_node,
        layout_config,
        &children,
        &mut global_transform,
    );
}

fn window_on_drag_end(
    mut drag: On<Pointer<DragEnd>>,
    mut scroll_position_query: Query<
        &mut FloatingWindowInteractionState,
        With<FloatingWindowInteractionState>,
    >,
) {
    if let Ok(mut state) = scroll_position_query.get_mut(drag.entity) {
        state.currently_drag = false;

        drag.propagate(false);
    }
}

/// If entity with this component is dragged
/// It will be resized in the given direction
#[derive(Component)]
#[require(Hovered)]
pub struct WindowResizeDragDirection(I8Vec2);

fn resolve_x(
    y: Val,
    target_info: &ComputedUiRenderTargetInfo,
) -> Result<f32, bevy_ui::ValArithmeticError> {
    y.resolve(
        target_info.scale_factor(),
        target_info.physical_size().x as f32,
        target_info.physical_size().as_vec2(),
    )
}
fn resolve_y(
    y: Val,
    target_info: &ComputedUiRenderTargetInfo,
) -> Result<f32, bevy_ui::ValArithmeticError> {
    y.resolve(
        target_info.scale_factor(),
        target_info.physical_size().y as f32,
        target_info.physical_size().as_vec2(),
    )
}

/// Stores cursor that was temporary replaced with cursor used for resizing window (dragging)
#[derive(Resource, Default)]
#[cfg(feature = "bevy_feathers")]
pub struct WindowDragTmpCursor {
    cursor: Option<cursor::EntityCursor>,
}

#[allow(clippy::too_many_arguments)]
fn window_resize_drag_start(
    mut drag_start: On<Pointer<DragStart>>,
    mut q_target: Query<(), With<WindowResizeDragDirection>>,
    mut q_windows: Query<
        (
            &Node,
            &mut FloatingWindowInteractionState,
            &ComputedNode,
            &ComputedUiRenderTargetInfo,
        ),
        With<FloatingWindow>,
    >,
    q_parents: Query<&ChildOf>,
    mut commands: Commands,

    #[cfg(feature = "bevy_feathers")] system_cursor: Query<&cursor::EntityCursor>,
    #[cfg(feature = "bevy_feathers")] mut default_cursor: ResMut<cursor::DefaultCursor>,
    #[cfg(feature = "bevy_feathers")] mut tmp_cursor: ResMut<WindowDragTmpCursor>,
) {
    let Ok(()) = q_target.get_mut(drag_start.entity) else {
        return;
    };
    // Avoid window dragging
    drag_start.propagate(false);

    commands.entity(drag_start.entity).insert(Pressed);

    #[cfg(feature = "bevy_feathers")]
    if let Ok(cursor) = system_cursor.get(drag_start.entity) {
        std::mem::swap(
            &mut default_cursor.0,
            tmp_cursor.cursor.insert(cursor.clone()),
        );
    }

    let Some(window_entity) = q_parents
        .iter_ancestors(drag_start.entity)
        .find(|ancestor| q_windows.contains(*ancestor))
    else {
        return;
    };

    let Ok((node, mut window_interaction_state, window_comp_node, window_comp_target_info)) =
        q_windows.get_mut(window_entity)
    else {
        return;
    };

    window_interaction_state.currently_resize = true;
    window_interaction_state.initial_resize_size = window_comp_node.size;
    window_interaction_state.initial_resize_offset = Vec2::new(
        resolve_x(node.left, window_comp_target_info).unwrap_or(0.),
        resolve_y(node.top, window_comp_target_info).unwrap_or(0.),
    );
}

fn window_resize_drag(
    mut drag: On<Pointer<Drag>>,
    mut position_query: Query<&WindowResizeDragDirection>,
    q_parents: Query<&ChildOf>,
    mut q_windows: Query<(
        &mut Node,
        &FloatingWindow,
        &FloatingWindowInteractionState,
        &ComputedNode,
        &ComputedUiRenderTargetInfo,
    )>,
    ui_scale: Res<UiScale>,
) {
    let Ok(drag_direction) = position_query.get_mut(drag.entity) else {
        return;
    };

    drag.propagate(false);

    let Some(window_entity) = q_parents
        .iter_ancestors(drag.entity)
        .find(|ancestor| q_windows.contains(*ancestor))
    else {
        return;
    };

    let Ok((
        mut window_node,
        floating_window,
        floating_window_inter_state,
        window_comp_node,
        window_comp_target_info,
    )) = q_windows.get_mut(window_entity)
    else {
        return;
    };

    // Distance in physical coordinates
    let distance = drag.distance / (window_comp_node.inverse_scale_factor * ui_scale.0);

    // Retrieve only necessary dimensions
    let drag_direction = drag_direction.0;
    let delta = drag_direction.as_vec2().abs() * distance;

    let mut size_change = Vec2::ZERO;
    let mut left_top_change = Vec2::ZERO;

    if drag_direction.x < 0 {
        left_top_change.x = 1.;
        size_change.x += -delta.x;
    } else if drag_direction.x > 0 {
        size_change.x += delta.x;
    }

    if drag_direction.y < 0 {
        left_top_change.y = 1.;
        size_change.y += -delta.y;
    } else if drag_direction.y > 0 {
        size_change.y += delta.y;
    }

    if size_change != Vec2::ZERO {
        let width = floating_window_inter_state.initial_resize_size.x;
        let height = floating_window_inter_state.initial_resize_size.y;

        // TODO: Set real min width in floating window configuration
        let mut final_width = width + size_change.x;
        let mut final_height = height + size_change.y;

        final_width = final_width
            .min(
                resolve_x(floating_window.max_width, window_comp_target_info)
                    .unwrap_or(window_comp_target_info.physical_size().x as f32),
            )
            .max(resolve_x(floating_window.min_width, window_comp_target_info).unwrap_or(50.));

        final_height = final_height
            .min(
                resolve_x(floating_window.max_height, window_comp_target_info)
                    .unwrap_or(window_comp_target_info.physical_size().y as f32),
            )
            .max(resolve_x(floating_window.min_height, window_comp_target_info).unwrap_or(50.));

        window_node.min_width = px(final_width * window_comp_node.inverse_scale_factor);
        window_node.min_height = px(final_height * window_comp_node.inverse_scale_factor);
        window_node.width = px(final_width * window_comp_node.inverse_scale_factor);
        window_node.height = px(final_height * window_comp_node.inverse_scale_factor);

        if left_top_change != Vec2::ZERO {
            let mut left = floating_window_inter_state.initial_resize_offset.x;
            let mut top = floating_window_inter_state.initial_resize_offset.y;

            left += (width - final_width) * left_top_change.x;
            top += (height - final_height) * left_top_change.y;

            window_node.left = px(left * window_comp_node.inverse_scale_factor);
            window_node.top = px(top * window_comp_node.inverse_scale_factor);
        }
    }
}

fn window_resize_drag_end(
    drag_end: On<Pointer<DragEnd>>,
    mut q_target: Query<(), With<WindowResizeDragDirection>>,
    q_parents: Query<&ChildOf>,
    mut commands: Commands,
    mut q_windows: Query<&mut FloatingWindowInteractionState, With<FloatingWindow>>,

    #[cfg(feature = "bevy_feathers")] mut default_cursor: ResMut<cursor::DefaultCursor>,
    #[cfg(feature = "bevy_feathers")] mut tmp_cursor: ResMut<WindowDragTmpCursor>,
) {
    let Ok(()) = q_target.get_mut(drag_end.entity) else {
        return;
    };
    commands.entity(drag_end.entity).remove::<Pressed>();

    let Some(window_entity) = q_parents
        .iter_ancestors(drag_end.entity)
        .find(|ancestor| q_windows.contains(*ancestor))
    else {
        return;
    };

    let Ok(mut window_interaction_state) = q_windows.get_mut(window_entity) else {
        return;
    };

    window_interaction_state.currently_resize = false;

    #[cfg(feature = "bevy_feathers")]
    if let Some(cursor) = tmp_cursor.cursor.take() {
        default_cursor.0 = cursor;
    }
}

/// Helper bundle that adds draggable borders to UI element
pub fn resizable_borders(border_thickness: f32, additional: impl Bundle + Copy) -> impl Bundle {
    children![(
        Node {
            display: bevy_ui::Display::Grid,
            position_type: bevy_ui::PositionType::Absolute,
            width: Val::Percent(100.),
            height: Val::Percent(100.),
            left: px(-border_thickness),
            top: px(-border_thickness),
            grid_template_columns: vec![
                RepeatedGridTrack::px(1, border_thickness),
                RepeatedGridTrack::percent(1, 100.),
                RepeatedGridTrack::px(1, border_thickness)
            ],
            grid_template_rows: vec![
                RepeatedGridTrack::px(1, border_thickness),
                RepeatedGridTrack::percent(1, 100.),
                RepeatedGridTrack::px(1, border_thickness)
            ],
            row_gap: Val::Px(0.),
            column_gap: Val::Px(0.),
            align_items: bevy_ui::AlignItems::Stretch,
            justify_items: bevy_ui::JustifyItems::Stretch,
            ..Default::default()
        },
        Pickable {
            should_block_lower: false,
            is_hoverable: false
        },
        bevy_ui::ZIndex(1),
        children![
            (
                Node::DEFAULT,
                WindowResizeDragDirection(I8Vec2 { x: -1, y: -1 }),
                #[cfg(feature = "bevy_feathers")]
                cursor::EntityCursor::System(SystemCursorIcon::NwResize),
                additional
            ),
            (
                Node::DEFAULT,
                WindowResizeDragDirection(I8Vec2 { x: 0, y: -1 }),
                #[cfg(feature = "bevy_feathers")]
                cursor::EntityCursor::System(SystemCursorIcon::NResize),
                additional
            ),
            (
                Node::DEFAULT,
                WindowResizeDragDirection(I8Vec2 { x: 1, y: -1 }),
                #[cfg(feature = "bevy_feathers")]
                cursor::EntityCursor::System(SystemCursorIcon::NeResize),
                additional
            ),
            (
                Node::DEFAULT,
                WindowResizeDragDirection(I8Vec2 { x: -1, y: 0 }),
                #[cfg(feature = "bevy_feathers")]
                cursor::EntityCursor::System(SystemCursorIcon::WResize),
                additional
            ),
            (
                Node::DEFAULT,
                Pickable {
                    should_block_lower: false,
                    is_hoverable: false
                },
            ),
            (
                Node::DEFAULT,
                WindowResizeDragDirection(I8Vec2 { x: 1, y: 0 }),
                #[cfg(feature = "bevy_feathers")]
                cursor::EntityCursor::System(SystemCursorIcon::EResize),
                additional
            ),
            (
                Node::DEFAULT,
                WindowResizeDragDirection(I8Vec2 { x: -1, y: 1 }),
                #[cfg(feature = "bevy_feathers")]
                cursor::EntityCursor::System(SystemCursorIcon::SwResize),
                additional
            ),
            (
                Node::DEFAULT,
                WindowResizeDragDirection(I8Vec2 { x: 0, y: 1 }),
                #[cfg(feature = "bevy_feathers")]
                cursor::EntityCursor::System(SystemCursorIcon::SResize),
                additional
            ),
            (
                Node::DEFAULT,
                WindowResizeDragDirection(I8Vec2 { x: 1, y: 1 }),
                #[cfg(feature = "bevy_feathers")]
                cursor::EntityCursor::System(SystemCursorIcon::SeResize),
                additional
            ),
        ]
    )]
}

/// Windows marked with this component will have their size stored in memory.
/// When window is reopened, window size will be restored.
#[derive(Component)]
pub struct FloatingWindowStoreLocationId(pub ImmId);

#[derive(Resource, Default)]
struct FloatingWindowLocationStore {
    stored: HashMap<ImmId, FloatingWindowLocation>,
}

struct FloatingWindowLocation {
    offset_px: Vec2,
    size_px: Vec2,
}

#[allow(clippy::type_complexity)]
fn floating_window_cache(
    mut query: Query<
        (
            &FloatingWindowStoreLocationId,
            &ComputedNode,
            &UiGlobalTransform,
        ),
        (
            Or<(
                Changed<FloatingWindowStoreLocationId>,
                Changed<ComputedNode>,
                Changed<UiGlobalTransform>,
            )>,
        ),
    >,
    mut location_store: ResMut<FloatingWindowLocationStore>,
) {
    for (store_location_id, comp_node, global_trasnform) in query.iter_mut() {
        location_store.stored.insert(
            store_location_id.0,
            FloatingWindowLocation {
                offset_px: (global_trasnform.translation - comp_node.size * 0.5)
                    * comp_node.inverse_scale_factor,
                size_px: comp_node.size * comp_node.inverse_scale_factor,
            },
        );
    }
}
