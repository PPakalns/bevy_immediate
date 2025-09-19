use bevy_ui::Interaction;

use crate::{CapSet, ImmCapability, ImmEntity, ImplCap};

/// Capability that makes [`bevy_ui::Interaction`] accessible from immediate UI
pub struct CapabilityUiInteraction;

impl ImmCapability for CapabilityUiInteraction {
    fn build<Cap: CapSet>(app: &mut bevy_app::App, cap_req: &mut crate::ImmCapAccessRequests<Cap>) {
        cap_req.request_component_read::<Interaction>(app.world_mut());
    }
}

/// Implements methods to access [`bevy_ui::Interaction`] in immediate mode
pub trait ImmUiInteraction {
    /// Retrieve current [`bevy_ui::Interaction`] state for entity
    fn interaction(&mut self) -> Interaction;

    /// Is [`bevy_ui::Interaction::Pressed`]
    fn pressed(&mut self) -> bool;

    /// Is [`bevy_ui::Interaction::Hovered`]
    fn hovered(&mut self) -> bool;
}

impl<Cap> ImmUiInteraction for ImmEntity<'_, '_, '_, Cap>
where
    Cap: ImplCap<CapabilityUiInteraction>,
{
    fn interaction(&mut self) -> Interaction {
        'correct: {
            let Ok(Some(interaction)) = self.cap_get_component::<Interaction>() else {
                break 'correct;
            };

            return *interaction;
        }

        // Component should have `Interaction` component
        self.entity_commands().insert_if_new(Interaction::default());
        Interaction::None
    }

    fn pressed(&mut self) -> bool {
        self.interaction() == Interaction::Pressed
    }

    fn hovered(&mut self) -> bool {
        self.interaction() == Interaction::Hovered
    }
}
