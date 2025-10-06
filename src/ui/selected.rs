use crate::{CapSet, ImmCapability, ImmEntity, ImplCap};

/// Implements capability to mark entities as selectable.
pub struct CapabilityUiSelectable;

impl ImmCapability for CapabilityUiSelectable {
    fn build<Cap: CapSet>(app: &mut bevy_app::App, cap_req: &mut crate::ImmCapAccessRequests<Cap>) {
        cap_req.request_component_write::<Selectable>(app.world_mut());
    }
}

/// Marks component as being selectable
#[derive(bevy_ecs::component::Component)]
pub struct Selectable {
    /// Is selectable component selected
    pub selected: bool,
}

/// Implements methods to set entity selectable
pub trait ImmUiSelectable {
    /// Insert [`Selected`] component with given boolean value
    ///
    /// Useful for styling purposes
    fn selected(self, selected: bool) -> Self;
}

impl<Cap> ImmUiSelectable for ImmEntity<'_, '_, '_, Cap>
where
    Cap: ImplCap<CapabilityUiSelectable>,
{
    fn selected(mut self, selected: bool) -> Self {
        if let Ok(Some(mut comp)) = self.cap_get_component_mut::<Selectable>() {
            if comp.selected != selected {
                comp.selected = selected;
            }
            return self;
        }

        self.entity_commands().insert(Selectable { selected });
        self
    }
}
