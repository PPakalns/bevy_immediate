use bevy_color::Color;
use bevy_feathers::controls::SliderBaseColor;

use crate::{CapSet, ImmCapability, ImmEntity, ImplCap};

/// Implements capability to set slider base color
pub struct CapabilityUiSliderBaseColor;

impl ImmCapability for CapabilityUiSliderBaseColor {
    fn build<Cap: CapSet>(app: &mut bevy_app::App, cap_req: &mut crate::ImmCapAccessRequests<Cap>) {
        let _ = cap_req;
        let _ = app;
        cap_req.request_component_write::<SliderBaseColor>(app.world_mut());
    }
}

/// Implements capability to set slider base color
pub trait ImmUiSliderBaseColor {
    /// Update slider base color [SliderBaseColor].
    fn slider_base_color(self, value: Color) -> Self;
}

impl<Cap> ImmUiSliderBaseColor for ImmEntity<'_, '_, '_, Cap>
where
    Cap: ImplCap<CapabilityUiSliderBaseColor>,
{
    fn slider_base_color(mut self, value: Color) -> Self {
        if let Some(mut entity) = self.cap_get_entity_mut().ok() {
            if let Some(mut current_value) = entity.get_mut::<SliderBaseColor>() {
                if current_value.0 != value {
                    current_value.0 = value;
                }
                return self;
            }
        }

        let mut commands = self.entity_commands();
        commands.insert(SliderBaseColor(value));
        self
    }
}
