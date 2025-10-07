use bevy_ecs::{
    component::Component,
    entity::Entity,
    hierarchy::Children,
    query::With,
    schedule::IntoScheduleConfigs,
    system::{Query, Single},
};
use bevy_math::Vec2;
use bevy_transform::TransformSystems;
use bevy_ui::{
    ComputedNode, ComputedUiRenderTargetInfo, LayoutConfig, Node, UiGlobalTransform, UiSystems,
    Val, px,
};
use bevy_window::{PrimaryWindow, Window};

/// Implements floating UI placement against other entities
pub struct AnchoredUiPlugin;

impl bevy_app::Plugin for AnchoredUiPlugin {
    fn build(&self, app: &mut bevy_app::App) {
        app.add_systems(
            bevy_app::PostUpdate,
            position_anchor
                .after(UiSystems::Layout)
                .before(TransformSystems::Propagate),
        );
    }
}

/// Specifies against what entity must be positioned
#[derive(Component)]
#[require(PlacementCache, AnchorOption)]
pub enum AnchorTarget {
    /// Place relative to entity
    Entity(Entity),
    /// Place relative to cursor
    Cursor,
    /// Place relative to physical position
    PhysicalPosition(Vec2),
}

/// Allows to specify how exactly entity must be aligned against [`AnchorTarget`]
#[derive(Component, Clone, Copy, PartialEq)]
pub struct AnchorOption {
    /// Anchor location for element to place
    pub anchor: Direction<Anchor>,
    /// Anchor location for element that this element is placed against
    pub target_anchor: Direction<Anchor>,
    /// Additional padding to location where element will be placed
    /// Padding is ignored for Middle anchor locations
    pub padding: Direction<Val>,
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

/// Specifies where anchor is located
///
/// | Anchor |  x     | y      |
/// | ------ | ------ | ------ |
/// | Start  | left   | top    |
/// | Middle | middle | middle |
/// | End    | right  | bottom |
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub enum Anchor {
    /// left or top
    Start,
    /// middle
    Middle,
    /// right or bottom
    End,
}

impl Anchor {
    /// Returns anchor relative position. See [`Anchor`]
    pub fn sign(&self) -> i32 {
        match self {
            Anchor::Start => -1,
            Anchor::Middle => 0,
            Anchor::End => 1,
        }
    }
}

/// Wrapper element to store information for two dimensions
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct Direction<T> {
    /// vertical, x - axis
    pub x: T,
    /// horizontal, y - axis
    pub y: T,
}

impl<T> Direction<T> {
    /// Map stored value from one type to another
    pub fn map<O>(&self, f: impl Fn(&T) -> O) -> Direction<O> {
        Direction {
            x: f(&self.x),
            y: f(&self.y),
        }
    }
}

impl Direction<Anchor> {
    /// Retrieve sign_vec from stored value
    pub fn sign_vec(&self) -> Vec2 {
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

/// Useful helper function to correctly update global transformations
/// and top, left element position in [`bevy_ui::Node`]
/// for the whole subtree rooted at `current` entity.
pub fn apply_position(
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

/// Useful helper function to correctly update global transformations
/// for the whole subtree rooted at `current` entity.
pub fn update_global_transforms(
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
