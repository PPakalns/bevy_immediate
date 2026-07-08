use bevy_ecs::{change_detection::DetectChanges, system::EntityCommand};
use bevy_input_focus::InputFocus;
use bevy_text::EditableText;

use bevy_immediate_core::{
    CapSet, ImmCapAccessRequests, ImmCapability, ImmEntity, ImplCap, imm_id,
    utils::ImmLocalHashMemoryHelper,
};
use bevy_ui::widget::Text;

/// Capability that allows reading and writing UI input text in immediate mode.
pub struct CapabilityUiTextInput;

impl ImmCapability for CapabilityUiTextInput {
    fn build<Cap: CapSet>(app: &mut bevy_app::App, cap_req: &mut ImmCapAccessRequests<Cap>) {
        cap_req.request_component_write::<EditableText>(app.world_mut());
        cap_req.request_resource_read::<InputFocus>(app.world_mut());
    }
}

/// Implements methods to synchronise [`EditableText`] in immediate mode.
pub trait ImmUiTextInput {
    /// Synchronise text with an [`EditableText`] component on the entity.
    fn input_text(self, text: &mut String) -> Self;

    /// Set the displayed text
    fn input_set(self, text: impl Into<String>) -> Self;

    /// Synchronise text using a get/set callback.
    ///
    /// Useful for small values that could contain invalid intermediate state.
    fn input_get_set(self, get_set: impl FnMut(Option<String>) -> String) -> Self;
}

impl<Cap> ImmUiTextInput for ImmEntity<'_, '_, '_, Cap>
where
    Cap: ImplCap<CapabilityUiTextInput>,
{
    fn input_get_set(mut self, mut get_set: impl FnMut(Option<String>) -> String) -> Self {
        let is_input_focused = self
            .cap_get_resource::<InputFocus>()
            .ok()
            .and_then(|focus| focus.get())
            == Some(self.entity());

        let mut external = get_set(None);

        if is_input_focused {
            let mut stored_comp_hash =
                ImmLocalHashMemoryHelper::new(&mut self, "__text_input_comp", &None);
            // let mut stored_external_hash =
            //     ImmLocalHashMemoryHelper::new(&mut self, "__text_input_external", &None);

            if let Ok(Some(editable_text)) = self.cap_get_component_mut::<EditableText>() {
                let comp_value = editable_text.editor.text().to_string();
                let comp_hash = imm_id(&comp_value);

                let mut external_hash = imm_id(&external);

                if editable_text.is_changed() && !stored_comp_hash.is_stored(&Some(comp_hash)) {
                    // Widget text changed
                    if comp_hash != external_hash {
                        external = get_set(Some(comp_value));

                        external_hash = imm_id(&external);
                        let _ = external_hash;
                    }
                    stored_comp_hash.store(&Some(comp_hash));
                }

                // if !stored_external_hash.is_stored(&Some(external_hash)) {
                //     if external_hash != comp_hash {
                //         editable_text.editor.set_text(&external);
                //     }
                //     stored_external_hash.store(&Some(external_hash));
                // }
            }

            // stored_external_hash.finalize(&mut self);
            stored_comp_hash.finalize(&mut self);
        } else {
            let mut buffer = external.clone();
            self = self.input_text(&mut buffer);

            if buffer != external {
                get_set(Some(buffer));
            }
        }

        self
    }

    fn input_set(mut self, text: impl Into<String>) -> Self {
        let text = text.into();

        'exist: {
            let Ok(Some(mut input_buffer)) = self.cap_get_component_mut::<EditableText>() else {
                break 'exist;
            };

            if input_buffer.editor.text() != text.as_str() {
                input_buffer.editor.set_text(&text);
            }

            return self;
        }

        if self.will_be_spawned() {
            self.entity_commands().queue_silenced(SetText { text });
        }

        self
    }

    fn input_text(mut self, text: &mut String) -> Self {
        let text_hash = imm_id(&text);
        let mut helper =
            ImmLocalHashMemoryHelper::new(&mut self, "__bevy_ui_text_input", &text_hash);

        'exist: {
            let Ok(mut entity) = self.cap_get_entity_mut() else {
                break 'exist;
            };

            let Some(mut input_buffer) = entity.get_mut::<EditableText>() else {
                break 'exist;
            };

            if !helper.is_stored(&text_hash) {
                helper.store(&text_hash);

                // External value updated
                input_buffer.editor.set_text(text);
            } else {
                let not_equal = input_buffer.is_changed() // Fast check
                    && input_buffer.editor.text() != text;

                if not_equal {
                    *text = input_buffer.editor.text().to_string();

                    helper.store(&imm_id(&text));
                }
            }

            helper.finalize(&mut self);
            return self;
        }

        helper.finalize(&mut self);
        if self.will_be_spawned() {
            self.entity_commands().insert(Text(text.clone()));
        } else {
            self.entity_commands()
                .queue_silenced(SetText { text: text.clone() });
        }

        self
    }
}

struct SetText {
    text: String,
}

impl EntityCommand for SetText {
    type Out = ();

    fn apply(self, mut entity: bevy_ecs::world::EntityWorldMut) -> Self::Out {
        if let Some(mut textedit) = entity.get_mut::<EditableText>() {
            textedit.editor.set_text(&self.text);
        }
    }
}
