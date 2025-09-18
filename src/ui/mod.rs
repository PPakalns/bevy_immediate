use crate::impl_capabilities;

/// Defines capability that contains all Ui capabilities from this crate
pub struct CapUi;

#[cfg(feature = "picking")]
impl_capabilities!(
    CapUi,
    (
        (CapUiWithoutFeatures, ImplCapUiWithoutFeatures),
        (picking::CapUiPickingAll, picking::ImplCapUiPickingAll)
    )
);
#[cfg(not(feature = "picking"))]
impl_capabilities!(ImmUiCap, (ImmCapUiWithoutPicking));

////////////////////////////////////////////////////////////////////////////////

/// Defines all ui capabilities except capabilities provided by "picking" - [`bevy_picking`].
pub struct CapUiWithoutFeatures;
impl_capabilities!(
    CapUiWithoutFeatures,
    (
        (ui_base::CapUiBase, ui_base::ImplCapUiBase),
        (
            interaction::CapUiInteraction,
            interaction::ImplCapUiInteraction
        ),
        (text::CapUiText, text::ImplCapUiText)
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
