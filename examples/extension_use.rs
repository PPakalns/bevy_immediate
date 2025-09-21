use bevy_ecs::{component::Component, system::SystemParam};
use bevy_immediate::{
    Imm,
    attach::{BevyImmediateAttachPlugin, ImmediateAttach},
    impl_capability_set,
    ui::{CapsUi, ImplCapsUi},
};

pub struct ExtensionUseExamplePlugin;

impl bevy_app::Plugin for ExtensionUseExamplePlugin {
    fn build(&self, app: &mut bevy_app::App) {
        // Initialize plugin with your root component
        app.add_plugins(BevyImmediateAttachPlugin::<CapsUi, ExtensionUseExampleRoot>::new());
    }
}

// To add your own extensions
//
// Create new type
pub struct CapMyUi;

impl_capability_set!(
    CapMyUi,
    ImplCapMyUi > ImplCapsUi,
    (
        // You need to list all capabilities
        //
        // Macro will add compile time check to check that
        // you correctly listed them
        //
        // Sadly rust type system has restrictions :( and transitive extensions are not possible
        // About future improvements: TODO
        bevy_immediate::ui::ui_base::CapabilityUiBase,
        bevy_immediate::ui::ui_layout_order::CapabilityUiLayoutOrder,
        bevy_immediate::ui::interaction::CapabilityUiInteraction,
        bevy_immediate::ui::text::CapabilityUiText,
        bevy_immediate::ui::picking::clicked::CapabilityUiClicked,
        //
        // Add your own capabilities
        crate::extension::CapUiToggle
    )
);

#[derive(Component)]
pub struct ExtensionUseExampleRoot;

#[derive(SystemParam)]
pub struct Params {}

impl ImmediateAttach<CapsUi> for ExtensionUseExampleRoot {
    type Params = Params;

    fn construct(ui: &mut Imm<CapsUi>, params: &mut Params) {}
}
