use bevy::ecs::component::Component;
use bevy_immediate::{CapSet, ImmCapability, ImmEntity, ImplCap};

// This extension example tries to showcase the WORST case.
// Where code wants to read from and write to component.
//
// For very simple capability implementations see
// [bevy_immediate::ui] capability implementations

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

#[derive(Component)]
struct ToggleState {
    state: bool,
}

/// Implements access to collapse state
#[allow(unused)]
pub trait ImmCapUiCollapse {
    fn get_toggle(&mut self) -> bool;
    fn set_toggle(&mut self, value: bool);
    fn flip_toggle(&mut self);
    fn on_insert_toggle(self, toggle: bool) -> Self;
    fn with_toggle(&mut self, f: impl FnOnce(&mut bool));
}

impl<Caps> ImmCapUiCollapse for ImmEntity<'_, '_, '_, Caps>
where
    // Trait functions only implemented for capability sets with
    // CapUiToggle capability
    Caps: ImplCap<CapUiToggle>,
{
    fn get_toggle(&mut self) -> bool {
        let mut ret = false;
        self.with_toggle(|state| {
            ret = *state;
        });
        ret
    }

    fn set_toggle(&mut self, value: bool) {
        self.with_toggle(|state| {
            *state = value;
        });
    }

    fn flip_toggle(&mut self) {
        self.with_toggle(|state| {
            *state = !*state;
        });
    }

    fn on_insert_toggle(mut self, state: bool) -> Self {
        if self.will_be_spawned() {
            self.with_toggle(|stored_state| {
                *stored_state = state;
            });
        }
        self
    }

    fn with_toggle(&mut self, f: impl FnOnce(&mut bool)) {
        // For simpler extension see [`bevy_immediate::ui::selected`]
        //
        // This showcases the worst case where stored state is important.

        if let Ok(Some(mut comp)) = self.cap_get_component_mut::<ToggleState>() {
            // Lookup directly from component
            f(&mut comp.state);
        } else if let Some(tmp_store) = self.cap_entity_tmp_store_mut().get_mut::<ToggleState>() {
            // Entity currently being built, use temporary value
            let old_state = tmp_store.state;
            f(&mut tmp_store.state);
            let new_state = tmp_store.state;

            // Overwrite inserted component with the new value
            if new_state != old_state {
                self.entity_commands()
                    .insert(ToggleState { state: new_state });
            }
        } else {
            // Toggle state not yet set for entity, assume false as default
            let mut state = false;
            f(&mut state);

            // Insert state into component and in entity tmp store
            //
            // Tmp store is required because components can not be looked up from entity_commands
            // therefore for first frame we need additionally to store in tmp store the toggle state
            self.entity_commands().insert(ToggleState { state });
            self.cap_entity_tmp_store_mut()
                .insert(ToggleState { state });
        }
    }
}
