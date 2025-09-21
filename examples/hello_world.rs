use bevy::{color::Color, text::TextColor, utils::default};
use bevy_ecs::component::Component;
use bevy_immediate::{
    Imm,
    attach::{BevyImmediateAttachPlugin, ImmediateAttach},
    ui::CapsUi,
};
use bevy_ui::{
    BackgroundColor, BorderColor, BorderRadius, FlexDirection, Node, TextShadow, UiRect, Val,
    widget::Text,
};

pub struct HelloWorldPlugin;

impl bevy_app::Plugin for HelloWorldPlugin {
    fn build(&self, app: &mut bevy_app::App) {
        // Add plugin
        app.add_plugins(BevyImmediateAttachPlugin::<CapsUi, HelloWorldRoot>::new());
    }
}

#[derive(Component)]
pub struct HelloWorldRoot;

impl ImmediateAttach<CapsUi> for HelloWorldRoot {
    type Params = (); // SystemParams from `World``

    fn construct(ui: &mut Imm<CapsUi>, _: &mut ()) {
        // CapsUi - Capability set for Ui.
        // Add usefult extensions for implementing UI with rust type system support!

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
                        ..default()
                    },
                    BorderColor(Color::srgb(1., 0., 0.)),
                    BorderRadius::all(Val::Px(5.)),
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
