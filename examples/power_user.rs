use bevy::text::TextFont;
use bevy_ecs::{
    component::Component,
    resource::Resource,
    system::{ResMut, SystemParam},
};
use bevy_immediate::{
    Imm, ImmEntity,
    attach::{BevyImmediateAttachPlugin, ImmediateAttach},
    ui::{
        CapsUi, ImplCapsUi, picking::clicked::ImmUiClicked, selected::ImmUiSelected,
        text::ImmUiText,
    },
};
use bevy_ui::{FlexDirection, UiRect, Val};

use crate::styles;

pub struct PowerUserExamplePlugin;

impl bevy_app::Plugin for PowerUserExamplePlugin {
    fn build(&self, app: &mut bevy_app::App) {
        // Initialize plugin with your root component
        app.add_plugins(BevyImmediateAttachPlugin::<CapsUi, PowerUserExampleRoot>::new());
        app.insert_resource(YesNoResource { yes: false });
    }
}

#[derive(Resource)]
struct YesNoResource {
    yes: bool,
}

#[derive(Component)]
pub struct PowerUserExampleRoot;

#[derive(SystemParam)]
pub struct Params<'w> {
    yes_no_res: ResMut<'w, YesNoResource>,
}

impl ImmediateAttach<CapsUi> for PowerUserExampleRoot {
    type Params = Params<'static>;

    fn construct(ui: &mut Imm<CapsUi>, params: &mut Params) {
        ui.ch().my_title("Bevy power user example");
        ui.ch()
            .my_subtitle("Use helper functions to simplify and reuse code!");

        ui.ch().my_subtitle("Show collapsible element");
        ui.ch().my_row_container().add(|ui| {
            for (text, state) in [("No", false), ("Yes", true)] {
                let mut button = ui
                    .ch_id(("choice", state))
                    .my_button()
                    .selected(params.yes_no_res.yes == state)
                    .add(|ui| {
                        ui.ch().my_text(text);
                    });
                if button.clicked() {
                    params.yes_no_res.yes = state;
                }
            }
        });

        if params.yes_no_res.yes {
            ui.ch_id("yes_no").my_container_with_background().add(|ui| {
                ui.ch().my_text("Lorem Ipsum!");
            });
        }

        ui.ch().my_text("It is really simple!");
    }
}

pub trait PowerUserHelper {
    fn my_title(self, text: &str) -> Self;
    fn my_subtitle(self, text: &str) -> Self;
    fn my_text(self, text: &str) -> Self;
    fn my_button(self) -> Self;
    fn my_row_container(self) -> Self;
    fn my_container_with_background(self) -> Self;
}

impl<Caps> PowerUserHelper for ImmEntity<'_, '_, '_, Caps>
where
    Caps: ImplCapsUi,
{
    fn my_title(self, text: &str) -> Self {
        self.on_spawn_insert(styles::text_style)
            .on_spawn_insert(|| {
                let mut text_font = TextFont::default();
                text_font.font_size *= 1.5;
                text_font
            })
            .text(text)
    }

    fn my_subtitle(self, text: &str) -> Self {
        self.on_spawn_insert(styles::text_style)
            .on_spawn_insert(|| {
                let mut text_font = TextFont::default();
                text_font.font_size *= 1.2;
                text_font
            })
            .text(text)
    }

    fn my_text(self, text: &str) -> Self {
        self.on_spawn_insert(styles::text_style).text(text)
    }

    fn my_button(self) -> Self {
        self.on_spawn_insert(styles::button_bundle)
    }

    fn my_row_container(self) -> Self {
        self.on_spawn_insert(|| {
            let mut style = styles::node_container();
            style.flex_direction = FlexDirection::Row;
            style.padding = UiRect::all(Val::Px(0.));
            style
        })
    }
    fn my_container_with_background(self) -> Self {
        self.on_spawn_insert(styles::container_with_background)
    }
}
