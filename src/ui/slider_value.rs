use bevy_ui_widgets::SliderValue;

use crate::{
    CapSet, ImmCapability, ImmEntity, ImplCap,
    ui::track_value_change_plugin::{NewValueChange, TrackValueChangePlugin},
};

/// Implements capability to set slider value
pub struct CapabilityUiSliderValue;

impl ImmCapability for CapabilityUiSliderValue {
    fn build<Cap: CapSet>(app: &mut bevy_app::App, cap_req: &mut crate::ImmCapAccessRequests<Cap>) {
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

            let Some(last_value) = entity.get::<SliderValue>() else {
                break 'initialized;
            };
            let last_value = last_value.0;

            if let Some(new_value) = new_value
                // Avoid update loop
                && new_value != last_value
            {
                *value = new_value;
                self.entity_commands().insert(SliderValue(new_value));
                return self;
            }

            if *value != last_value {
                // Checked value changed
                self.entity_commands().insert(SliderValue(*value));
            }

            return self;
        }

        let mut commands = self.entity_commands();
        commands
            .insert(NewValueChange::<f32>::default())
            .insert(SliderValue(*value));
        self
    }
}

////////////////////////////////////////////////////////////////////////////////
