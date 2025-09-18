use bevy_ecs::{component::Component, system::SystemParam};
use bevy_immediate::{
    Imm,
    attach::{BevyImmediateAttachPlugin, ImmediateAttach},
    impl_capabilities,
    ui::{CapUi, ImplCapUi},
};

use crate::extension::{CapUiToggle, ImplCapUiToggle};

pub struct ExtensionUseExamplePlugin;

impl bevy_app::Plugin for ExtensionUseExamplePlugin {
    fn build(&self, app: &mut bevy_app::App) {
        // Initialize plugin with your root component
        app.add_plugins(BevyImmediateAttachPlugin::<CapUi, ExtensionUseExampleRoot>::new());
    }
}

// To add your own extensions
//
// Create new type
pub struct CapMyUi;

impl_capabilities!(
    CapMyUi,
    ((CapUiToggle, ImplCapUiToggle), (CapUi, ImplCapUi),)
);

#[derive(Component)]
pub struct ExtensionUseExampleRoot;

#[derive(SystemParam)]
pub struct Params {}

impl ImmediateAttach<CapUi> for ExtensionUseExampleRoot {
    type Params = Params;

    fn construct(ui: &mut Imm<CapUi>, params: &mut Params) {}
}
