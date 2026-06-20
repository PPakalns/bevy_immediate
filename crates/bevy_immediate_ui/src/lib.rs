#![warn(missing_docs)]
#![doc = include_str!("../README.md")]

use bevy_immediate_core::{capabilities::ImplCapsEmpty, impl_capability_set};

/// Capabilities for bevy_ui.
pub struct CapsUiBase;

// Definition without bevy_ui_widgets
impl_capability_set!(
    CapsUiBase,
    ImplCapsUiBase > ImplCapsEmpty,
    (
        base::CapabilityUiBase,
        layout_order::CapabilityUiLayoutOrder,
        look::CapabilityUiLook,
        disabled::CapabilityUiDisabled,
        interaction::CapabilityUiInteraction,
        text::CapabilityUiText,
        text_input::CapabilityUiTextInput,
        selected::CapabilityUiSelectable,
        checked::CapabilityUiChecked,
        clicked::CapabilityUiClicked,
        anchored::CapabilityUiAnchored,
    )
);

/// Defines capability set for Ui
///
/// If bevy_ui_widgets feature is enabled, will include capabilities for it too.
pub struct CapsUi;

// Definition without bevy_ui_widgets
#[cfg(not(feature = "bevy_ui_widgets"))]
impl_capability_set!(
    CapsUi,
    ImplCapsUi > ImplCapsUiBase,
    (
        base::CapabilityUiBase,
        layout_order::CapabilityUiLayoutOrder,
        look::CapabilityUiLook,
        disabled::CapabilityUiDisabled,
        interaction::CapabilityUiInteraction,
        text::CapabilityUiText,
        text_input::CapabilityUiTextInput,
        selected::CapabilityUiSelectable,
        checked::CapabilityUiChecked,
        clicked::CapabilityUiClicked,
        anchored::CapabilityUiAnchored,
    )
);

// Definition with bevy_ui_widgets capabilities
#[cfg(feature = "bevy_ui_widgets")]
impl_capability_set!(
    CapsUi,
    ImplCapsUi > ImplCapsUiBase,
    (
        base::CapabilityUiBase,
        layout_order::CapabilityUiLayoutOrder,
        look::CapabilityUiLook,
        disabled::CapabilityUiDisabled,
        interaction::CapabilityUiInteraction,
        text::CapabilityUiText,
        text_input::CapabilityUiTextInput,
        selected::CapabilityUiSelectable,
        checked::CapabilityUiChecked,
        clicked::CapabilityUiClicked,
        activated::CapabilityUiActivated,
        anchored::CapabilityUiAnchored,
        // bevy_ui_widgets
        slider_value::CapabilityUiSliderValue,
    )
);

/// Defined capability set for Ui with bevy_feathers and bevy_ui_widgets support
pub struct CapsUiFeathers;

#[cfg(feature = "bevy_feathers")]
impl_capability_set!(
    CapsUiFeathers,
    ImplCapsUiFeathers > ImplCapsUi,
    (
        base::CapabilityUiBase,
        layout_order::CapabilityUiLayoutOrder,
        look::CapabilityUiLook,
        disabled::CapabilityUiDisabled,
        interaction::CapabilityUiInteraction,
        text::CapabilityUiText,
        text_input::CapabilityUiTextInput,
        selected::CapabilityUiSelectable,
        checked::CapabilityUiChecked,
        clicked::CapabilityUiClicked,
        activated::CapabilityUiActivated,
        anchored::CapabilityUiAnchored,
        // bevy_ui_widgets
        slider_value::CapabilityUiSliderValue,
        // bevy_feathers
        slider_base_color::CapabilityUiSliderBaseColor,
        button_variant::CapabilityUiFeathersButtonVariant,
    )
);

////////////////////////////////////////////////////////////////////////////////

/// Implements capability that correctly set ups immediate mode systems execution order for UI case
pub mod base;

/// Implements functionality to place children entities created in immediate mode in creation (.ch)
/// call order. Needed for correct Ui layout.
pub mod layout_order;

/// Implements functions to access node styling
pub mod look;

/// Implements functions to manage disabled node state
pub mod disabled;

/// Implements functionality to calculate interaciton state
pub mod interaction;

/// Implements capabilities for working with nodes that contain [`bevy_ui::widget::Text`]
pub mod text;

/// Implements capabilities for working with nodes that contain [`bevy_text::EditableText`]
pub mod text_input;

/// Implements capabilities for Selected marker component
pub mod selected;

/// Implements capabilities for checked status
pub mod checked;

/// Implements capabilities for detecting activated entity
#[cfg(feature = "bevy_ui_widgets")]
pub mod activated;

/// Module implments `Pointer<Click>` related
/// functionality like `.clicked()`
pub mod clicked;

/// Contains plugin implementation for value change entity event tracking
pub mod track_value_change_plugin;

/// Module implements functionality for setting feathers SliderValue
#[cfg(feature = "bevy_ui_widgets")]
pub mod slider_value;

/// Module implements functionality for setting feathers Slider base color
#[cfg(feature = "bevy_feathers")]
pub mod slider_base_color;

/// Module implements functionality for setting feathers button variant
#[cfg(feature = "bevy_feathers")]
pub mod button_variant;

/// Implements capabilities for floating anchored elements
pub mod anchored;

pub use bevy_immediate_floating_ui::{
    anchored_ui_plugin, floating_ui_focus_plugin, floating_ui_ordering_plugin,
    floating_window_plugin, tooltip_plugin, utils,
};
