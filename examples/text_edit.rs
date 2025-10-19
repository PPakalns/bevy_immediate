use std::{iter::Peekable, marker::PhantomData};

use bevy::utils::default;
use bevy_color::palettes::css::NAVY;
use bevy_ecs::{
    change_detection::{DetectChanges, DetectChangesMut},
    component::Component,
    system::{Local, SystemParam},
};
use bevy_immediate::{
    CapSet, Imm, ImmCapAccessRequests, ImmCapability, ImmEntity, ImplCap,
    attach::{BevyImmediateAttachPlugin, ImmediateAttach},
    imm_id,
    ui::text::ImmUiText,
    utils::ImmLocalHashMemoryHelper,
};
use bevy_ui::{BackgroundColor, FlexDirection, Node, px};
use bevy_ui_text_input::{
    TextInputBuffer, TextInputMode, TextInputNode, TextInputPlugin, TextInputPrompt,
};
use cosmic_text::{Buffer, BufferLine, Cursor, Edit};

use crate::extension_use::CapsMyUi;

pub struct TextEditExamplePlugin;

impl bevy_app::Plugin for TextEditExamplePlugin {
    fn build(&self, app: &mut bevy_app::App) {
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
                                TextInputNode {
                                    mode: TextInputMode::SingleLine,
                                    max_chars: Some(100),
                                    clear_on_submit: false,
                                    ..Default::default()
                                },
                                TextInputPrompt::default(),
                                TextInputBuffer::default(),
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
    fn build<Cap: CapSet>(app: &mut bevy_app::App, cap_req: &mut ImmCapAccessRequests<Cap>) {
        // See bevy_immediate dev-dependencies for bevy_ui_text_input version
        // that works correctly in newest bevy
        if !app.is_plugin_added::<TextInputPlugin>() {
            app.add_plugins(TextInputPlugin);
        }

        cap_req.request_component_write::<TextInputBuffer>(app.world_mut());
        cap_req.request_component_write::<TextInputNode>(app.world_mut());
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

            let Some(mut input_buffer) = entity.get_mut::<TextInputBuffer>() else {
                break 'exist;
            };

            if !helper.is_stored(&text_hash) {
                helper.store(&text_hash);

                // External value updated
                input_buffer.editor.with_buffer_mut(|buffer| {
                    buffer.lines.clear();
                });
                input_buffer.editor.insert_at(Cursor::new(0, 0), text, None);

                if let Some(mut text_input_node) = entity.get_mut::<TextInputNode>() {
                    text_input_node.set_changed(); // Trigger text formatting
                }
            } else {
                let not_equal = input_buffer.is_changed() // Fast check
                    && !input_buffer.editor.with_buffer(|buffer| { // Slow check
                        let mut remaining: &str = &text;
                        for part in BufferTextIterator::new(buffer) {
                            if !remaining.starts_with(part) {
                                return false;
                            }
                            remaining = &remaining[part.len()..];
                        }

                        return remaining.is_empty();
                    });

                if not_equal {
                    input_buffer.editor.with_buffer(|buffer| {
                        *text = BufferTextIterator::new(buffer).collect();
                    });
                    helper.store(&imm_id(&text));
                }
            }

            helper.finalize(&mut self);
            return self;
        }

        let mut input_buffer = TextInputBuffer::default();
        let editor = &mut input_buffer.editor;
        editor.insert_string(text, None);

        self.entity_commands().insert(input_buffer);
        helper.finalize(&mut self);

        self
    }
}

struct BufferTextIterator<'a> {
    lines: Peekable<std::slice::Iter<'a, BufferLine>>,
    newline: bool,
    insert_newline: bool,
}

impl<'a> BufferTextIterator<'a> {
    pub fn new(buffer: &'a Buffer) -> Self {
        Self {
            lines: buffer.lines.iter().peekable(),
            newline: false,
            insert_newline: false,
        }
    }
}

impl<'a> Iterator for BufferTextIterator<'a> {
    type Item = &'a str;

    fn next(&mut self) -> Option<Self::Item> {
        if self.lines.peek().is_none() {
            return None;
        }

        if self.insert_newline {
            self.insert_newline = false;
            return Some("\n");
        }

        let next = self.lines.next()?;
        if !self.newline && !next.text().is_empty() {
            self.newline = true;
        }

        if self.newline {
            self.insert_newline = true;
        }

        Some(next.text())
    }
}
