#[cfg(feature = "picking")]
use crate::capabilities::ImplCapsEmpty;
use crate::impl_capability_set;

/// Defines capability set for Ui
pub struct CapsUi;

#[cfg(feature = "picking")]
impl_capability_set!(
    CapsUi,
    ImplCapsUi > ImplCapsEmpty,
    (
        ui_base::CapabilityUiBase,
        ui_layout_order::CapabilityUiLayoutOrder,
        interaction::CapabilityUiInteraction,
        text::CapabilityUiText,
        // picking
        picking::clicked::CapabilityUiClicked,
    )
);

#[cfg(not(feature = "picking"))]
impl_capability_set!(
    CapsUi,
    ImplCapsUi > ImplCapsEmpty,
    (
        ui_base::CapabilityUiBase,
        ui_layout_order::CapabilityUiLayoutOrder,
        interaction::CapabilityUiInteraction,
        text::CapabilityUiText,
    )
);

////////////////////////////////////////////////////////////////////////////////

/// Implements capability that correctly set ups immediate mode systems execution order for UI case
pub mod ui_base;

/// Implements functionality to place children entities created in immediate mode in creation (.ch)
/// call order. Needed for correct Ui layout.
pub mod ui_layout_order;

/// Implements functionality to access [`bevy_ui::Interaction`]
pub mod interaction;

/// Implements capabilities for working with nodes that contain [`bevy_ui::widget::Text`]
pub mod text;

/// Contains API extensions for ergonomic API that use [`bevy_picking`]
#[cfg(feature = "picking")]
pub mod picking;
