use bevy_ecs::system::EntityCommands;
use bevy_ui::Checked;

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
    }
}

/// Implements capability to synchronise checked value on component
pub trait ImmUiChecked {
    /// Synchronise checked value
    fn checked(self, value: &mut bool) -> Self;
}

impl<Cap> ImmUiChecked for ImmEntity<'_, '_, '_, Cap>
where
    Cap: ImplCap<CapabilityUiChecked>,
{
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
        commands.insert(NewValueChange::<bool>::default());
        update_checked(&mut commands, *value);

        self
    }
}
