use bevy::ecs::{component::Component, entity::Entity, system::Command};
use bevy_immediate::{CapSet, ImmCapability, ImmEntity, ImplCap};

// This extension example tries to showcase the WORST case.
// Where code wants to read from and write to component.
//
// For very simple capability implementations see
// [bevy_immediate::ui] capability implementations
// like [bevy_immediate::ui::checked]

pub struct ExtensionExamplePlugin;

impl bevy::app::Plugin for ExtensionExamplePlugin {
    fn build(&self, _app: &mut bevy::app::App) {
        // No need to implement plugin in this case
    }
}

/// Create your own capability
///
/// For example, we will create a capability which will allow to store toggle state
/// attached to component
pub struct CapUiToggle;

impl ImmCapability for CapUiToggle {
    fn build<Cap: CapSet>(
        app: &mut bevy::app::App,
        cap_req: &mut bevy_immediate::ImmCapAccessRequests<Cap>,
    ) {
        // Require necessary plugins
        if !app.is_plugin_added::<ExtensionExamplePlugin>() {
            app.add_plugins(ExtensionExamplePlugin);
        }

        // If necessary, you can add requirements for resources
        // cap_req.request_resource_write::<Resource1>(app.world_mut());
        // cap_req.request_resource_read::<Resource2>(app.world_mut());

        // Add read, write requests for immediate mode managed entity components
        cap_req.request_component_write::<ToggleState>(app.world_mut());
    }
}

/// Component which will be modified by capability
#[derive(Component)]
struct ToggleState {
    state: bool,
}

/// Implements access to collapse state
#[allow(unused)]
pub trait ImmCapUiCollapse {
    fn get_toggle(&mut self) -> bool;
    fn flip_toggle(&mut self);

    fn set_toggle(self, value: bool) -> Self;
    fn on_insert_toggle(self, toggle: bool) -> Self;

    /// Toggle toggle with given transformation function
    fn with_toggle(&mut self, f: impl FnOnce(Option<bool>) -> bool);
}

impl<Caps> ImmCapUiCollapse for ImmEntity<'_, '_, '_, Caps>
where
    // Trait functions only implemented for capability sets with
    // CapUiToggle capability
    Caps: ImplCap<CapUiToggle>,
{
    fn get_toggle(&mut self) -> bool {
        if let Ok(Some(toggle)) = self.cap_get_component::<ToggleState>() {
            toggle.state
        } else {
            let entity = self.entity();
            self.commands().queue(SetOrInitToggleStateCommand {
                entity,
                state: false,
            });
            false
        }
    }

    fn set_toggle(mut self, value: bool) -> Self {
        self.with_toggle(|state| {
            let _ = state;
            value
        });
        self
    }

    fn flip_toggle(&mut self) {
        self.with_toggle(|state| {
            match state {
                Some(state) => !state,
                // We assume toggle is off by default
                None => true,
            }
        });
    }

    fn on_insert_toggle(mut self, state: bool) -> Self {
        if self.will_be_spawned() {
            self.with_toggle(|stored_state| {
                let _ = stored_state;
                state
            });
        }
        self
    }

    fn with_toggle(&mut self, f: impl FnOnce(Option<bool>) -> bool) {
        if let Ok(Some(mut toggle)) = self.cap_get_component_mut::<ToggleState>() {
            let new_state = f(Some(toggle.state));

            // minimize change triggers
            if new_state != toggle.state {
                toggle.state = new_state;
            }
            return;
        }

        let new_state = f(None);
        let entity = self.entity();
        self.commands().queue(SetOrInitToggleStateCommand {
            entity,
            state: new_state,
        });
    }
}

struct SetOrInitToggleStateCommand {
    entity: Entity,
    state: bool,
}

impl Command for SetOrInitToggleStateCommand {
    type Out = ();

    fn apply(self, world: &mut bevy::ecs::world::World) -> Self::Out {
        if let Ok(mut entity) = world.get_entity_mut(self.entity) {
            if let Some(mut comp) = entity.get_mut::<ToggleState>() {
                // minimize change triggers
                if comp.state != self.state {
                    comp.state = self.state;
                }
                return;
            }
            entity.insert(ToggleState { state: self.state });
        }
    }
}
