use bevy::color::Color;
use bevy::{color::palettes::basic::*, prelude::*, winit::WinitSettings};

pub struct DemoUtilsPlugin;

impl bevy_app::Plugin for DemoUtilsPlugin {
    fn build(&self, app: &mut bevy_app::App) {
        app.insert_resource(WinitSettings::desktop_app());

        app.add_systems(Update, button_system);
    }
}

#[allow(clippy::type_complexity)]
fn button_system(
    mut interaction_query: Query<
        (&Interaction, &mut BackgroundColor, &mut BorderColor),
        (Changed<Interaction>, With<Button>),
    >,
) {
    for (interaction, mut color, mut border_color) in &mut interaction_query {
        match *interaction {
            Interaction::Pressed => {
                *color = PRESSED_BUTTON.into();
                border_color.0 = RED.into();
            }
            Interaction::Hovered => {
                *color = HOVERED_BUTTON.into();
                border_color.0 = Color::WHITE;
            }
            Interaction::None => {
                *color = NORMAL_BUTTON.into();
                border_color.0 = Color::BLACK;
            }
        }
    }
}

pub const BACKGROUND: Color = Color::srgb(0.05, 0.05, 0.05);
pub const NORMAL_BUTTON: Color = Color::srgb(0.15, 0.15, 0.15);
pub const HOVERED_BUTTON: Color = Color::srgb(0.25, 0.25, 0.25);
pub const PRESSED_BUTTON: Color = Color::srgb(0.35, 0.75, 0.35);

pub fn node_full_screen_centered() -> Node {
    Node {
        flex_direction: FlexDirection::Column,
        width: Val::Percent(100.0),
        height: Val::Percent(100.0),
        align_items: AlignItems::Center,
        justify_content: JustifyContent::Center,
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

pub fn my_text_style() -> impl Bundle + use<> {
    (TextColor(Color::srgb(0.9, 0.9, 0.9)), TextShadow::default())
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
                // horizontally center child text
                justify_content: JustifyContent::Center,
                // vertically center child text
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
    node: Node,
    border_color: BorderColor,
    border_radius: BorderRadius,
    background_color: BackgroundColor,
}

#[derive(Bundle)]
pub struct MyButtonBundle {
    button: Button,
    style: MyStyleBundle,
    interact: Interaction,
}
