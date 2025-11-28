use bevy_ecs::{
    component::Component, entity::Entity, hierarchy::ChildOf, observer::On,
    schedule::IntoScheduleConfigs, system::Query,
};
use bevy_picking::events::{Pointer, Press};
use bevy_ui::{GlobalZIndex, UiSystems};

/// Logic to handle UI layer Z ordering
pub struct FloatingUiOrderingPlugin;

impl bevy_app::Plugin for FloatingUiOrderingPlugin {
    fn build(&self, app: &mut bevy_app::App) {
        app.add_systems(
            bevy_app::PostUpdate,
            update_ui_layer_order.before(UiSystems::Prepare),
        );

        app.add_observer(window_on_focus);
    }
}

/// Component that tracks if ui layer should be brought to the top
#[derive(Component)]
pub struct UiBringForward {
    forward: bool,
}

impl UiBringForward {
    /// See [`UiBringForward`]
    pub fn new(forward: bool) -> Self {
        Self { forward }
    }
}

impl Default for UiBringForward {
    fn default() -> Self {
        Self { forward: true }
    }
}

fn window_on_focus(
    pointer: On<Pointer<Press>>,
    mut forward: Query<&mut UiBringForward>,
    child_of: Query<&ChildOf>,
) {
    if pointer.original_event_target() != pointer.entity {
        return;
    }

    let root_entity = child_of.root_ancestor(pointer.entity);

    let Ok(mut forward) = forward.get_mut(root_entity) else {
        return;
    };

    forward.forward = true;
}

/// To which UI layer ui entity tree is assigned to
#[derive(Component, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
#[require(UiBringForward { forward: true }, GlobalZIndex)]
pub enum UiZOrderLayer {
    /// Layer for floating windows
    Window,
    /// Layer for dropdowns
    Dropdown,
    /// Layer for popups
    Popup,
    /// Layer for tooltips
    Tooltip,
    /// Layer for notifications
    Notifications,
}

impl UiZOrderLayer {
    /// Return GlobalZIndex number from which given layer starts
    pub fn base(&self) -> i32 {
        match self {
            UiZOrderLayer::Window => 1000,
            UiZOrderLayer::Dropdown => 2000,
            UiZOrderLayer::Popup => 3000,
            UiZOrderLayer::Tooltip => 4000,
            UiZOrderLayer::Notifications => 5000,
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
