use bevy_ecs::hierarchy::Children;
use bevy_feathers::controls::{NumberInputValue, UpdateNumberInput};
use bevy_input_focus::InputFocus;

use crate::track_value_change_plugin::{NewValueChange, TrackValueChangePlugin};
use bevy_immediate_core::{
    CapSet, ImmCapAccessRequests, ImmCapability, ImmEntity, ImplCap, imm_id,
    utils::ImmLocalHashMemoryHelper,
};

/// Capability for synchronising [`bevy_feathers::controls::FeathersNumberInput`] widgets.
pub struct CapabilityUiNumberInput;

impl ImmCapability for CapabilityUiNumberInput {
    fn build<Cap: CapSet>(app: &mut bevy_app::App, cap_req: &mut ImmCapAccessRequests<Cap>) {
        register_number_input_value::<f32>(app, cap_req);
        register_number_input_value::<f64>(app, cap_req);
        register_number_input_value::<i32>(app, cap_req);
        register_number_input_value::<i64>(app, cap_req);
    }
}

fn register_number_input_value<T>(
    app: &mut bevy_app::App,
    cap_req: &mut ImmCapAccessRequests<impl CapSet>,
) where
    T: ImmUiNumberInputValue,
{
    cap_req.request_component_write::<NewValueChange<T>>(app.world_mut());
    cap_req.request_resource_read::<InputFocus>(app.world_mut());
    cap_req.request_component_read::<Children>(app.world_mut());

    if !app.is_plugin_added::<TrackValueChangePlugin<T>>() {
        app.add_plugins(TrackValueChangePlugin::<T>::default());
    }
}

/// Synchronises a [`FeathersNumberInput`] widget with stored application state.
pub trait ImmUiNumberInput {
    /// Synchronises a [`FeathersNumberInput`] widget with stored application state.
    fn number_input<T: ImmUiNumberInputValue>(self, value: &mut T) -> Self;

    /// Synchronise value using a get/set callback.
    ///
    /// Useful for values that could contain invalid intermediate state while editing.
    fn number_input_get_set<T: ImmUiNumberInputValue>(
        self,
        get_set: impl FnMut(Option<T>) -> T,
    ) -> Self;
}

impl<Cap> ImmUiNumberInput for ImmEntity<'_, '_, '_, Cap>
where
    Cap: ImplCap<CapabilityUiNumberInput>,
{
    fn number_input_get_set<T: ImmUiNumberInputValue>(
        mut self,
        mut get_set: impl FnMut(Option<T>) -> T,
    ) -> Self {
        let is_input_focused = self
            .cap_get_resource::<InputFocus>()
            .ok()
            .and_then(|focus| focus.get())
            .is_some_and(|focus_entity| {
                focus_entity == self.entity()
                    || self
                        .cap_get_entity()
                        .ok()
                        .and_then(|entity| entity.get::<Children>())
                        .is_some_and(|children| children.contains(&focus_entity))
            });

        let external = get_set(None);

        if is_input_focused {
            let mut stored_comp_hash =
                ImmLocalHashMemoryHelper::new(&mut self, "__number_input_comp", &None);

            if let Ok(Some(mut new_value)) = self.cap_get_component_mut::<NewValueChange<T>>()
                && let Some(comp_value) = NewValueChange::take(&mut new_value)
            {
                let comp_hash = imm_id(comp_value.hash_bits());
                let external_hash = imm_id(external.hash_bits());

                if !stored_comp_hash.is_stored(&Some(comp_hash)) {
                    if comp_hash != external_hash {
                        get_set(Some(comp_value));
                    }
                    stored_comp_hash.store(&Some(comp_hash));
                }
            }

            stored_comp_hash.finalize(&mut self);
        } else {
            let mut buffer = external;
            self = self.number_input(&mut buffer);

            if buffer != external {
                get_set(Some(buffer));
            }
        }

        self
    }

    fn number_input<T: ImmUiNumberInputValue>(mut self, value: &mut T) -> Self {
        let value_hash = value.hash_bits();
        let mut helper = ImmLocalHashMemoryHelper::new(&mut self, T::memory_key(), &value_hash);

        'initialized: {
            let Ok(Some(mut new_value)) = self.cap_get_component_mut::<NewValueChange<T>>() else {
                break 'initialized;
            };

            if let Some(new_value) = NewValueChange::take(&mut new_value) {
                *value = new_value;
                helper.store(&value.hash_bits());
            }

            if !helper.is_stored(&value.hash_bits()) {
                helper.store(&value.hash_bits());
                let entity = self.entity();
                self.commands().trigger(UpdateNumberInput {
                    entity,
                    value: value.to_number_input_value(),
                });
            }

            helper.finalize(&mut self);
            return self;
        }

        let entity_id = self.entity();
        self.entity_commands()
            .insert(NewValueChange::<T>::default());
        self.commands().trigger(UpdateNumberInput {
            entity: entity_id,
            value: value.to_number_input_value(),
        });
        helper.finalize(&mut self);
        self
    }
}

/// Maps application number types to [`NumberInputValue`].
pub trait ImmUiNumberInputValue: Copy + Clone + PartialEq + Send + Sync + 'static {
    /// Widget value for [`UpdateNumberInput`].
    fn to_number_input_value(self) -> NumberInputValue;

    /// Per-type immediate-mode hash memory key.
    fn memory_key() -> &'static str;

    /// Stable hash for synchronisation memory.
    fn hash_bits(self) -> u64;
}

impl ImmUiNumberInputValue for f32 {
    fn to_number_input_value(self) -> NumberInputValue {
        NumberInputValue::F32(self)
    }

    fn memory_key() -> &'static str {
        "__bevy_ui_number_input_f32"
    }

    fn hash_bits(self) -> u64 {
        self.to_bits() as u64
    }
}

impl ImmUiNumberInputValue for f64 {
    fn to_number_input_value(self) -> NumberInputValue {
        NumberInputValue::F64(self)
    }

    fn memory_key() -> &'static str {
        "__bevy_ui_number_input_f64"
    }

    fn hash_bits(self) -> u64 {
        self.to_bits()
    }
}

impl ImmUiNumberInputValue for i32 {
    fn to_number_input_value(self) -> NumberInputValue {
        NumberInputValue::I32(self)
    }

    fn memory_key() -> &'static str {
        "__bevy_ui_number_input_i32"
    }

    fn hash_bits(self) -> u64 {
        self as u32 as u64
    }
}

impl ImmUiNumberInputValue for i64 {
    fn to_number_input_value(self) -> NumberInputValue {
        NumberInputValue::I64(self)
    }

    fn memory_key() -> &'static str {
        "__bevy_ui_number_input_i64"
    }

    fn hash_bits(self) -> u64 {
        self as u64
    }
}
