use bevy_ecs::system::EntityCommands;
use bevy_ui::{Selectable, Selected};

use bevy_immediate_core::{CapSet, ImmCapAccessRequests, ImmCapability, ImmEntity, ImplCap};

use crate::track_value_change_plugin::{NewValueChange, TrackValueChangePlugin};

/// Implements capability to synchronise [`Selected`] on entities.
pub struct CapabilityUiSelectable;

impl ImmCapability for CapabilityUiSelectable {
    fn build<Cap: CapSet>(app: &mut bevy_app::App, cap_req: &mut ImmCapAccessRequests<Cap>) {
        cap_req.request_component_write::<NewValueChange<bool>>(app.world_mut());

        if !app.is_plugin_added::<TrackValueChangePlugin<bool>>() {
            app.add_plugins(TrackValueChangePlugin::<bool>::default());
        }
    }
}

/// Synchronises [`bevy_ui::Selected`] on entities.
pub trait ImmUiSelected {
    /// Control if [`Selected`] is attached using a get/set callback.
    fn selected_get_set(self, f: impl FnMut(Option<bool>) -> bool) -> Self;

    /// Two-way sync with [`Selected`].
    fn selected(self, value: &mut bool) -> Self;

    /// Set [`Selected`] without reading widget changes back.
    fn selected_set(self, value: bool) -> Self;

    /// Sync [`Selected`] when `current == this`.
    ///
    /// If the entity becomes selected, `current` is set to `this`.
    /// If it becomes unselected, `current` is unchanged.
    fn selected_if_eq<T: PartialEq>(self, this: T, current: &mut T) -> Self;
}

impl<Cap> ImmUiSelected for ImmEntity<'_, '_, '_, Cap>
where
    Cap: ImplCap<CapabilityUiSelectable>,
{
    fn selected_get_set(mut self, mut f: impl FnMut(Option<bool>) -> bool) -> Self {
        let current = f(None);
        let mut new = current;
        self = self.selected(&mut new);
        if new != current {
            f(Some(new));
        }
        self
    }

    fn selected_set(mut self, value: bool) -> Self {
        'initialized: {
            let Ok(entity) = self.cap_get_entity() else {
                break 'initialized;
            };

            let last_value = entity.contains::<Selected>();
            if !entity.contains::<Selectable>() {
                self.entity_commands().insert(Selectable);
            }
            if last_value != value {
                update_selected(&mut self.entity_commands(), value);
            }

            return self;
        }

        let mut commands = self.entity_commands();
        commands.insert(Selectable);
        if value {
            commands.insert(Selected);
        }

        self
    }

    fn selected(mut self, value: &mut bool) -> Self {
        'initialized: {
            let Ok(mut entity) = self.cap_get_entity_mut() else {
                break 'initialized;
            };
            let Some(mut new_value) = entity.get_mut::<NewValueChange<bool>>() else {
                break 'initialized;
            };

            let new_value = NewValueChange::take(&mut new_value);

            let last_value = entity.contains::<Selected>();

            if let Some(new_value) = new_value
                    // Avoid update loop
                    && new_value != last_value
            {
                *value = new_value;
                update_selected(&mut self.entity_commands(), new_value);
                return self;
            }

            if !entity.contains::<Selectable>() {
                self.entity_commands().insert(Selectable);
            }

            if *value != last_value {
                update_selected(&mut self.entity_commands(), *value);
            }

            return self;
        }

        let mut commands = self.entity_commands();
        commands.insert((NewValueChange::<bool>::default(), Selectable));
        update_selected(&mut commands, *value);

        self
    }

    fn selected_if_eq<T: PartialEq>(mut self, this: T, current: &mut T) -> Self {
        let before = current == &this;
        let mut after = before;

        self = self.selected(&mut after);

        if after != before && after {
            *current = this;
        }
        self
    }
}

fn update_selected(commands: &mut EntityCommands, value: bool) {
    if value {
        commands.insert(Selected);
    } else {
        commands.remove::<Selected>();
    }
}
