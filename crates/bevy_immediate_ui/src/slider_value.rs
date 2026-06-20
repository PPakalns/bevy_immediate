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
    /// Synchronise slider value with [`SliderValue`].
    fn slider(self, value: &mut f32) -> Self;

    /// Set slider value without reading edits back from the widget.
    fn slider_set(self, value: f32) -> Self;

    /// Synchronise slider value using a get/set callback.
    fn slider_get_set(self, get_set: impl FnMut(Option<f32>) -> f32) -> Self;
}

impl<Cap> ImmUiSliderValue for ImmEntity<'_, '_, '_, Cap>
where
    Cap: ImplCap<CapabilityUiSliderValue>,
{
    fn slider_get_set(mut self, mut get_set: impl FnMut(Option<f32>) -> f32) -> Self {
        let current = get_set(None);
        let mut new = current;
        self = self.slider(&mut new);
        if new != current {
            get_set(Some(new));
        }
        self
    }

    fn slider_set(mut self, value: f32) -> Self {
        'initialized: {
            let Ok(entity) = self.cap_get_entity() else {
                break 'initialized;
            };

            if let Some(slider_value) = entity.get::<SliderValue>() {
                if slider_value.0 != value {
                    self.entity_commands().insert(SliderValue(value));
                }
                return self;
            }
        }

        self.entity_commands().insert(SliderValue(value));
        self
    }

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
                self.entity_commands().insert(SliderValue(*value));
            }

            if let Some(new_value) = new_value
                && new_value != slider_value
            {
                *value = new_value;
            }

            return self;
        }

        self.entity_commands()
            .insert((NewValueChange::<f32>::default(), SliderValue(*value)));
        self
    }
}
