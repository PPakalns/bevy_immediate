use bevy_ui::InteractionDisabled;

use bevy_immediate_core::{CapSet, ImmCapAccessRequests, ImmCapability, ImmEntity, ImplCap};

/// Implements capability to interaction disable ui
pub struct CapabilityUiDisabled;

impl ImmCapability for CapabilityUiDisabled {
    fn build<Cap: CapSet>(app: &mut bevy_app::App, cap_req: &mut ImmCapAccessRequests<Cap>) {
        let _ = cap_req;
        let _ = app;
    }
}

/// Implements logic to manage entity disabled state
pub trait ImmUiInteractionsDisabled {
    /// Are interactions enabled for this entity
    /// Method checks [`InteractionDisabled`] existence
    fn is_interactions_enabled(&self) -> bool;

    /// Set entity interactions enabled
    ///
    /// This manages insertion and removal of [`InteractionDisabled`]
    fn interactions_enabled(self, enabled: bool) -> Self;

    /// Set entity interactions disabled
    ///
    /// This manages insertion and removal of [`InteractionDisabled`]
    fn interactions_disabled(self, disabled: bool) -> Self;
}

impl<Cap> ImmUiInteractionsDisabled for ImmEntity<'_, '_, '_, Cap>
where
    Cap: ImplCap<CapabilityUiDisabled>,
{
    fn is_interactions_enabled(&self) -> bool {
        if let Ok(entity) = self.cap_get_entity()
            && entity.contains::<InteractionDisabled>()
        {
            return false;
        }
        return true;
    }

    fn interactions_enabled(self, enabled: bool) -> Self {
        self.interactions_disabled(!enabled)
    }

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
