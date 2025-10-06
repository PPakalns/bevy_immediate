use bevy_ui::InteractionDisabled;

use crate::{CapSet, ImmCapability, ImmEntity, ImplCap};

/// Implements capability to modify UI node look
pub struct CapabilityUiDisabled;

impl ImmCapability for CapabilityUiDisabled {
    fn build<Cap: CapSet>(app: &mut bevy_app::App, cap_req: &mut crate::ImmCapAccessRequests<Cap>) {
        let _ = cap_req;
        let _ = app;
        // No need to set even this
        // cap_req.request_component_read::<InteractionDisabled>(app.world_mut());
    }
}

/// Implements logic to manage entity disabled state
pub trait ImmUiInteractionsDisabled {
    /// Set entity disabled
    fn interactions_disabled(self, disabled: bool) -> Self;
}

impl<Cap> ImmUiInteractionsDisabled for ImmEntity<'_, '_, '_, Cap>
where
    Cap: ImplCap<CapabilityUiDisabled>,
{
    fn interactions_disabled(mut self, disabled: bool) -> Self {
        if let Ok(entity) = self.cap_get_entity()
            && entity.contains::<InteractionDisabled>() == disabled
        {
            // Everything correct
            return self;
        }

        if disabled {
            self.entity_commands().insert(InteractionDisabled);
        } else {
            self.entity_commands().remove::<InteractionDisabled>();
        }

        self
    }
}
