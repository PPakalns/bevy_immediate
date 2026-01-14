use bevy::ecs::{component::Component, system::SystemParam};
use bevy_immediate::{
    Imm,
    attach::{BevyImmediateAttachPlugin, ImmediateAttach},
    impl_capability_set,
    ui::{ImplCapsUi, clicked::ImmUiClicked, selected::ImmUiSelectable, text::ImmUiText},
};

use crate::{extension::ImmCapUiCollapse, styles};

// Create custom capability set
pub struct CapsMyUi;

impl_capability_set!(
    CapsMyUi,
    ImplCapMyUi > ImplCapsUi,
    (
        // You need to list all capabilities
        //
        // Macro will add compile time check to check that
        // you correctly listed them
        bevy_immediate::ui::base::CapabilityUiBase,
        bevy_immediate::ui::layout_order::CapabilityUiLayoutOrder,
        bevy_immediate::ui::look::CapabilityUiLook,
        bevy_immediate::ui::disabled::CapabilityUiDisabled,
        bevy_immediate::ui::interaction::CapabilityUiInteraction,
        bevy_immediate::ui::text::CapabilityUiText,
        bevy_immediate::ui::selected::CapabilityUiSelectable,
        bevy_immediate::ui::checked::CapabilityUiChecked,
        bevy_immediate::ui::clicked::CapabilityUiClicked,
        bevy_immediate::ui::activated::CapabilityUiActivated,
        bevy_immediate::ui::anchored::CapabilityUiAnchored,
        //
        // Add your own capabilities
        crate::extension::CapUiToggle,
        // crate::text_edit::CapabilityUiTextInput,
    )
);

pub struct ExtensionUseExamplePlugin;

impl bevy::app::Plugin for ExtensionUseExamplePlugin {
    fn build(&self, app: &mut bevy::app::App) {
        // Initialize plugin with your custom capability set and root component
        app.add_plugins(BevyImmediateAttachPlugin::<CapsMyUi, ExtensionUseExampleRoot>::new());
    }
}

#[derive(Component)]
pub struct ExtensionUseExampleRoot;

#[derive(SystemParam)]
pub struct Params {}

// Use new capability set: CapsMyUi
impl ImmediateAttach<CapsMyUi> for ExtensionUseExampleRoot {
    type Params = Params;

    fn construct(ui: &mut Imm<CapsMyUi>, _params: &mut Params) {
        let mut button = ui.ch().on_spawn_insert(styles::button_bundle);

        let toggle = button.get_toggle();

        button = button.selected(toggle).add(|ui| {
            if toggle {
                ui.ch().text("Open");
            } else {
                ui.ch().text("Closed");
            };
        });

        if button.clicked() {
            button.flip_toggle();
        }

        if toggle {
            ui.ch_id("my_hidden_element")
                .on_spawn_insert(styles::container_with_background)
                .add(|ui| {
                    ui.ch().text("Collapsable content");
                });
        }

        ui.ch().text("Text after collapsable element");
    }
}
