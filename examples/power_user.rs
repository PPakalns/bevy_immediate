use bevy_ecs::{component::Component, system::SystemParam};
use bevy_immediate::{
    Imm,
    attach::{BevyImmediateAttachPlugin, ImmediateAttach},
    ui::CapsUi,
};

pub struct PowerUserExamplePlugin;

impl bevy_app::Plugin for PowerUserExamplePlugin {
    fn build(&self, app: &mut bevy_app::App) {
        // Initialize plugin with your root component
        app.add_plugins(BevyImmediateAttachPlugin::<CapsUi, PowerUserExampleRoot>::new());
    }
}

#[derive(Component)]
pub struct PowerUserExampleRoot;

#[derive(SystemParam)]
pub struct Params {}

impl ImmediateAttach<CapsUi> for PowerUserExampleRoot {
    type Params = Params;

    fn construct(ui: &mut Imm<CapsUi>, params: &mut Params) {}
}
