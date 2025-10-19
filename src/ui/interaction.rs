use bevy_picking::hover::Hovered;
use bevy_ui::Pressed;

use crate::{CapSet, ImmCapability, ImmEntity, ImplCap};

/// Capability for long lasting interaction tracking from immediate UI
pub struct CapabilityUiInteraction;

impl ImmCapability for CapabilityUiInteraction {
    fn build<Cap: CapSet>(app: &mut bevy_app::App, cap_req: &mut crate::ImmCapAccessRequests<Cap>) {
        cap_req.request_component_read::<Hovered>(app.world_mut());
    }
}

/// Implements support for hovered, clicked effects
pub trait ImmUiInteraction {
    /// Checks state from [`Hovered`]
    fn hovered(&mut self) -> bool;

    /// Checks if component has [`Pressed`]
    fn pressed(&self) -> bool;
}

impl<Cap: CapSet> ImmUiInteraction for ImmEntity<'_, '_, '_, Cap>
where
    Cap: ImplCap<CapabilityUiInteraction>,
{
    /// Check if entity is hovered. Logic reads [`bevy_picking::hover::Hovered`]
    fn hovered(&mut self) -> bool {
        let Ok(Some(hovered)) = self.cap_get_component::<Hovered>() else {
            self.entity_commands().insert(Hovered::default());
            return false;
        };
        hovered.get()
    }

    /// Checks if entity contains [`Pressed`]
    ///
    /// To insert, remove [`Pressed`] component to entity
    /// use similar logic as in [`bevy_ui_widgets::ButtonPlugin`]
    fn pressed(&self) -> bool {
        let Ok(entity) = self.cap_get_entity() else {
            return false;
        };
        entity.contains::<Pressed>()
    }
}
