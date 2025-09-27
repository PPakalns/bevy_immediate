use bevy_feathers::controls::ButtonVariant;

use crate::{CapSet, ImmCapability, ImmEntity, ImplCap};

/// Implements capability to set slider base color
pub struct CapabilityUiFeathersButtonVariant;

impl ImmCapability for CapabilityUiFeathersButtonVariant {
    fn build<Cap: CapSet>(app: &mut bevy_app::App, cap_req: &mut crate::ImmCapAccessRequests<Cap>) {
        let _ = cap_req;
        let _ = app;
        cap_req.request_component_write::<ButtonVariant>(app.world_mut());
    }
}

/// Implements capability to set button variant
pub trait ImmUiFeathersButtonVariant {
    /// Set primary or normal button variant
    fn primary_button(self, primary: bool) -> Self;

    /// Set button variant
    fn button_variant(self, variant: ButtonVariant) -> Self;
}

impl<Cap> ImmUiFeathersButtonVariant for ImmEntity<'_, '_, '_, Cap>
where
    Cap: ImplCap<CapabilityUiFeathersButtonVariant>,
{
    fn primary_button(self, primary: bool) -> Self {
        let to_set = match primary {
            true => ButtonVariant::Primary,
            false => ButtonVariant::Normal,
        };

        self.button_variant(to_set)
    }

    fn button_variant(mut self, variant_to_set: ButtonVariant) -> Self {
        if let Ok(Some(mut variant)) = self.cap_get_component_mut::<ButtonVariant>() {
            if *variant != variant_to_set {
                *variant = variant_to_set;
            }
            return self;
        }
        self.entity_commands().insert(variant_to_set);
        self
    }
}
