use bevy_ecs::{component::Component, system::SystemParam};
use bevy_immediate::{
    Imm,
    attach::{BevyImmediateAttachPlugin, ImmediateAttach},
    impl_capabilities,
    ui::CapUi,
};

use crate::extension::CapUiToggle;

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

impl bevy_immediate::ImmCap for CapMyUi {
    fn build<Cap: bevy_immediate::ImmCap>(
        app: &mut bevy_app::App,
        cap_req: &mut bevy_immediate::ImmCapAccessRequests<Cap>,
    ) {
        <CapUiToggle as bevy_immediate::ImmCap>::build(app, cap_req);
        <CapUi as bevy_immediate::ImmCap>::build(app, cap_req);
    }
}
impl<T: bevy_immediate::ImplCap<CapMyUi>> bevy_immediate::ImplCap<CapUiToggle> for T {}

impl<T: bevy_immediate::ImplCap<CapMyUi>> bevy_immediate::ImplCap<CapUi> for T {}

#[derive(Component)]
pub struct ExtensionUseExampleRoot;

#[derive(SystemParam)]
pub struct Params {}

impl ImmediateAttach<CapUi> for ExtensionUseExampleRoot {
    type Params = Params;

    fn construct(ui: &mut Imm<CapUi>, params: &mut Params) {}
}
