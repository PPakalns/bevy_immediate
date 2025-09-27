use bevy_color::Color;
use bevy_ecs::world::Mut;
use bevy_ui::{BackgroundColor, BorderColor, Node};

use crate::{CapSet, ImmCapability, ImmEntity, ImplCap};

/// Implements capability to modify ui node look
pub struct CapabilityUiLook;

impl ImmCapability for CapabilityUiLook {
    fn build<Cap: CapSet>(app: &mut bevy_app::App, cap_req: &mut crate::ImmCapAccessRequests<Cap>) {
        let _ = cap_req;
        let _ = app;
        cap_req.request_component_write::<Node>(app.world_mut());
        cap_req.request_component_write::<BackgroundColor>(app.world_mut());
        cap_req.request_component_write::<BorderColor>(app.world_mut());
    }
}

/// Implements methods to modify ui node look
pub trait ImmUiLook {
    /// Modify [Node] value on entity
    ///
    /// Given function will not be called if entity doesn't have Node component
    fn node_mut(self, f: impl FnOnce(&mut Mut<'_, Node>)) -> Self;

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
        if let Ok(Some(mut current_value)) = self.cap_get_component_mut::<BackgroundColor>() {
            if current_value.0 != value {
                current_value.0 = value;
            }
            return self;
        }

        let mut commands = self.entity_commands();
        commands.insert(BackgroundColor(value));
        self
    }

    fn border_color(mut self, value: &BorderColor) -> Self {
        if let Ok(Some(mut current_value)) = self.cap_get_component_mut::<BorderColor>() {
            if *current_value != *value {
                *current_value = *value;
            }
            return self;
        }

        let mut commands = self.entity_commands();
        commands.insert(*value);
        self
    }

    fn node_mut(mut self, f: impl FnOnce(&mut Mut<'_, Node>)) -> Self {
        if let Ok(Some(mut value)) = self.cap_get_component_mut::<Node>() {
            f(&mut value)
        }
        self
    }
}
