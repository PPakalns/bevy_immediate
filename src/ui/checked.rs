use bevy_ecs::system::EntityCommands;
use bevy_ui::{Checkable, Checked};

use crate::{
    CapSet, ImmCapability, ImmEntity, ImplCap,
    ui::track_value_change_plugin::{NewValueChange, TrackValueChangePlugin},
};

/// Implements capability to synchronise checked value on component
pub struct CapabilityUiChecked;

impl ImmCapability for CapabilityUiChecked {
    fn build<Cap: CapSet>(app: &mut bevy_app::App, cap_req: &mut crate::ImmCapAccessRequests<Cap>) {
        let _ = cap_req;
        let _ = app;
        cap_req.request_component_write::<NewValueChange<bool>>(app.world_mut());

        if !app.is_plugin_added::<TrackValueChangePlugin<bool>>() {
            app.add_plugins(TrackValueChangePlugin::<bool>::default());
        }

        #[cfg(feature = "bevy_ui_widgets")]
        {
            use crate::ui::radio_button_fix_plugin::RadioButtonFixPlugin;
            if !app.is_plugin_added::<RadioButtonFixPlugin>() {
                app.add_plugins(RadioButtonFixPlugin);
            }
        }
    }
}

/// Implements capability to synchronise checked value on component
pub trait ImmUiChecked {
    /// Control if [`Checked`] component is attached to entity by get set approach
    fn checked_get_set(self, f: impl FnMut(Option<bool>) -> bool) -> Self;

    /// Synchronise checked value
    fn checked(self, value: &mut bool) -> Self;

    /// Synchronise checked value
    ///
    /// Useful for radio buttons.
    ///
    /// Component is checked if `current` equals `this`.
    /// If entity becomes checked, `current` value is overriden by `this`.
    /// If entity becomes unchecked, nothing happens.
    fn checked_if_eq<T: PartialEq>(self, this: T, current: &mut T) -> Self;
}

impl<Cap> ImmUiChecked for ImmEntity<'_, '_, '_, Cap>
where
    Cap: ImplCap<CapabilityUiChecked>,
{
    /// Get set access for checked value
    fn checked_get_set(mut self, mut f: impl FnMut(Option<bool>) -> bool) -> Self {
        let current = f(None);
        let mut new = current;
        self = self.checked(&mut new);
        if new != current {
            f(Some(new));
        }
        self
    }

    fn checked(mut self, value: &mut bool) -> Self {
        fn update_checked(commands: &mut EntityCommands, value: bool) {
            if value {
                commands.insert(Checked);
            } else {
                commands.remove::<Checked>();
            }
        }

        'initialized: {
            let Ok(mut entity) = self.cap_get_entity_mut() else {
                break 'initialized;
            };
            let Some(mut new_value) = entity.get_mut::<NewValueChange<bool>>() else {
                break 'initialized;
            };

            let new_value = NewValueChange::take(&mut new_value);

            let last_value = entity.contains::<Checked>();

            if let Some(new_value) = new_value
                    // Avoid update loop
                    && new_value != last_value
            {
                *value = new_value;
                update_checked(&mut self.entity_commands(), new_value);
                return self;
            }

            if *value != last_value {
                // Checked value changed
                update_checked(&mut self.entity_commands(), *value);
            }

            return self;
        }

        let mut commands = self.entity_commands();
        commands.insert((NewValueChange::<bool>::default(), Checkable));
        update_checked(&mut commands, *value);

        self
    }

    fn checked_if_eq<T: PartialEq>(mut self, this: T, current: &mut T) -> Self {
        let before = current == &this;
        let mut after = before;

        self = self.checked(&mut after);

        if after != before && after {
            *current = this;
        }
        self
    }
}
