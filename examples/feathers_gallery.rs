use bevy::app::{App, Plugin, PluginGroup};
use bevy::color::palettes;
use bevy::color::{Alpha, Color, Hsla, Srgba};
use bevy::ecs::{
    component::Component,
    entity::Entity,
    hierarchy::{ChildOf, Children},
    observer::On,
    query::With,
    resource::Resource,
    system::{Commands, In, Query, Res, ResMut, SystemParam},
};
use bevy::feathers::{
    FeathersCorePlugin, FeathersPlugins,
    constants::{fonts, icons},
    containers::{
        flex_spacer, group, group_body, group_header, pane, pane_body, pane_header,
        pane_header_divider, subpane, subpane_body, subpane_header,
    },
    controls::{
        ButtonVariant, ColorChannel, FeathersButton, FeathersCheckbox, FeathersColorPlane,
        FeathersColorSlider, FeathersColorSwatch, FeathersDisclosureToggle, FeathersMenu,
        FeathersMenuButton, FeathersMenuDivider, FeathersMenuItem, FeathersMenuPopup,
        FeathersNumberInput, FeathersRadio, FeathersScrollbar, FeathersSlider, FeathersTextInput,
        FeathersTextInputContainer, FeathersToggleSwitch, FeathersToolButton,
    },
    cursor::{EntityCursor, OverrideCursor},
    dark_theme::create_dark_theme,
    display::{icon, label, label_dim, label_small},
    font_styles::InheritableFont,
    rounded_corners::RoundedCorners,
    theme::{ThemeBackgroundColor, ThemedText, UiTheme},
    tokens,
};
use bevy::input_focus::{AcquireFocus, FocusGained, InputFocus, tab_navigation::TabGroup};
use bevy::log::info;
use bevy::math::Vec3;
use bevy::scene::bsn;
use bevy::ui::widget::Text;
use bevy::ui::{AlignItems, FlexDirection, JustifyContent, Node, Overflow, UiRect, percent, px};
use bevy::ui_widgets::{
    ActivateOnPress, ControlOrientation, RadioGroup, ScrollArea, ScrollIntoView, SliderPrecision,
    SliderStep,
};
use bevy::utils::default;
use bevy::window::{PrimaryWindow, SystemCursorIcon};
use bevy_immediate::ImmEntity;
use bevy_immediate::{
    Imm,
    attach::{BevyImmediateAttachPlugin, ImmediateAttach},
    ui::{
        CapsUiFeathers, activated::ImmUiActivated, button_variant::ImmUiFeathersButtonVariant,
        checked::ImmUiChecked, color_plane::ImmUiColorPlane, color_swatch::ImmUiColorSwatch,
        disabled::ImmUiInteractionsDisabled, number_input::ImmUiNumberInput,
        slider_base_color::ImmUiSliderBaseColor, slider_value::ImmUiSliderValue,
        text_input::ImmUiTextInput,
    },
};
pub struct FeathersGalleryExamplePlugin;

const LIST_ITEMS: [&str; 12] = [
    "First World",
    "Second Nature",
    "Third Degree",
    "Fourth Wall",
    "Fifth Column",
    "Sixth Sense",
    "Seventh Heaven",
    "Eighth Wonder",
    "Ninth Inning",
    "Tenth Amendment",
    "Eleventh Hour",
    "Twelfth Night",
];

impl Plugin for FeathersGalleryExamplePlugin {
    fn build(&self, app: &mut App) {
        let rgb_color = palettes::tailwind::EMERALD_800.with_alpha(0.7);
        app.insert_resource(GalleryState {
            rgb_color,
            hsl_color: palettes::tailwind::AMBER_800.into(),
            scalar_prop: 7.0,
            vec3_prop: Vec3::new(10.1, 7.124, 100.0),
            slider_value: 20.0,
            list_selected: 1,
            demo_button_disabled: true,
            fast_click_checkbox: true,
            fast_click_toggle: true,
            radio: RadioChoice::default(),
            toggle: false,
            disclosure: false,
        });

        if !app.is_plugin_added::<FeathersCorePlugin>() {
            app.add_plugins(FeathersPlugins.build())
                .insert_resource(UiTheme(create_dark_theme()));
        }

        app.add_plugins(BevyImmediateAttachPlugin::<
            CapsUiFeathers,
            FeathersGalleryExampleRoot,
        >::new());

        app.add_observer(scroll_focused_item_into_view);
    }
}

// Bevy immediate uses manual implementation for listbox, we want to scroll item into view on focus
fn scroll_focused_item_into_view(
    _focus: On<FocusGained>,
    input_focus: Res<InputFocus>,
    scroll_areas: Query<(), With<ScrollArea>>,
    parents: Query<&ChildOf>,
    mut commands: Commands,
) {
    let Some(focused) = input_focus.get() else {
        return;
    };

    let inside_scroll_area = scroll_areas.contains(focused)
        || parents
            .iter_ancestors(focused)
            .any(|ancestor| scroll_areas.contains(ancestor));

    if inside_scroll_area {
        commands.trigger(ScrollIntoView { entity: focused });
    }
}

fn acquire_list_row_focus(
    In(row_entity): In<Entity>,
    windows: Query<Entity, With<PrimaryWindow>>,
    mut commands: Commands,
) {
    if let Ok(window) = windows.single() {
        commands.trigger(AcquireFocus {
            focused_entity: row_entity,
            window,
        });
    }
}

#[derive(Component)]
pub struct FeathersGalleryExampleRoot;

/// Widget state shared across the gallery.
#[derive(Resource)]
struct GalleryState {
    pub rgb_color: Srgba,
    pub hsl_color: Hsla,
    pub scalar_prop: f32,
    pub vec3_prop: Vec3,
    pub slider_value: f32,
    pub demo_button_disabled: bool,
    pub fast_click_checkbox: bool,
    pub radio: RadioChoice,
    pub toggle: bool,
    pub fast_click_toggle: bool,
    pub disclosure: bool,
    pub list_selected: usize,
}

#[derive(Default, Clone, Copy, PartialEq, Eq)]
pub(crate) enum RadioChoice {
    #[default]
    One,
    Two,
    FastClick,
}

#[derive(SystemParam)]
pub struct Params<'w> {
    state: ResMut<'w, GalleryState>,
    override_cursor: ResMut<'w, OverrideCursor>,
}

impl ImmediateAttach<CapsUiFeathers> for FeathersGalleryExampleRoot {
    type Params = Params<'static>;

    fn construct(ui: &mut Imm<CapsUiFeathers>, params: &mut Params) {
        ui.ch()
            .on_spawn_insert(|| {
                (
                    Node {
                        width: percent(100),
                        height: percent(100),
                        align_items: AlignItems::Start,
                        justify_content: JustifyContent::Stretch,
                        flex_direction: FlexDirection::Row,
                        column_gap: px(8),
                        ..default()
                    },
                    TabGroup::default(),
                    ThemeBackgroundColor(tokens::WINDOW_BG),
                )
            })
            .add(|ui| {
                ui.ch()
                    .on_spawn_insert(column_node)
                    .add(|ui| demo_column_1(ui, params));

                ui.ch()
                    .on_spawn_insert(column_node)
                    .add(|ui| demo_column_2(ui, params));
            });
    }
}

fn column_node() -> Node {
    Node {
        display: bevy::ui::Display::Flex,
        flex_direction: FlexDirection::Column,
        align_items: AlignItems::Stretch,
        justify_content: JustifyContent::Start,
        flex_grow: 1.,
        padding: px(8).into(),
        row_gap: px(8),
        // width: percent(30),
        min_width: px(200),
        ..default()
    }
}

fn demo_column_1(ui: &mut Imm<CapsUiFeathers>, params: &mut Params) {
    let Params {
        state,
        override_cursor,
        ..
    } = params;

    ui.ch().on_spawn_insert(row_node).add(|ui| {
        if ui
            .ch()
            .on_spawn_apply_scene(|| {
                bsn! {
                    @FeathersButton {
                        @caption: bsn! { Text("Normal") ThemedText }
                    }
                    Node {
                        flex_grow: 1.0,
                    }
                }
            })
            .activated()
        {
            info!("Normal button clicked!");
        }

        if ui
            .ch()
            .on_spawn_apply_scene(|| {
                bsn! {
                    @FeathersButton {
                        @caption: bsn! { Text("Disabled") ThemedText }
                    }
                    Node {
                        flex_grow: 1.0,
                    }
                }
            })
            .interactions_disabled(state.demo_button_disabled)
            .activated()
        {
            info!("Disabled button clicked!");
        }

        if ui
            .ch()
            .on_spawn_apply_scene(|| {
                bsn! {
                    @FeathersButton {
                        @caption: bsn! { Text("Primary") ThemedText },
                        @variant: ButtonVariant::Primary,
                    }
                    Node {
                        flex_grow: 1.0,
                    }
                }
            })
            .activated()
        {
            info!("Primary button clicked!");
        }

        ui.ch()
            .on_spawn_apply_scene(|| bsn! { @FeathersMenu })
            .add(|ui| {
                ui.ch().on_spawn_queue_apply_scene(|| {
                    bsn! {
                        @FeathersMenuButton {
                            @caption: bsn! { Text("Menu") ThemedText }
                        }
                        Node {
                            flex_grow: 1.0,
                        }
                    }
                });

                ui.ch()
                    .on_spawn_apply_scene(|| bsn! { @FeathersMenuPopup })
                    .add(|ui| {
                        for (index, label) in ["MenuItem 1", "MenuItem 2", "MenuItem 3"]
                            .into_iter()
                            .enumerate()
                        {
                            if index == 2 {
                                ui.ch_id((index, "divider"))
                                    .on_spawn_apply_scene(|| bsn! { @FeathersMenuDivider });
                            }

                            if ui
                                .ch_id(index)
                                .on_spawn_apply_scene(|| {
                                    bsn! {
                                        @FeathersMenuItem {
                                            @caption: bsn! { Text(label) ThemedText }
                                        }
                                    }
                                })
                                .activated()
                            {
                                info!("Menu item {} clicked!", index + 1);
                            }
                        }
                    });
            });
    });

    ui.ch().on_spawn_insert(row_tight_node).add(|ui| {
        if ui
            .ch()
            .on_spawn_apply_scene(|| {
                bsn! {
                    @FeathersButton {
                        @caption: bsn! { Text("Left") ThemedText },
                        @corners: RoundedCorners::Left,
                    }
                    Node {
                        flex_grow: 1.0,
                    }
                }
            })
            .activated()
        {
            info!("Left button clicked!");
        }

        if ui
            .ch()
            .on_spawn_apply_scene(|| {
                bsn! {
                    @FeathersButton {
                        @caption: bsn! { Text("Center") ThemedText },
                        @corners: RoundedCorners::None,
                    }
                    Node {
                        flex_grow: 1.0,
                    }
                }
            })
            .activated()
        {
            info!("Center button clicked!");
        }

        if ui
            .ch()
            .on_spawn_apply_scene(|| {
                bsn! {
                    @FeathersButton {
                        @caption: bsn! { Text("Right") ThemedText },
                        @variant: ButtonVariant::Primary,
                        @corners: RoundedCorners::Right,
                    }
                    Node {
                        flex_grow: 1.0,
                    }
                }
            })
            .activated()
        {
            info!("Right button clicked!");
        }
    });

    if ui
        .ch()
        .on_spawn_apply_scene(|| {
            bsn! {
                @FeathersButton
                Children [ (Text("Toggle override") ThemedText) ]
            }
        })
        .activated()
    {
        override_cursor.0 = if override_cursor.0.is_some() {
            None
        } else {
            Some(EntityCursor::System(SystemCursorIcon::Wait))
        };
        info!("Override cursor button clicked!");
    }

    ui.ch_id("checkbox")
        .on_spawn_apply_scene(|| {
            bsn! {
                @FeathersCheckbox {
                    @caption: bsn! { Text("Checkbox") ThemedText }
                }
            }
        })
        .checked(&mut state.demo_button_disabled);

    ui.ch_id("fast_click_checkbox")
        .on_spawn_apply_scene(|| {
            bsn! {
                @FeathersCheckbox {
                    @caption: bsn! { Text("Fast Click Checkbox") ThemedText }
                }
                ActivateOnPress
            }
        })
        .checked(&mut state.fast_click_checkbox);

    ui.ch()
        .on_spawn_apply_scene(|| {
            bsn! {
                @FeathersCheckbox {
                    @caption: bsn! { Text("Disabled") ThemedText }
                }
            }
        })
        .interactions_disabled(true);

    ui.ch()
        .on_spawn_apply_scene(|| {
            bsn! {
                @FeathersCheckbox {
                    @caption: bsn! { Text("Checked+Disabled") ThemedText }
                }
            }
        })
        .interactions_disabled(true)
        .checked_set(true);

    ui.ch().on_spawn_insert(row_node).add(|ui| {
        ui.ch()
            .on_spawn_insert(|| {
                (
                    Node {
                        display: bevy::ui::Display::Flex,
                        flex_direction: FlexDirection::Column,
                        row_gap: px(4),
                        ..default()
                    },
                    RadioGroup,
                )
            })
            .add(|ui| {
                ui.ch_id("radio_one")
                    .on_spawn_apply_scene(|| {
                        bsn! {
                            @FeathersRadio {
                                @caption: bsn! { Text("One") ThemedText }
                            }
                        }
                    })
                    .checked_if_eq(RadioChoice::One, &mut state.radio);

                ui.ch_id("radio_two")
                    .on_spawn_apply_scene(|| {
                        bsn! {
                            @FeathersRadio {
                                @caption: bsn! { Text("Two") ThemedText }
                            }
                        }
                    })
                    .checked_if_eq(RadioChoice::Two, &mut state.radio);

                ui.ch_id("radio_fast_click")
                    .on_spawn_apply_scene(|| {
                        bsn! {
                            @FeathersRadio {
                                @caption: bsn! { Text("Fast Click") ThemedText }
                            }
                            ActivateOnPress
                        }
                    })
                    .checked_if_eq(RadioChoice::FastClick, &mut state.radio);

                ui.ch()
                    .on_spawn_apply_scene(|| {
                        bsn! {
                            @FeathersRadio {
                                @caption: bsn! { Text("Disabled") ThemedText }
                            }
                        }
                    })
                    .interactions_disabled(true);
            });
    });

    ui.ch().on_spawn_insert(row_node).add(|ui| {
        ui.ch()
            .on_spawn_apply_scene(|| bsn! { @FeathersToggleSwitch })
            .checked(&mut state.toggle);

        ui.ch()
            .on_spawn_apply_scene(|| {
                bsn! {
                    @FeathersToggleSwitch
                    ActivateOnPress
                }
            })
            .checked(&mut state.fast_click_toggle);

        ui.ch()
            .on_spawn_apply_scene(|| bsn! { @FeathersToggleSwitch })
            .interactions_disabled(true);

        ui.ch()
            .on_spawn_apply_scene(|| bsn! { @FeathersToggleSwitch })
            .interactions_disabled(true)
            .checked_set(true);

        ui.ch()
            .on_spawn_apply_scene(|| bsn! { @FeathersDisclosureToggle })
            .checked(&mut state.disclosure);
    });

    ui.ch()
        .on_spawn_apply_scene(|| {
            bsn! {
                @FeathersSlider {
                    @max: 100.0,
                    @value: 20.0,
                }
                SliderStep(10.)
                SliderPrecision(2)
            }
        })
        .slider(&mut state.slider_value);

    ui.ch().on_spawn_insert(row_between_node).add(|ui| {
        ui.ch().on_spawn_apply_scene(|| label("Srgba"));

        ui.ch().on_spawn_insert(|| Node {
            flex_grow: 1.0,
            ..default()
        });

        ui.ch()
            .on_spawn_apply_scene(|| {
                bsn! {
                    @FeathersTextInputContainer
                    Node {
                        flex_grow: 0.,
                        padding: { px(4).left() },
                    }
                }
            })
            .add(|ui| {
                ui.ch()
                    .on_spawn_apply_scene(|| {
                        bsn! {
                            @FeathersTextInput {
                                @visible_width: 10f32,
                                @max_characters: 9usize,
                            }
                            InheritableFont {
                                font: fonts::MONO,
                            }
                        }
                    })
                    .input_get_set(|value| match value {
                        None => state.rgb_color.to_hex(),
                        Some(hex) => {
                            if let Ok(color) = Srgba::hex(hex) {
                                state.rgb_color = color;
                            }
                            state.rgb_color.to_hex()
                        }
                    });
            });

        ui.ch()
            .on_spawn_apply_scene(|| bsn! { @FeathersColorSwatch })
            .color_swatch(state.rgb_color.into());
    });

    ui.ch()
        .on_spawn_apply_scene(|| bsn! { @FeathersColorPlane::RedBlue })
        .color_plane_get_set(|value| match value {
            None => Vec3::new(
                state.rgb_color.red,
                state.rgb_color.blue,
                state.rgb_color.green,
            ),
            Some(xy) => {
                state.rgb_color.red = xy.x;
                state.rgb_color.blue = xy.y;
                Vec3::new(
                    state.rgb_color.red,
                    state.rgb_color.blue,
                    state.rgb_color.green,
                )
            }
        });

    color_slider(
        ui.ch(),
        ColorChannel::Red,
        Color::from(state.rgb_color),
        |value| match value {
            None => state.rgb_color.red,
            Some(red) => {
                state.rgb_color.red = red;
                red
            }
        },
    );
    color_slider(
        ui.ch(),
        ColorChannel::Green,
        Color::from(state.rgb_color),
        |value| match value {
            None => state.rgb_color.green,
            Some(green) => {
                state.rgb_color.green = green;
                green
            }
        },
    );
    color_slider(
        ui.ch(),
        ColorChannel::Blue,
        Color::from(state.rgb_color),
        |value| match value {
            None => state.rgb_color.blue,
            Some(blue) => {
                state.rgb_color.blue = blue;
                blue
            }
        },
    );
    color_slider(
        ui.ch(),
        ColorChannel::Alpha,
        Color::from(state.rgb_color),
        |value| match value {
            None => state.rgb_color.alpha,
            Some(alpha) => {
                state.rgb_color.alpha = alpha;
                alpha
            }
        },
    );

    ui.ch().on_spawn_insert(row_between_node).add(|ui| {
        ui.ch().on_spawn_apply_scene(|| label("Hsl"));
        ui.ch()
            .on_spawn_apply_scene(|| bsn! { @FeathersColorSwatch })
            .color_swatch(state.hsl_color.into());
    });

    color_slider(
        ui.ch(),
        ColorChannel::HslHue,
        Color::from(state.hsl_color),
        |value| match value {
            None => state.hsl_color.hue,
            Some(hue) => {
                state.hsl_color.hue = hue;
                hue
            }
        },
    );
    color_slider(
        ui.ch(),
        ColorChannel::HslSaturation,
        Color::from(state.hsl_color),
        |value| match value {
            None => state.hsl_color.saturation,
            Some(saturation) => {
                state.hsl_color.saturation = saturation;
                saturation
            }
        },
    );
    color_slider(
        ui.ch(),
        ColorChannel::HslLightness,
        Color::from(state.hsl_color),
        |value| match value {
            None => state.hsl_color.lightness,
            Some(lightness) => {
                state.hsl_color.lightness = lightness;
                lightness
            }
        },
    );

    ui.ch().on_spawn_queue_apply_scene(|| {
        label_dim("Example is based on bevy feathers_gallery example!");
    });
}

fn demo_column_2(ui: &mut Imm<CapsUiFeathers>, params: &mut Params) {
    let state = &mut *params.state;

    ui.ch().on_spawn_apply_scene(pane).add(|ui| {
        ui.ch().on_spawn_apply_scene(pane_header).add(|ui| {
            ui.ch().on_spawn_apply_scene(|| {
                bsn! {
                    @FeathersToolButton {
                        @variant: ButtonVariant::Primary,
                    }
                    Children [(Text("\u{0398}") ThemedText)]
                }
            });

            ui.ch().on_spawn_apply_scene(pane_header_divider);

            for (id, text) in [
                ("tool_quarter", "\u{00BC}"),
                ("tool_half", "\u{00BD}"),
                ("tool_three_quarter", "\u{00BE}"),
            ] {
                ui.ch_id(id).on_spawn_apply_scene(|| {
                    bsn! {
                        @FeathersToolButton {
                            @variant: ButtonVariant::Plain,
                        }
                        Children [(Text(text) ThemedText)]
                    }
                });
            }
            ui.ch().on_spawn_apply_scene(pane_header_divider);
            ui.ch().on_spawn_queue_apply_scene(|| {
                bsn! {
                    @FeathersToolButton {
                        @variant: ButtonVariant::Plain,
                    }
                    Children [icon(icons::CHEVRON_DOWN)]
                }
            });
            ui.ch().on_spawn_apply_scene(flex_spacer);
            ui.ch().on_spawn_queue_apply_scene(|| {
                bsn! {
                    @FeathersToolButton {
                        @variant: ButtonVariant::Plain,
                    }
                    Children [icon(icons::X)]
                }
            });
        });

        ui.ch().on_spawn_apply_scene(pane_body).add(|ui| {
            ui.ch()
                .on_spawn_apply_scene(|| label_dim("A standard editor pane"));

            ui.ch().on_spawn_apply_scene(subpane).add(|ui| {
                ui.ch().on_spawn_apply_scene(subpane_header).add(|ui| {
                    for (id, text) in [("left", "Left"), ("center", "Center"), ("right", "Right")] {
                        ui.ch_id(id).on_spawn_apply_scene(|| {
                            bsn! {
                                Text(text)
                                ThemedText
                            }
                        });
                    }
                });

                ui.ch().on_spawn_apply_scene(subpane_body).add(|ui| {
                    ui.ch()
                        .on_spawn_apply_scene(|| label_dim("A standard sub-pane"));

                    ui.ch().on_spawn_apply_scene(group).add(|ui| {
                        ui.ch().on_spawn_apply_scene(group_header).add(|ui| {
                            ui.ch().on_spawn_apply_scene(|| {
                                bsn! {
                                    Text("Group")
                                    ThemedText
                                }
                            });
                        });

                        ui.ch().on_spawn_apply_scene(group_body).add(|ui| {
                            ui.ch().on_spawn_apply_scene(|| label("A standard group"));

                            ui.ch()
                                .on_spawn_apply_scene(|| label_small("Scalar property"));
                            ui.ch()
                                .on_spawn_apply_scene(|| {
                                    bsn! {
                                        @FeathersNumberInput
                                        Node {
                                            flex_grow: 1.0,
                                            max_width: px(100),
                                        }
                                    }
                                })
                                .number_input(&mut state.scalar_prop);

                            ui.ch()
                                .on_spawn_apply_scene(|| label_small("Scalar property (copy)"));
                            ui.ch()
                                .on_spawn_apply_scene(|| {
                                    bsn! {
                                        @FeathersNumberInput
                                        Node {
                                            flex_grow: 1.0,
                                            max_width: px(100),
                                        }
                                    }
                                })
                                .number_input(&mut state.scalar_prop);

                            ui.ch()
                                .on_spawn_apply_scene(|| label_small("Vec3 property"));
                            ui.ch().on_spawn_insert(vec3_row_node).add(|ui| {
                                vec3_number_input(
                                    ui.ch(),
                                    &mut state.vec3_prop.x,
                                    tokens::TEXT_INPUT_X_AXIS,
                                    "X",
                                );
                                vec3_number_input(
                                    ui.ch(),
                                    &mut state.vec3_prop.y,
                                    tokens::TEXT_INPUT_Y_AXIS,
                                    "Y",
                                );
                                vec3_number_input(
                                    ui.ch(),
                                    &mut state.vec3_prop.z,
                                    tokens::TEXT_INPUT_Z_AXIS,
                                    "Z",
                                );
                            });
                        });
                    });
                });
            });

            ui.ch().on_spawn_apply_scene(subpane).add(|ui| {
                ui.ch().on_spawn_apply_scene(subpane_header).add(|ui| {
                    ui.ch().on_spawn_apply_scene(|| {
                        bsn! {
                            Text("List")
                            ThemedText
                        }
                    });
                });

                ui.ch()
                    .on_spawn_apply_scene(subpane_body)
                    .add(|ui| demo_list_view(ui, state));
            });
        });
    });
}

fn demo_list_view(ui: &mut Imm<CapsUiFeathers>, state: &mut GalleryState) {
    // WARN: Bevy Feathers does not trigger events on the list items, instead
    // it triggers them on ListBox.
    //
    // Instead we create our own list view widget (which can be improved even further yourself)

    ui.ch()
        .on_spawn_insert(|| Node {
            display: bevy::ui::Display::Flex,
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Stretch,
            justify_content: JustifyContent::Start,
            max_height: px(130),
            overflow: Overflow::clip_y(),
            padding: UiRect {
                right: px(10),
                ..default()
            },
            ..default()
        })
        .add(|ui| {
            let scroll_container = ui.ch().on_spawn_insert(|| {
                (
                    Node {
                        display: bevy::ui::Display::Flex,
                        flex_direction: FlexDirection::Column,
                        align_items: AlignItems::Stretch,
                        justify_content: JustifyContent::Start,
                        flex_grow: 1.0,
                        min_height: px(0),
                        overflow: Overflow::scroll_y(),
                        ..default()
                    },
                    ScrollArea,
                    TabGroup::default(),
                )
            });
            let scroll_target = scroll_container.entity();

            scroll_container.add(|ui| {
                for (index, &label) in LIST_ITEMS.iter().enumerate() {
                    let disabled = index == 3;
                    let selected = index == state.list_selected;

                    let mut row = ui
                        .ch_id(("list_row", index))
                        .on_spawn_apply_scene(move || {
                            bsn! {
                                @FeathersButton {
                                    @variant: ButtonVariant::Plain,
                                    @caption: bsn! { Text(label) ThemedText },
                                }
                                Node {
                                    justify_content: JustifyContent::Start,
                                    padding: px(4),
                                }
                            }
                        })
                        .interactions_disabled(disabled)
                        .button_variant(if selected {
                            ButtonVariant::Normal
                        } else {
                            ButtonVariant::Plain
                        });

                    let activated = !disabled && row.activated();
                    let row_entity = row.entity();

                    if activated {
                        state.list_selected = index;
                        ui.commands_mut()
                            .run_system_cached_with(acquire_list_row_focus, row_entity);
                    }
                }
            });

            ui.ch().on_spawn_queue_apply_scene(move || {
                bsn! {
                    @FeathersScrollbar {
                        @target: {scroll_target},
                        @orientation: {ControlOrientation::Vertical},
                    }
                }
            });
        });
}

fn vec3_number_input(
    ui: ImmEntity<CapsUiFeathers>,
    value: &mut f32,
    sigil_color: bevy::feathers::theme::ThemeToken,
    label_text: &'static str,
) {
    ui.on_spawn_apply_scene(move || {
        bsn! {
            @FeathersNumberInput {
                @sigil_color: sigil_color,
                @label_text: label_text,
            }
            Node {
                flex_grow: 1.0,
            }
        }
    })
    .number_input(value);
}

fn vec3_row_node() -> Node {
    Node {
        display: bevy::ui::Display::Flex,
        flex_direction: FlexDirection::Row,
        column_gap: px(6),
        align_items: AlignItems::Center,
        justify_content: JustifyContent::SpaceBetween,
        ..default()
    }
}

fn row_node() -> Node {
    Node {
        display: bevy::ui::Display::Flex,
        flex_direction: FlexDirection::Row,
        align_items: AlignItems::Center,
        justify_content: JustifyContent::Start,
        column_gap: px(8),
        ..default()
    }
}

fn row_tight_node() -> Node {
    Node {
        display: bevy::ui::Display::Flex,
        flex_direction: FlexDirection::Row,
        align_items: AlignItems::Center,
        justify_content: JustifyContent::Start,
        column_gap: px(1),
        ..default()
    }
}

fn row_between_node() -> Node {
    Node {
        display: bevy::ui::Display::Flex,
        flex_direction: FlexDirection::Row,
        align_items: AlignItems::Center,
        justify_content: JustifyContent::SpaceBetween,
        column_gap: px(4),
        ..default()
    }
}

fn color_slider(
    ui: ImmEntity<CapsUiFeathers>,
    channel: ColorChannel,
    base_color: Color,
    value_get_set: impl FnMut(Option<f32>) -> f32,
) {
    ui.on_spawn_apply_scene(move || {
        bsn! {
            @FeathersColorSlider {
                @channel: channel,
            }
        }
    })
    .slider_base_color(base_color)
    .slider_get_set(value_get_set);
}
