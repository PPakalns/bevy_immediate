use bevy::color::palettes::tailwind::{GRAY_100, GRAY_300, GRAY_500, GRAY_700};
use bevy::ecs::{
    component::Component,
    system::{Local, SystemParam},
};
use bevy::text::{EditableText, TextCursorStyle, TextLayout};
use bevy::ui::{BackgroundColor, FlexDirection, Node, px};
use bevy::utils::default;
use bevy_immediate::{
    Imm,
    attach::{BevyImmediateAttachPlugin, ImmediateAttach},
    ui::{text::ImmUiText, text_input::ImmUiTextInput},
};
use std::marker::PhantomData;

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
                                BackgroundColor(GRAY_700.into()),
                                EditableText {
                                    max_characters: Some(100),
                                    ..default()
                                },
                                TextCursorStyle {
                                    color: GRAY_100.into(),
                                    selection_color: GRAY_300.into(),
                                    unfocused_selection_color: GRAY_500.into(),
                                    selected_text_color: None,
                                },
                                TextLayout::no_wrap(),
                            )
                        })
                        .input_text(&mut params.text);
                }

                ui.ch().text(params.text.as_str());
            });
    }
}
