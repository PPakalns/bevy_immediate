use bevy_color::Color;
use bevy_ui::{BackgroundColor, BorderColor};

use crate::{CapSet, ImmCapability, ImmEntity, ImplCap};

/// Implements capability to modify ui node look
pub struct CapabilityUiLook;

impl ImmCapability for CapabilityUiLook {
    fn build<Cap: CapSet>(app: &mut bevy_app::App, cap_req: &mut crate::ImmCapAccessRequests<Cap>) {
        let _ = cap_req;
        let _ = app;
        cap_req.request_component_write::<BackgroundColor>(app.world_mut());
        cap_req.request_component_write::<BorderColor>(app.world_mut());
    }
}

/// Implements methods to modify ui node look
pub trait ImmUiLook {
    /// Set BackgroundColor
    fn background_color(self, value: Color) -> Self;
    /// Set BorderColor
    fn border_color(self, value: &BorderColor) -> Self;
}

impl<Cap> ImmUiLook for ImmEntity<'_, '_, '_, Cap>
where
    Cap: ImplCap<CapabilityUiLook>,
{
    fn background_color(mut self, value: Color) -> Self {
        if let Some(mut entity) = self.cap_get_entity_mut().ok() {
            if let Some(mut current_value) = entity.get_mut::<BackgroundColor>() {
                if current_value.0 != value {
                    current_value.0 = value;
                }
                return self;
            }
        }

        let mut commands = self.entity_commands();
        commands.insert(BackgroundColor(value));
        self
    }

    fn border_color(mut self, value: &BorderColor) -> Self {
        if let Some(mut entity) = self.cap_get_entity_mut().ok() {
            if let Some(mut current_value) = entity.get_mut::<BorderColor>() {
                if *current_value != *value {
                    *current_value = *value;
                }
                return self;
            }
        }

        let mut commands = self.entity_commands();
        commands.insert(*value);
        self
    }
}
