use bevy_color::Color;
use bevy_feathers::controls::ColorSwatchValue;

use bevy_immediate_core::{CapSet, ImmCapAccessRequests, ImmCapability, ImmEntity, ImplCap};

/// Capability for synchronising [`bevy_feathers::controls::FeathersColorSwatch`] widgets.
pub struct CapabilityUiColorSwatch;

impl ImmCapability for CapabilityUiColorSwatch {
    fn build<Cap: CapSet>(app: &mut bevy_app::App, cap_req: &mut ImmCapAccessRequests<Cap>) {
        let _ = app;
        cap_req.request_component_write::<ColorSwatchValue>(app.world_mut());
    }
}

/// Synchronises a [`FeathersColorSwatch`] widget with a stored color.
pub trait ImmUiColorSwatch {
    /// Push `color` into the swatch's [`ColorSwatchValue`].
    fn color_swatch(self, color: Color) -> Self;
}

impl<Cap> ImmUiColorSwatch for ImmEntity<'_, '_, '_, Cap>
where
    Cap: ImplCap<CapabilityUiColorSwatch>,
{
    fn color_swatch(mut self, color: Color) -> Self {
        if let Ok(Some(mut swatch_value)) = self.cap_get_component_mut::<ColorSwatchValue>() {
            if swatch_value.0 != color {
                swatch_value.0 = color;
            }
            return self;
        }

        self.entity_commands().insert(ColorSwatchValue(color));
        self
    }
}
