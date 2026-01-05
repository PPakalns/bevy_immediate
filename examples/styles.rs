use bevy::input_focus::{InputFocus, InputFocusVisible};
use bevy::picking::hover::Hovered;
use bevy::ui::Pressed;
use bevy::{
    color::{Color, palettes::basic::*},
    prelude::*,
};
use bevy_immediate::ui::{floating_window_plugin::WindowResizeDragDirection, selected::Selectable};

pub struct DemoStylePlugin;

impl bevy::app::Plugin for DemoStylePlugin {
    fn build(&self, app: &mut bevy::app::App) {
        app.add_systems(
            Update,
            (button_system, focus_system, resizable_style_system),
        );
    }
}

// Component used to apply button style
// to entities with this component
#[derive(Component)]
pub struct MyStyle;

#[allow(clippy::type_complexity)]
fn button_system(
    mut interaction_query: Query<
        (
            &Hovered,
            &mut BackgroundColor,
            &mut BorderColor,
            Option<&Selectable>,
            Has<Pressed>,
        ),
        (With<Button>, With<MyStyle>),
    >,
    changed: Query<
        Entity,
        (
            Or<(Changed<Hovered>, Changed<Selectable>, Changed<Pressed>)>,
            (With<Button>, With<MyStyle>),
        ),
    >,
    mut removed_pressed: RemovedComponents<Pressed>,
) {
    // Set interactable element
    // background and border colors
    // when inactive, pressed, hovered and when selected

    for entity in changed.iter().chain(removed_pressed.read()) {
        let Ok((hovered, mut color, mut border_color, selected, pressed)) =
            interaction_query.get_mut(entity)
        else {
            continue;
        };

        if pressed {
            color.0 = PRESSED_BUTTON;
            border_color.set_all(RED);
        } else if hovered.get() {
            color.0 = HOVERED_BUTTON;
            border_color.set_all(Color::WHITE);
        } else {
            color.0 = NORMAL_BUTTON;
            border_color.set_all(Color::BLACK);
        }

        if selected.map(|s| s.selected) == Some(true) {
            fn assign_color(color: &mut Color) {
                *color = color.mix(&SELECTED, 0.5);
            }

            assign_color(&mut color.0);
            assign_color(&mut border_color.top);
            assign_color(&mut border_color.right);
            assign_color(&mut border_color.bottom);
            assign_color(&mut border_color.left);
        }
    }
}

#[derive(Component)]
#[component(storage = "SparseSet")]
pub struct Focus;

fn focus_system(
    mut commands: Commands,
    focus: Res<InputFocus>,
    focus_visible: Res<InputFocusVisible>,
    mut focus_entities: Query<Entity, With<Focus>>,
) {
    if focus.is_changed() || focus_visible.is_changed() {
        for entity in focus_entities.iter_mut() {
            commands.entity(entity).remove::<(Focus, Outline)>();
        }

        if focus_visible.0
            && let Some(entity) = focus.0
        {
            commands.entity(entity).insert((
                Focus,
                Outline {
                    color: Color::WHITE,
                    width: Val::Px(2.0),
                    offset: Val::Px(2.0),
                },
            ));
        }
    }
}

pub const BACKGROUND: Color = Color::srgb(0.05, 0.05, 0.05);
pub const NORMAL_BUTTON: Color = Color::srgb(0.15, 0.15, 0.15);
pub const HOVERED_BUTTON: Color = Color::srgb(0.25, 0.25, 0.25);
pub const PRESSED_BUTTON: Color = Color::srgb(0.35, 0.75, 0.35);
pub const SELECTED: Color = Color::srgb(0.0, 0.0, 0.45);

pub fn fill_parent_node() -> Node {
    Node {
        flex_direction: FlexDirection::Column,
        width: Val::Percent(100.0),
        height: Val::Percent(100.0),
        row_gap: Val::Px(10.),
        ..default()
    }
}

pub fn node_container() -> Node {
    Node {
        flex_direction: FlexDirection::Column,
        padding: UiRect::all(Val::Px(10.)),
        column_gap: Val::Px(10.),
        row_gap: Val::Px(10.),
        ..default()
    }
}

pub fn compact_node_container() -> Node {
    Node {
        flex_direction: FlexDirection::Column,
        padding: UiRect::all(Val::Px(2.)),
        column_gap: Val::Px(2.),
        row_gap: Val::Px(2.),
        ..default()
    }
}

pub fn row_node_container() -> Node {
    Node {
        flex_direction: FlexDirection::Row,
        padding: UiRect::all(Val::Px(0.)),
        ..node_container()
    }
}

pub fn text_style() -> impl Bundle + use<> {
    (TextColor(Color::srgb(0.9, 0.9, 0.9)), TextShadow::default())
}

pub fn centered_text_style() -> impl Bundle + use<> {
    (
        TextColor(Color::srgb(0.9, 0.9, 0.9)),
        TextShadow::default(),
        TextLayout::new_with_justify(Justify::Center),
    )
}

pub fn title_text_style() -> impl Bundle + use<> {
    let mut text = TextFont::default();
    text.font_size *= 2.0;

    (
        text,
        TextColor(Color::srgb(0.9, 0.9, 0.9)),
        TextShadow::default(),
    )
}

pub fn container_with_background() -> MyStyleBundle {
    let mut node = node_container();
    node.overflow = Overflow::clip();
    node.border = UiRect::all(Val::Px(5.0));

    MyStyleBundle {
        node,
        border_color: BorderColor::all(Color::srgb(1., 0., 0.)),
        border_radius: BorderRadius::all(Val::Px(5.)),
        background_color: BackgroundColor(BACKGROUND),
    }
}

pub fn button_bundle() -> MyButtonBundle {
    MyButtonBundle {
        button: Button,
        // Has Implementation for adding, removing Pressed
        widget_button: bevy::ui_widgets::Button,
        // For bevy_picking to track entity hovered state
        picking: Hovered::default(),

        my_style: MyStyle,
        style: MyStyleBundle {
            node: Node {
                border: UiRect::all(Val::Px(5.0)),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                padding: UiRect::all(Val::Px(5.)),
                ..default()
            },
            border_color: BorderColor::all(Color::BLACK),
            border_radius: BorderRadius::all(Val::Px(5.)),
            background_color: BackgroundColor(NORMAL_BUTTON),
        },
    }
}

pub fn compact_button_bundle() -> MyButtonBundle {
    let mut bundle = button_bundle();
    bundle.style.node.border = UiRect::all(Val::Px(2.));
    bundle.style.node.padding = UiRect::all(Val::Px(2.));
    bundle.style.border_radius = BorderRadius::all(Val::Px(2.));
    bundle
}

#[derive(Bundle)]
pub struct MyStyleBundle {
    pub node: Node,
    pub border_color: BorderColor,
    pub border_radius: BorderRadius,
    pub background_color: BackgroundColor,
}

#[derive(Bundle)]
pub struct MyButtonBundle {
    pub button: Button,
    pub picking: Hovered,
    pub my_style: MyStyle,
    pub style: MyStyleBundle,
    pub widget_button: bevy::ui_widgets::Button,
}

#[allow(clippy::type_complexity)]
fn resizable_style_system(
    mut q_elements: Query<(&Hovered, Has<Pressed>), With<WindowResizeDragDirection>>,
    q_changed: Query<
        Entity,
        (
            Or<(Changed<Hovered>, Changed<Pressed>)>,
            With<WindowResizeDragDirection>,
        ),
    >,
    mut removed_pressed: RemovedComponents<Pressed>,
    mut commands: Commands,
) {
    // Set interactable element
    // background and border colors
    // when inactive, pressed, hovered and when selected

    for entity in q_changed.iter().chain(removed_pressed.read()) {
        let Ok((hovered, pressed)) = q_elements.get_mut(entity) else {
            continue;
        };

        if pressed {
            commands
                .entity(entity)
                .insert(BackgroundColor(WHITE.with_alpha(0.8).into()));
        } else if hovered.get() {
            commands
                .entity(entity)
                .insert(BackgroundColor(WHITE.with_alpha(0.4).into()));
        } else {
            commands.entity(entity).remove::<BackgroundColor>();
        }
    }
}
