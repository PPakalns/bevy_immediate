use bevy::ecs::component::Component;
use bevy::ui::{
    BackgroundColor, BorderColor, BorderRadius, FlexDirection, Node, UiRect, Val,
    widget::{Text, TextShadow},
};
use bevy::{color::Color, text::TextColor, utils::default};
use bevy_immediate::{
    Imm,
    attach::{BevyImmediateAttachPlugin, ImmediateAttach},
    ui::CapsUi,
};

pub struct HelloWorldPlugin;

impl bevy::app::Plugin for HelloWorldPlugin {
    fn build(&self, app: &mut bevy::app::App) {
        // Add bevy immediate plugin with UI support which will construct UI
        // rooted at entity with `HelloWorldRoot` component
        app.add_plugins(BevyImmediateAttachPlugin::<CapsUi, HelloWorldRoot>::new());
    }
}

#[derive(Component)]
pub struct HelloWorldRoot;

impl ImmediateAttach<CapsUi> for HelloWorldRoot {
    type Params = (); // Access data from World using SystemParam

    fn construct(ui: &mut Imm<CapsUi>, _: &mut ()) {
        // Construct entity hierarchies
        // and attach necessary components
        ui.ch()
            .on_spawn_insert(|| {
                (
                    Node {
                        flex_direction: FlexDirection::Column,
                        border: UiRect::all(Val::Px(10.)),
                        padding: UiRect::all(Val::Px(10.)),
                        column_gap: Val::Px(10.),
                        row_gap: Val::Px(10.),
                        border_radius: BorderRadius::all(Val::Px(5.)),
                        ..default()
                    },
                    BorderColor::all(Color::srgb(1., 0., 0.)),
                    BackgroundColor(Color::srgb(0.05, 0.05, 0.05)),
                )
            })
            .add(|ui| {
                ui.ch().on_spawn_insert(|| {
                    (
                        TextColor(Color::srgb(0.9, 0.9, 0.9)),
                        TextShadow::default(),
                        Text("Hello world".into()),
                    )
                });
            });
    }
}

// To spawn UI rooted at HelloWorldRoot, you need to add it to world
//
// 1. Using classical approach
// ```
// fn commands(mut commmand: Commands) {
//     commands.spawn((Node::DEFAULT, HelloWorldRoot));
// }
// ```
// Remember to spawn necessary camera for UI too if not done already.
//
//
// 2. Spawn ui ussing immediate mode systems
//
// See "./plain_ui.rs" example
