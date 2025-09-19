#[cfg(feature = "picking")]
use crate::capabilities::ImplEmpty;
use crate::impl_capabilities;

/// Defines capability that contains all Ui capabilities from this crate
pub struct CapUi;

#[cfg(feature = "picking")]
impl_capabilities!(
    CapUi,
    ImplCapUi > ImplEmpty,
    (
        ui_base::CapabilityUiBase,
        interaction::CapabilityUiInteraction,
        text::CapabilityUiText,
        picking::clicked::CapabilityUiClicked,
    )
);

#[cfg(not(feature = "picking"))]
impl_capabilities!(
    CapUi,
    ImplCapUi > ImplEmpty,
    (
        ui_base::CapUiBase,
        interaction::CapUiInteraction,
        text::CapUiText
    )
);

////////////////////////////////////////////////////////////////////////////////

/// Implements capability that correctly set ups immediate mode systems for UI case
pub mod ui_base;

/// Implements functionality to access [`bevy_ui::Interaction`]
pub mod interaction;

/// Implements capabilities for working with nodes that contain [`bevy_ui::widget::Text`]
pub mod text;

/// Contains API extensions for ergonomic API that use [`bevy_picking`]
#[cfg(feature = "picking")]
pub mod picking;
