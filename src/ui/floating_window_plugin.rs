use bevy_ecs::{
    bundle::Bundle,
    children,
    component::Component,
    hierarchy::{ChildOf, Children},
    observer::On,
    query::With,
    resource::Resource,
    spawn::SpawnRelated,
    system::{Commands, Query, Res, ResMut},
};
use bevy_feathers::cursor::{DefaultCursor, EntityCursor};
use bevy_math::{I8Vec2, Vec2};
use bevy_picking::{
    Pickable,
    events::{Drag, DragEnd, DragStart, Pointer},
    hover::Hovered,
};
use bevy_ui::{
    ComputedNode, ComputedUiRenderTargetInfo, LayoutConfig, Node, Pressed, RepeatedGridTrack,
    UiGlobalTransform, UiScale, Val, px,
};
use bevy_window::SystemCursorIcon;

use crate::ui::floating_entity_plugin::{FloatingEntityPlugin, UiBringForward, UiZOrderLayer};

pub struct FloatingWindowPlugin;

impl bevy_app::Plugin for FloatingWindowPlugin {
    fn build(&self, app: &mut bevy_app::App) {
        if !app.is_plugin_added::<FloatingEntityPlugin>() {
            app.add_plugins(FloatingEntityPlugin);
        }

        app.add_observer(window_on_drag_start)
            .add_observer(window_on_drag);

        app.insert_resource(WindowDragTmpCursor::default());
        app.add_observer(window_resize_drag_start)
            .add_observer(window_resize_drag)
            .add_observer(window_resize_drag_end);
    }
}

#[derive(Component)]
#[require(FloatingWindowDrag, UiZOrderLayer::Window, UiBringForward)]
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

    super::anchored_entity_plugin::apply_position(
        drag.entity,
        target_position,
        &mut node,
        comp_node,
        layout_config,
        &children,
        &mut global_transform,
    );
}

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

#[derive(Resource, Default)]
pub struct WindowDragTmpCursor {
    cursor: Option<EntityCursor>,
}

fn window_resize_drag_start(
    mut drag_start: On<Pointer<DragStart>>,
    mut q_target: Query<(), With<WindowResizeDragDirection>>,
    mut q_windows: Query<(&mut Node, &ComputedNode), With<FloatingWindow>>,
    q_parents: Query<&ChildOf>,
    mut commands: Commands,

    system_cursor: Query<&EntityCursor>,
    mut default_cursor: ResMut<DefaultCursor>,
    mut tmp_cursor: ResMut<WindowDragTmpCursor>,
) {
    let Ok(()) = q_target.get_mut(drag_start.entity) else {
        return;
    };
    // Avoid window dragging
    drag_start.propagate(false);

    commands.entity(drag_start.entity).insert(Pressed);

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

    let Ok((mut window_node, window_comp_node)) = q_windows.get_mut(window_entity) else {
        return;
    };

    // Reset min_width, min_height
    let width = window_comp_node.size.x;
    let height = window_comp_node.size.y;

    window_node.min_width = px(width * window_comp_node.inverse_scale_factor);
    window_node.min_height = px(height * window_comp_node.inverse_scale_factor);
}

fn window_resize_drag(
    mut drag: On<Pointer<Drag>>,
    mut position_query: Query<&WindowResizeDragDirection>,
    q_parents: Query<&ChildOf>,
    mut q_windows: Query<
        (&mut Node, &ComputedNode, &ComputedUiRenderTargetInfo),
        With<FloatingWindow>,
    >,
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

    let Ok((mut window_node, window_comp_node, window_comp_target_info)) =
        q_windows.get_mut(window_entity)
    else {
        return;
    };

    // Delta in physical coordinates
    let delta = drag.delta / (window_comp_node.inverse_scale_factor * ui_scale.0);

    // Retrieve only necessary dimensions
    let drag_direction = drag_direction.0;
    let delta = drag_direction.as_vec2().abs() * delta;

    let mut size_change = Vec2::ZERO;
    let mut left_top_change = Vec2::ZERO;

    if drag_direction.x < 0 {
        left_top_change.x += delta.x;
        size_change.x += -delta.x;
    } else if drag_direction.x > 0 {
        size_change.x += delta.x;
    }

    if drag_direction.y < 0 {
        left_top_change.y += delta.y;
        size_change.y += -delta.y;
    } else if drag_direction.y > 0 {
        size_change.y += delta.y;
    }

    if left_top_change != Vec2::ZERO {
        let mut left = resolve_x(window_node.left, window_comp_target_info).unwrap_or(0.);
        let mut top = resolve_y(window_node.top, window_comp_target_info).unwrap_or(0.);

        left += left_top_change.x;
        top += left_top_change.y;

        window_node.left = px(left * window_comp_node.inverse_scale_factor);
        window_node.top = px(top * window_comp_node.inverse_scale_factor);
    }

    if size_change != Vec2::ZERO {
        let mut width = resolve_x(window_node.min_width, window_comp_target_info)
            .unwrap_or(window_comp_node.size.x);
        let mut height = resolve_y(window_node.min_height, window_comp_target_info)
            .unwrap_or(window_comp_node.size.y);

        // TODO: Set real min width in floating window configuration
        width = (width + size_change.x).max(50.);
        height = (height + size_change.y).max(50.);

        window_node.min_width = px(width * window_comp_node.inverse_scale_factor);
        window_node.min_height = px(height * window_comp_node.inverse_scale_factor);
        window_node.width = px(width * window_comp_node.inverse_scale_factor);
        window_node.height = px(height * window_comp_node.inverse_scale_factor);
    }
}

fn window_resize_drag_end(
    drag_end: On<Pointer<DragEnd>>,
    mut q_target: Query<(), With<WindowResizeDragDirection>>,
    mut commands: Commands,

    mut default_cursor: ResMut<DefaultCursor>,
    mut tmp_cursor: ResMut<WindowDragTmpCursor>,
) {
    let Ok(()) = q_target.get_mut(drag_end.entity) else {
        return;
    };
    commands.entity(drag_end.entity).remove::<Pressed>();

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
                EntityCursor::System(SystemCursorIcon::NwResize),
                additional
            ),
            (
                Node::DEFAULT,
                WindowResizeDragDirection(I8Vec2 { x: 0, y: -1 }),
                EntityCursor::System(SystemCursorIcon::NResize),
                additional
            ),
            (
                Node::DEFAULT,
                WindowResizeDragDirection(I8Vec2 { x: 1, y: -1 }),
                EntityCursor::System(SystemCursorIcon::NeResize),
                additional
            ),
            (
                Node::DEFAULT,
                WindowResizeDragDirection(I8Vec2 { x: -1, y: 0 }),
                EntityCursor::System(SystemCursorIcon::WResize),
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
                EntityCursor::System(SystemCursorIcon::EResize),
                additional
            ),
            (
                Node::DEFAULT,
                WindowResizeDragDirection(I8Vec2 { x: -1, y: 1 }),
                EntityCursor::System(SystemCursorIcon::SwResize),
                additional
            ),
            (
                Node::DEFAULT,
                WindowResizeDragDirection(I8Vec2 { x: 0, y: 1 }),
                EntityCursor::System(SystemCursorIcon::SResize),
                additional
            ),
            (
                Node::DEFAULT,
                WindowResizeDragDirection(I8Vec2 { x: 1, y: 1 }),
                EntityCursor::System(SystemCursorIcon::SeResize),
                additional
            ),
        ]
    )]
}
