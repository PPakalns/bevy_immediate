use bevy_ecs::component::Component;
use bevy_ui_widgets::SliderValue;

use crate::{CapSet, ImmCapability, ImmEntity, ImplCap};

/// Implements capability to set slider value
pub struct CapabilityUiSliderValue;

impl ImmCapability for CapabilityUiSliderValue {
    fn build<Cap: CapSet>(app: &mut bevy_app::App, cap_req: &mut crate::ImmCapAccessRequests<Cap>) {
        let _ = cap_req;
        let _ = app;
        cap_req.request_component_write::<StoredSliderValue>(app.world_mut());
        cap_req.request_component_read::<SliderValue>(app.world_mut());
    }
}

#[derive(Component)]
struct StoredSliderValue {
    value: f32,
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
        if let Ok(mut entity) = self.cap_get_entity_mut() {
            let current_value = entity.get::<SliderValue>().map(|val| val.0);
            let last_value = entity.get::<StoredSliderValue>().map(|val| val.value);

            if let (Some(current_value), Some(last_value)) = (current_value, last_value) {
                if last_value != current_value {
                    // Component has triggered checked value change

                    *value = current_value;
                    entity.get_mut::<StoredSliderValue>().unwrap().value = current_value;
                    return self;
                }

                if *value != last_value {
                    // Checked value changed
                    entity.get_mut::<StoredSliderValue>().unwrap().value = *value;
                    self.entity_commands().insert(SliderValue(*value));
                }

                return self;
            }
        }

        let mut commands = self.entity_commands();
        commands.insert((StoredSliderValue { value: *value }, SliderValue(*value)));
        self
    }
}
