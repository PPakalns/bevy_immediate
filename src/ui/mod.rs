use crate::merge_capabilities;

/// Defines capability that contains all Ui capabilities from this crate
#[cfg(feature = "picking")]
merge_capabilities!(CapUi, (CapUiWithoutFeatures, picking::CapUiPickingAll));

#[cfg(not(feature = "picking"))]
merge_capabilities!(ImmUiCap, (ImmCapUiWithoutPicking));

////////////////////////////////////////////////////////////////////////////////

/// Defines all ui capabilities except capabilities provided by "picking" - [`bevy_picking`].
merge_capabilities!(
    CapUiWithoutFeatures,
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
