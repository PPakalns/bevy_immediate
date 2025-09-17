use crate::{ImmCap, ImmEntity, ImmImplCap};

/// Capability that makes [`bevy_ui::Interaction`] accessible from immediate UI
pub struct ImmCapUiInteraction;

impl ImmCap for ImmCapUiInteraction {
    fn build<Cap: ImmCap>(app: &mut bevy_app::App, cap_req: &mut crate::CapAccessRequests<Cap>) {
        cap_req.request_optional_component::<bevy_ui::Interaction>(app.world_mut(), false);
    }
}

/// Implements methods to access [`bevy_ui::Interaction`] in immediate mode
pub trait ImmUiInteraction {
    /// Retrieve current [`bevy_ui::Interaction`] state for entity
    fn interaction(&mut self) -> bevy_ui::Interaction;

    /// Is [`bevy_ui::Interaction::Pressed`]
    fn pressed(&mut self) -> bool;

    /// Is [`bevy_ui::Interaction::Hovered`]
    fn hovered(&mut self) -> bool;
}

impl<Cap> ImmUiInteraction for ImmEntity<'_, '_, '_, Cap>
where
    Cap: ImmImplCap<ImmCapUiInteraction>,
{
    fn interaction(&mut self) -> bevy_ui::Interaction {
        let entity = self.entity();

        let mut query = self
            .ctx()
            .query
            .get_query::<Option<&bevy_ui::Interaction>>();

        match query.query().get(entity) {
            Ok(Some(entity)) => *entity,
            Ok(None) | Err(_) => {
                self.entity_commands()
                    .insert_if_new(bevy_ui::Interaction::default());
                bevy_ui::Interaction::None
            }
        }
    }

    fn pressed(&mut self) -> bool {
        self.interaction() == bevy_ui::Interaction::Pressed
    }

    fn hovered(&mut self) -> bool {
        self.interaction() == bevy_ui::Interaction::Hovered
    }
}
