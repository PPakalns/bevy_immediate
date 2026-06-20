use bevy_feathers::controls::ColorPlaneValue;
use bevy_math::{Vec2, Vec3};

use crate::track_value_change_plugin::{NewValueChange, TrackValueChangePlugin};
use bevy_immediate_core::{CapSet, ImmCapAccessRequests, ImmCapability, ImmEntity, ImplCap};

/// Capability for synchronising [`bevy_feathers::controls::FeathersColorPlane`] widgets.
pub struct CapabilityUiColorPlane;

impl ImmCapability for CapabilityUiColorPlane {
    fn build<Cap: CapSet>(app: &mut bevy_app::App, cap_req: &mut ImmCapAccessRequests<Cap>) {
        cap_req.request_component_write::<NewValueChange<Vec2>>(app.world_mut());
        cap_req.request_component_write::<ColorPlaneValue>(app.world_mut());

        if !app.is_plugin_added::<TrackValueChangePlugin<Vec2>>() {
            app.add_plugins(TrackValueChangePlugin::<Vec2>::default());
        }
    }
}

/// Synchronises [`FeathersColorPlane`] widgets with stored state.
pub trait ImmUiColorPlane {
    /// Synchronises color plane value and `ColorPlaneValue`
    fn color_plane_get_set(self, get_set: impl FnMut(Option<Vec2>) -> Vec3) -> Self;

    /// Set `ColorPlaneValue`
    fn color_plane_set(self, value: Vec3) -> Self;
}

impl<Cap> ImmUiColorPlane for ImmEntity<'_, '_, '_, Cap>
where
    Cap: ImplCap<CapabilityUiColorPlane>,
{
    fn color_plane_get_set(mut self, mut get_set: impl FnMut(Option<Vec2>) -> Vec3) -> Self {
        let mut value = get_set(None);

        'initialized: {
            let Ok(mut entity) = self.cap_get_entity_mut() else {
                break 'initialized;
            };

            if let Some(mut new_value) = entity.get_mut::<NewValueChange<Vec2>>()
                && let Some(xy) = NewValueChange::take(&mut new_value)
            {
                value = get_set(Some(xy));
            }

            if let Some(mut plane_value) = entity.get_mut::<ColorPlaneValue>()
                && plane_value.0 != value
            {
                plane_value.0 = value;
            }

            return self;
        }

        self.entity_commands()
            .insert((NewValueChange::<Vec2>::default(), ColorPlaneValue(value)));
        self
    }

    fn color_plane_set(mut self, value: Vec3) -> Self {
        'initialized: {
            let Ok(Some(mut plane_value)) = self.cap_get_component_mut::<ColorPlaneValue>() else {
                break 'initialized;
            };

            if plane_value.0 != value {
                plane_value.0 = value;
            }

            return self;
        }

        self.entity_commands()
            .insert((NewValueChange::<Vec2>::default(), ColorPlaneValue(value)));
        self
    }
}
