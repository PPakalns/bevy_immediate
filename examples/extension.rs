use bevy_ecs::component::Component;
use bevy_immediate::{ImmCap, ImmEntity, ImplCap};

pub struct ExtensionExamplePlugin;

impl bevy_app::Plugin for ExtensionExamplePlugin {
    fn build(&self, _app: &mut bevy_app::App) {
        // Add your resources, systems that are needed for extension to work
    }
}

/// Create your own capability
pub struct CapUiToggle;

impl ImmCap for CapUiToggle {
    fn build<Cap: ImmCap>(
        app: &mut bevy_app::App,
        cap_req: &mut bevy_immediate::ImmCapAccessRequests<Cap>,
    ) {
        // Require necessary plugins
        if !app.is_plugin_added::<ExtensionExamplePlugin>() {
            app.add_plugins(ExtensionExamplePlugin);
        }

        // cap_req.request_resource_write::<Resource1>(app.world_mut());
        // cap_req.request_resource_read::<Resource2>(app.world_mut());

        cap_req.request_component_write::<ToggleState>(app.world_mut());
    }
}

#[derive(Component)]
struct ToggleState {
    state: bool,
}

/// Implements access to collapse state
pub trait ImmCapUiCollapse {
    fn get_toggle(&mut self) -> bool;
    fn set_toggle(&mut self, value: bool);
    fn flip_toggle(&mut self);
    fn on_insert_toggle(self, toggle: bool) -> Self;
    fn with_toggle(&mut self, f: impl FnOnce(&mut bool));
}

impl<Cap> ImmCapUiCollapse for ImmEntity<'_, '_, '_, Cap>
where
    Cap: ImplCap<CapUiToggle>,
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
        if let Ok(Some(mut comp)) = self.cap_get_component_mut::<ToggleState>() {
            // Lookup from component
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

            // Insert state
            self.entity_commands().insert(ToggleState { state });
            self.cap_entity_tmp_store_mut()
                .insert(ToggleState { state });
        }
    }
}
