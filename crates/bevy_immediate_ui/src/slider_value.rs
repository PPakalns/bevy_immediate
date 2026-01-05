use bevy_ui_widgets::SliderValue;

use crate::track_value_change_plugin::{NewValueChange, TrackValueChangePlugin};
use bevy_immediate_core::{CapSet, ImmCapAccessRequests, ImmCapability, ImmEntity, ImplCap};

/// Implements capability to set slider value
pub struct CapabilityUiSliderValue;

impl ImmCapability for CapabilityUiSliderValue {
    fn build<Cap: CapSet>(app: &mut bevy_app::App, cap_req: &mut ImmCapAccessRequests<Cap>) {
        let _ = cap_req;
        let _ = app;
        cap_req.request_component_write::<NewValueChange<f32>>(app.world_mut());
        cap_req.request_component_read::<SliderValue>(app.world_mut());

        if !app.is_plugin_added::<TrackValueChangePlugin<f32>>() {
            app.add_plugins(TrackValueChangePlugin::<f32>::default());
        }
    }
}

/// Implements capability to set slider value
pub trait ImmUiSliderValue {
    /// Update slider value [SliderValue].
    fn slider(self, value: &mut f32) -> Self;
    /// Update slider value [SliderValue] with custom funtion to get, set value.
    ///
    /// Useful when working with integer values.
    fn slider_get_set(self, get_set: impl FnMut(Option<f32>) -> f32) -> Self;
}

impl<Cap> ImmUiSliderValue for ImmEntity<'_, '_, '_, Cap>
where
    Cap: ImplCap<CapabilityUiSliderValue>,
{
    fn slider(mut self, value: &mut f32) -> Self {
        'initialized: {
            let Ok(mut entity) = self.cap_get_entity_mut() else {
                break 'initialized;
            };

            let Some(mut new_value) = entity.get_mut::<NewValueChange<f32>>() else {
                break 'initialized;
            };
            let new_value = NewValueChange::take(&mut new_value);

            let Some(slider_value) = entity.get::<SliderValue>() else {
                break 'initialized;
            };
            let slider_value = slider_value.0;

            if *value != slider_value {
                // Checked value changed
                self.entity_commands().insert(SliderValue(*value));
            }

            if let Some(new_value) = new_value
                // Avoid update loop
                && new_value != slider_value
            {
                *value = new_value;
            }

            return self;
        }

        let mut commands = self.entity_commands();
        commands
            .insert(NewValueChange::<f32>::default())
            .insert(SliderValue(*value));
        self
    }

    fn slider_get_set(mut self, mut get_set: impl FnMut(Option<f32>) -> f32) -> Self {
        let current = get_set(None);
        let mut new = current;
        self = self.slider(&mut new);
        if new != current {
            get_set(Some(new));
        }
        self
    }
}

////////////////////////////////////////////////////////////////////////////////
