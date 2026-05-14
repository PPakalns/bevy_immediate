use std::marker::PhantomData;

use bevy::color::palettes::css::NAVY;
use bevy::ecs::system::EntityCommand;
use bevy::ecs::{
    change_detection::DetectChanges,
    component::Component,
    system::{Local, SystemParam},
};
use bevy::text::EditableText;
use bevy::ui::{BackgroundColor, FlexDirection, Node, px};
use bevy::utils::default;
use bevy_immediate::{
    CapSet, Imm, ImmCapAccessRequests, ImmCapability, ImmEntity, ImplCap,
    attach::{BevyImmediateAttachPlugin, ImmediateAttach},
    imm_id,
    ui::text::ImmUiText,
    utils::ImmLocalHashMemoryHelper,
};

use crate::extension_use::CapsMyUi;

pub struct TextEditExamplePlugin;

impl bevy::app::Plugin for TextEditExamplePlugin {
    fn build(&self, app: &mut bevy::app::App) {
        app.add_plugins(BevyImmediateAttachPlugin::<CapsMyUi, TextEditExampleRoot>::new());
    }
}

#[derive(Component)]
pub struct TextEditExampleRoot;

#[derive(SystemParam)]
pub struct Params<'w, 's> {
    text: Local<'s, String>,
    _ph: PhantomData<(&'w (), &'s ())>,
}

impl ImmediateAttach<CapsMyUi> for TextEditExampleRoot {
    type Params = Params<'static, 'static>;

    fn construct(ui: &mut Imm<CapsMyUi>, params: &mut Params) {
        ui.ch()
            .on_spawn_insert(|| Node {
                flex_direction: FlexDirection::Column,
                row_gap: px(4.),
                ..default()
            })
            .add(|ui| {
                for idx in 0..10 {
                    ui.ch_id(("text", idx))
                        .on_spawn_insert(|| {
                            (
                                Node {
                                    width: px(500.),
                                    height: px(30.),
                                    ..default()
                                },
                                BackgroundColor(NAVY.into()),
                                EditableText {
                                    max_characters: Some(100),
                                    ..default()
                                },
                            )
                        })
                        .input_text(&mut params.text);
                }

                ui.ch().text(params.text.as_str());
            });
    }
}

/// Capability that allows to write, read ui input text in immediate mode
pub struct CapabilityUiTextInput;

impl ImmCapability for CapabilityUiTextInput {
    fn build<Cap: CapSet>(app: &mut bevy::app::App, cap_req: &mut ImmCapAccessRequests<Cap>) {
        cap_req.request_component_write::<EditableText>(app.world_mut());
    }
}

pub trait ImmUiTextInput {
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
                    input_buffer.editor.set_text(text);

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

    fn apply(self, mut entity: bevy::ecs::world::EntityWorldMut) -> Self::Out {
        if let Some(mut textedit) = entity.get_mut::<EditableText>() {
            textedit.editor.set_text(&self.text);
        }
    }
}
