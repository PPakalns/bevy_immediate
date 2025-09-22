use bevy::color::Color;
use bevy::{color::palettes::basic::*, prelude::*};
use bevy_immediate::ui::selected::Selectable;

pub struct DemoStylePlugin;

impl bevy_app::Plugin for DemoStylePlugin {
    fn build(&self, app: &mut bevy_app::App) {
        app.add_systems(Update, button_system);
    }
}

#[allow(clippy::type_complexity)]
fn button_system(
    mut interaction_query: Query<
        (
            &Interaction,
            &mut BackgroundColor,
            &mut BorderColor,
            Option<&mut Selectable>,
        ),
        (
            Or<(Changed<Interaction>, Changed<Selectable>)>,
            With<Button>,
        ),
    >,
) {
    for (interaction, mut color, mut border_color, selected) in &mut interaction_query {
        match *interaction {
            Interaction::Pressed => {
                color.0 = PRESSED_BUTTON;
                border_color.0 = RED.into();
            }
            Interaction::Hovered => {
                color.0 = HOVERED_BUTTON;
                border_color.0 = Color::WHITE;
            }
            Interaction::None => {
                color.0 = NORMAL_BUTTON;
                border_color.0 = Color::BLACK;
            }
        }
        if selected.map(|s| s.selected) == Some(true) {
            color.0 = color.0.mix(&SELECTED, 0.5);
            border_color.0 = border_color.0.mix(&SELECTED, 0.5);
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

pub fn text_style() -> impl Bundle + use<> {
    (TextColor(Color::srgb(0.9, 0.9, 0.9)), TextShadow::default())
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
    node.border = UiRect::all(Val::Px(5.0));

    MyStyleBundle {
        node,
        border_color: BorderColor(Color::srgb(1., 0., 0.)),
        border_radius: BorderRadius::all(Val::Px(5.)),
        background_color: BackgroundColor(BACKGROUND),
    }
}

pub fn button_bundle() -> MyButtonBundle {
    MyButtonBundle {
        button: Button,
        interact: Interaction::None,
        style: MyStyleBundle {
            node: Node {
                border: UiRect::all(Val::Px(5.0)),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                padding: UiRect::all(Val::Px(5.)),
                ..default()
            },
            border_color: BorderColor(Color::BLACK),
            border_radius: BorderRadius::all(Val::Px(5.)),
            background_color: BackgroundColor(NORMAL_BUTTON),
        },
    }
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
    pub style: MyStyleBundle,
    pub interact: Interaction,
}
