use bevy_ui::Interaction;

use crate::{ImmCap, ImmEntity, ImmImplCap};

/// Capability that makes [`bevy_ui::Interaction`] accessible from immediate UI
pub struct ImmCapUiInteraction;

impl ImmCap for ImmCapUiInteraction {
    fn build<Cap: ImmCap>(app: &mut bevy_app::App, cap_req: &mut crate::CapAccessRequests<Cap>) {
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
    Cap: ImmImplCap<ImmCapUiInteraction>,
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
