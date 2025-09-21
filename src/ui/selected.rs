use crate::{CapSet, ImmCapability, ImmEntity, ImplCap};

/// Capability that makes [`bevy_ui::Interaction`] accessible from immediate UI
pub struct CapabilityUiSelected;

impl ImmCapability for CapabilityUiSelected {
    fn build<Cap: CapSet>(app: &mut bevy_app::App, cap_req: &mut crate::ImmCapAccessRequests<Cap>) {
        cap_req.request_component_write::<Selected>(app.world_mut());
    }
}

/// Marks component as being selectable
#[derive(bevy_ecs::component::Component)]
pub struct Selected {
    /// Is selectable component selected
    pub selected: bool,
}

/// Implements methods to access [`bevy_ui::Interaction`] in immediate mode
pub trait ImmUiSelected {
    /// Insert [`Selected`] component with given boolean value
    ///
    /// Useful for styling purposes
    fn selected(self, selected: bool) -> Self;
}

impl<Cap> ImmUiSelected for ImmEntity<'_, '_, '_, Cap>
where
    Cap: ImplCap<CapabilityUiSelected>,
{
    fn selected(mut self, selected: bool) -> Self {
        if let Ok(Some(mut comp)) = self.cap_get_component_mut::<Selected>() {
            if comp.selected == selected {
                return self;
            }
            comp.selected = selected;
            return self;
        }

        self.entity_commands().insert(Selected { selected });
        self
    }
}
