use bevy::{math::Vec2, text::TextLayout, time::Time, utils::default};
use bevy_color::Color;
use bevy_ecs::{
    component::Component,
    hierarchy::Children,
    lifecycle,
    observer::On,
    query::{Changed, Or, With},
    spawn::{Spawn, SpawnRelated},
    system::{Commands, Query, Res, SystemParam},
};
use bevy_immediate::{
    CapSet, Imm, ImmEntity,
    attach::{BevyImmediateAttachPlugin, ImmediateAttach},
    ui::{CapsUiWidget, ImplCapsUiWidget, text::ImmUiText},
};
use bevy_input::{ButtonInput, keyboard::KeyCode};
use bevy_input_focus::{InputDispatchPlugin, tab_navigation::TabNavigationPlugin};
use bevy_picking::{
    events::{Drag, DragStart, Pointer, Scroll},
    hover::Hovered,
};
use bevy_ui::{
    BackgroundColor, BorderRadius, ComputedNode, Display, FlexDirection, FlexWrap, GridPlacement,
    Node, Overflow, OverflowAxis, PositionType, RepeatedGridTrack, ScrollPosition, UiRect, UiScale,
    px, vh, vw,
};
use bevy_ui_widgets::{
    ControlOrientation, CoreScrollbarDragState, CoreScrollbarThumb, Scrollbar, ScrollbarPlugin,
};

use crate::{bevy_scrollarea::colors::GRAY1, styles::title_text_style};

pub struct BevyScrollareaExamplePlugin;

impl bevy_app::Plugin for BevyScrollareaExamplePlugin {
    fn build(&self, app: &mut bevy_app::App) {
        // You will need bevy feature `experimental_bevy_ui_widgets`
        //
        // As these plugins are added by other examples
        // additional checks are done to avoid bevy error about already added plugins
        //

        // For scrollbar support
        if !app.is_plugin_added::<ScrollbarPlugin>() {
            app.add_plugins(ScrollbarPlugin);
        }

        // For keyboard support
        if !app.is_plugin_added::<InputDispatchPlugin>() {
            app.add_plugins(InputDispatchPlugin);
        }
        // For tab navigation support
        if !app.is_plugin_added::<TabNavigationPlugin>() {
            app.add_plugins(TabNavigationPlugin);
        }

        // Initialize plugin with your root component
        app.add_plugins(BevyImmediateAttachPlugin::<
            CapsUiWidget,
            BevyScrollareaExampleRoot,
        >::new());

        app.add_systems(bevy_app::Update, update_scrollbar_style_on_drag);

        app.add_observer(
            |event: On<lifecycle::Add, MyScrollableNode>, mut commands: Commands| {
                commands
                    .entity(event.event().entity)
                    .insert(ScrollState::default())
                    .observe(scroll_on_mouse)
                    .observe(scroll_on_drag_start)
                    .observe(scroll_on_drag);
            },
        );
    }
}

// See bevy example `bevy/examples/ui/scrollbars.rs`
// https://github.com/bevyengine/bevy/tree/main/examples
// for more information

#[derive(Component)]
pub struct BevyScrollareaExampleRoot;

#[derive(SystemParam)]
pub struct Params {}

impl ImmediateAttach<CapsUiWidget> for BevyScrollareaExampleRoot {
    type Params = Params;

    fn construct(ui: &mut Imm<CapsUiWidget>, _params: &mut Params) {
        ui.ch()
            .on_spawn_insert(title_text_style)
            .on_spawn_text("Bevy scrollareas");
        ui.ch().on_spawn_text("Powered by bevy_ui_widgets");

        ui.ch().on_spawn_insert(|| Node {
            height: px(20.),
            ..default()
        });
        ui.ch()
            .on_spawn_insert(|| TextLayout {
                linebreak: bevy::text::LineBreak::WordBoundary,
                ..default()
            })
            .on_spawn_text(
                "Example showcases implementation of reusable scrollarea (scrollable, draggable):",
            );

        ui.ch()
            .on_spawn_insert(|| Node {
                display: Display::Grid,
                flex_wrap: FlexWrap::Wrap,
                grid_template_columns: vec![RepeatedGridTrack::fr(2, 1.)],
                grid_template_rows: vec![RepeatedGridTrack::fr(2, 1.)],
                column_gap: px(20.),
                row_gap: px(20.),

                min_height: px(200.),
                min_width: px(200.),

                ..default()
            })
            .add(|ui| {
                for (idx, overflow) in [
                    Overflow {
                        x: OverflowAxis::Scroll,
                        y: OverflowAxis::Scroll,
                    },
                    Overflow {
                        x: OverflowAxis::Scroll,
                        y: OverflowAxis::Clip,
                    },
                    Overflow {
                        x: OverflowAxis::Clip,
                        y: OverflowAxis::Scroll,
                    },
                    Overflow {
                        x: OverflowAxis::Clip,
                        y: OverflowAxis::Clip,
                    },
                ]
                .into_iter()
                .enumerate()
                {
                    ui.ch_id(("overflow", idx)).scrollarea(
                        Node {
                            min_height: px(100),
                            min_width: px(100),
                            // max_height: vh(20.),
                            // max_width: vw(20.),
                            ..default()
                        },
                        Node {
                            display: Display::Flex,
                            flex_direction: FlexDirection::Column,
                            padding: UiRect::all(px(4)),
                            overflow,
                            ..default()
                        },
                        |ui_entity| {
                            // Function gives partially initialized content entity
                            // Let's add background
                            ui_entity
                                .on_spawn_insert(|| BackgroundColor(GRAY1.into()))
                                .add(|ui| {
                                    let no_wrap = || TextLayout {
                                        linebreak: bevy::text::LineBreak::NoWrap,
                                        ..default()
                                    };

                                    ui.ch().on_spawn_text_fn(|| format!("{:?}", overflow));

                                    // And fill content
                                    for idx in 0..30 {
                                        ui.ch().on_spawn_insert(no_wrap).on_spawn_text_fn(|| {
                                            format!("{idx} Lorem ipsum dolor sit amet, consectetur adipiscing elit, sed do eiusmod tempor incididunt ut labore et dolore magna aliqua.")
                                        });
                                    }
                                });
                        },
                    );
                }
            });
    }
}

pub trait ScrollBar<Caps: CapSet> {
    /// Content node must include overflow-axis logic
    ///
    /// Some node styles about how outer and content node will be placed will be overriden
    ///
    /// content node must set expected overflow axis
    fn scrollarea(
        self,
        outer_content_node: Node,
        content_node_style: Node,
        content: impl FnOnce(ImmEntity<'_, '_, '_, Caps>),
    ) -> Self;
}

impl<Caps> ScrollBar<Caps> for ImmEntity<'_, '_, '_, Caps>
where
    Caps: ImplCapsUiWidget,
{
    fn scrollarea(
        self,
        outer_content_node: Node,
        content_node_style: Node,
        content: impl FnOnce(ImmEntity<'_, '_, '_, Caps>),
    ) -> Self {
        let grid_template = |scrollbar: bool| {
            if scrollbar {
                vec![RepeatedGridTrack::flex(1, 1.), RepeatedGridTrack::auto(1)]
            } else {
                vec![RepeatedGridTrack::flex(1, 1.)]
            }
        };

        let horizontal = content_node_style.overflow.x == OverflowAxis::Scroll;
        let vertical = content_node_style.overflow.y == OverflowAxis::Scroll;

        self.on_spawn_insert(|| {
            Node {
                display: Display::Grid,
                grid_template_columns: grid_template(horizontal),
                grid_template_rows: grid_template(vertical),

                // Use all remaining values from user provided style
                ..outer_content_node
            }
        })
        .add(|ui| {
            let scrollarea_content = ui.ch().on_spawn_insert(|| {
                (
                    content_node_style,
                    ScrollPosition(Vec2::ZERO),
                    // Currently logic to scroll content
                    // by dragging content or by scrolling needs custom implementation
                    MyScrollableNode,
                )
            });

            // Store entity for scrollable content area
            let scrollbar_target = scrollarea_content.entity();

            // Finalize construction of scrollarea entity
            content(scrollarea_content);

            if vertical {
                // Vertical scrollbar
                ui.ch().on_spawn_insert(|| {
                    (
                        Node {
                            min_width: px(8),
                            grid_row: GridPlacement::start(1),
                            grid_column: GridPlacement::start(2),
                            ..default()
                        },
                        Scrollbar {
                            orientation: ControlOrientation::Vertical,
                            target: scrollbar_target,
                            min_thumb_length: 8.0,
                        },
                        Children::spawn(Spawn((
                            Node {
                                position_type: PositionType::Absolute,
                                ..default()
                            },
                            Hovered::default(),
                            BackgroundColor(colors::GRAY2.into()),
                            BorderRadius::all(px(4)),
                            CoreScrollbarThumb,
                        ))),
                    )
                });
            }

            if horizontal {
                // Horizontal scrollbar
                ui.ch().on_spawn_insert(|| {
                    (
                        Node {
                            min_height: px(8),
                            grid_row: GridPlacement::start(2),
                            grid_column: GridPlacement::start(1),
                            ..default()
                        },
                        Scrollbar {
                            orientation: ControlOrientation::Horizontal,
                            target: scrollbar_target,
                            min_thumb_length: 8.0,
                        },
                        Children::spawn(Spawn((
                            Node {
                                position_type: PositionType::Absolute,
                                ..default()
                            },
                            Hovered::default(),
                            BackgroundColor(colors::GRAY2.into()),
                            BorderRadius::all(px(4)),
                            CoreScrollbarThumb,
                        ))),
                    )
                });
            }
        })
    }
}

#[derive(Component)]
struct MyScrollableNode;

#[derive(Component, Default)]
struct ScrollState {
    initial_pos: Vec2,
}

fn scroll_on_mouse(
    scroll: On<Pointer<Scroll>>,
    ui_scale: Res<UiScale>,
    mut scroll_position_query: Query<(&mut ScrollPosition, &ComputedNode), With<MyScrollableNode>>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
) {
    if let Ok((mut scroll_position, node)) = scroll_position_query.get_mut(scroll.entity) {
        let visible_size = node.size() * node.inverse_scale_factor;
        let content_size = node.content_size() * node.inverse_scale_factor;
        let max_range = (content_size - visible_size).max(Vec2::ZERO);

        let mut delta = Vec2::new(scroll.x, scroll.y);

        match scroll.unit {
            bevy_input::mouse::MouseScrollUnit::Line => {
                delta *= 28.;
            }
            bevy_input::mouse::MouseScrollUnit::Pixel => {}
        }

        if keyboard_input.any_pressed([KeyCode::ControlLeft, KeyCode::ControlRight]) {
            std::mem::swap(&mut delta.x, &mut delta.y);
        }

        scroll_position.0 = (scroll_position.0 - (delta / ui_scale.0))
            .min(max_range)
            .max(Vec2::ZERO);
    }
}

fn scroll_on_drag_start(
    drag_start: On<Pointer<DragStart>>,
    mut scroll_position_query: Query<(&ComputedNode, &mut ScrollState), With<MyScrollableNode>>,
) {
    if let Ok((computed_node, mut state)) = scroll_position_query.get_mut(drag_start.entity) {
        state.initial_pos = computed_node.scroll_position;
    }
}

fn scroll_on_drag(
    drag: On<Pointer<Drag>>,
    ui_scale: Res<UiScale>,
    mut scroll_position_query: Query<
        (&mut ScrollPosition, &ComputedNode, &ScrollState),
        With<MyScrollableNode>,
    >,
) {
    if let Ok((mut scroll_position, comp_node, state)) = scroll_position_query.get_mut(drag.entity)
    {
        let visible_size = comp_node.size();
        let content_size = comp_node.content_size();
        let max_range = (content_size - visible_size).max(Vec2::ZERO);

        // Convert from screen coordinates to UI coordinates then back to physical coordinates
        let distance = drag.distance / (comp_node.inverse_scale_factor * ui_scale.0);

        scroll_position.0 = ((state.initial_pos - distance)
            .min(max_range)
            .max(Vec2::ZERO))
            * comp_node.inverse_scale_factor;
    }
}

// Update the color of the scrollbar thumb.
#[allow(clippy::type_complexity)]
fn update_scrollbar_style_on_drag(
    mut q_thumb: Query<
        (&mut BackgroundColor, &Hovered, &CoreScrollbarDragState),
        (
            With<CoreScrollbarThumb>,
            Or<(Changed<Hovered>, Changed<CoreScrollbarDragState>)>,
        ),
    >,
) {
    for (mut thumb_bg, Hovered(is_hovering), drag) in q_thumb.iter_mut() {
        let color: Color = if *is_hovering || drag.dragging {
            colors::GRAY3
        } else {
            colors::GRAY2
        }
        .into();

        if thumb_bg.0 != color {
            thumb_bg.0 = color;
        }
    }
}

mod colors {
    use bevy::color::Srgba;

    pub const GRAY1: Srgba = Srgba::new(0.224, 0.224, 0.243, 1.0);
    pub const GRAY2: Srgba = Srgba::new(0.486, 0.486, 0.529, 1.0);
    pub const GRAY3: Srgba = Srgba::new(1.0, 1.0, 1.0, 1.0);
}
