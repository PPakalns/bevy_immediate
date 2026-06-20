use bevy_ecs::{change_detection::DetectChanges, system::EntityCommand};
use bevy_text::EditableText;

use bevy_immediate_core::{
    CapSet, ImmCapAccessRequests, ImmCapability, ImmEntity, ImplCap, imm_id,
    utils::ImmLocalHashMemoryHelper,
};

/// Capability that allows reading and writing UI input text in immediate mode.
pub struct CapabilityUiTextInput;

impl ImmCapability for CapabilityUiTextInput {
    fn build<Cap: CapSet>(app: &mut bevy_app::App, cap_req: &mut ImmCapAccessRequests<Cap>) {
        cap_req.request_component_write::<EditableText>(app.world_mut());
    }
}

/// Implements methods to synchronise [`EditableText`] in immediate mode.
pub trait ImmUiTextInput {
    /// Synchronise text with an [`EditableText`] component on the entity.
    fn input_text(self, text: &mut String) -> Self;
}

impl<Cap> ImmUiTextInput for ImmEntity<'_, '_, '_, Cap>
where
    Cap: ImplCap<CapabilityUiTextInput>,
{
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
