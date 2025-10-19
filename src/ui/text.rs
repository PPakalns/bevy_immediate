use std::ops::Deref;

use bevy_ui::widget::Text;

use crate::{CapSet, ImmCapability, ImmEntity, ImplCap, imm_id};

/// Functionality to manage text rendering inside ui
pub struct CapabilityUiText;

impl ImmCapability for CapabilityUiText {
    fn build<Cap: CapSet>(app: &mut bevy_app::App, cap_req: &mut crate::ImmCapAccessRequests<Cap>) {
        cap_req.request_component_write::<Text>(app.world_mut());
    }
}

/// Implements methods to update [`Text`] in immediate mode
pub trait ImmUiText {
    /// Insert [`Text`] on entity spawn and update it to given text upon change
    fn text(self, text: impl Deref<Target = str> + Into<String>) -> Self;

    /// On entity spawn insert given text into [`Text`]
    fn on_spawn_text_fn(self, text: impl FnOnce() -> String) -> Self;

    /// On entity spawn insert given text into [`Text`]
    fn on_spawn_text(self, text: &str) -> Self;

    /// Insert text if something changed
    fn on_change_text_fn(self, changed: bool, text: impl FnOnce() -> String) -> Self;

    /// Update text upon hash change for given `hash_source`
    fn on_change_hash_text_fn<T: std::hash::Hash>(
        self,
        hash_source: &T,
        text: impl FnOnce() -> String,
    ) -> Self;
}

impl<Cap> ImmUiText for ImmEntity<'_, '_, '_, Cap>
where
    Cap: ImplCap<CapabilityUiText>,
{
    fn text(mut self, text: impl Deref<Target = str> + Into<String>) -> Self {
        'text_exists: {
            let Ok(Some(mut text_comp)) = self.cap_get_component_mut::<Text>() else {
                break 'text_exists;
            };

            // No need to update text and trigger state change
            if text_comp.0 == text.deref() {
                return self;
            }
            *text_comp = Text(text.into());

            return self;
        }

        // Fallback
        self.entity_commands().insert_if_new(Text(text.into()));
        self
    }

    fn on_spawn_text_fn(self, text: impl FnOnce() -> String) -> Self {
        self.on_spawn_insert(|| Text(text()))
    }

    fn on_spawn_text(self, text: &str) -> Self {
        self.on_spawn_insert(|| Text(text.to_owned()))
    }

    fn on_change_text_fn(self, changed: bool, text: impl FnOnce() -> String) -> Self {
        self.on_change_insert(changed, || Text(text()))
    }

    fn on_change_hash_text_fn<T: std::hash::Hash>(
        mut self,
        hash_source: &T,
        text: impl FnOnce() -> String,
    ) -> Self {
        struct SealedKey;

        let source = imm_id(hash_source);
        let current = self.hash_get_typ::<SealedKey>();

        if current != Some(source) {
            self.hash_set_typ::<SealedKey>(source);

            if let Ok(Some(mut text_comp)) = self.cap_get_component_mut::<Text>() {
                text_comp.0 = text();
            } else {
                self.entity_commands().insert(Text(text()));
            }
        }

        self
    }
}
