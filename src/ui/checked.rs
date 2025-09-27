use bevy_ecs::component::Component;
use bevy_ui::Checked;

use crate::{CapSet, ImmCapability, ImmEntity, ImplCap};

/// Implements capability to mark entities as selectable.
pub struct CapabilityUiChecked;

impl ImmCapability for CapabilityUiChecked {
    fn build<Cap: CapSet>(app: &mut bevy_app::App, cap_req: &mut crate::ImmCapAccessRequests<Cap>) {
        let _ = cap_req;
        let _ = app;
        cap_req.request_component_write::<StoredCheckedValue>(app.world_mut());
    }
}

#[derive(Component)]
struct StoredCheckedValue {
    checked: bool,
}

/// Implements methods to set entity as checked
pub trait ImmUiChecked {
    /// Update checked state. It requires for component to use [`Checked`] component
    /// for checked state synchronization
    fn checked(self, checked: &mut bool) -> Self;
}

impl<Cap> ImmUiChecked for ImmEntity<'_, '_, '_, Cap>
where
    Cap: ImplCap<CapabilityUiChecked>,
{
    fn checked(mut self, checked: &mut bool) -> Self {
        let is_checked = self
            .cap_get_entity()
            .ok()
            .map(|entity| entity.contains::<Checked>());

        if let (Ok(Some(mut last_store)), Some(is_checked)) = (
            self.cap_get_component_mut::<StoredCheckedValue>(),
            is_checked,
        ) {
            if last_store.checked != is_checked {
                // Component has triggered checked value change

                *checked = is_checked;
                last_store.checked = is_checked;
                return self;
            }

            if *checked != last_store.checked {
                // Checked value changed

                last_store.checked = *checked;
                if *checked {
                    self.entity_commands().insert(Checked);
                } else {
                    self.entity_commands().remove::<Checked>();
                }
            }
            return self;
        }

        let mut commands = self.entity_commands();
        commands.insert(StoredCheckedValue { checked: *checked });
        if *checked {
            commands.insert(Checked);
        } else {
            commands.remove::<Checked>();
        }
        self
    }
}
