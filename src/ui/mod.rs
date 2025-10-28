use crate::{capabilities::ImplCapsEmpty, impl_capability_set};

/// Defines capability set for Ui
pub struct CapsUi;

impl_capability_set!(
    CapsUi,
    ImplCapsUi > ImplCapsEmpty,
    (
        base::CapabilityUiBase,
        layout_order::CapabilityUiLayoutOrder,
        look::CapabilityUiLook,
        disabled::CapabilityUiDisabled,
        interaction::CapabilityUiInteraction,
        text::CapabilityUiText,
        selected::CapabilityUiSelectable,
        checked::CapabilityUiChecked,
        clicked::CapabilityUiClicked,
        activated::CapabilityUiActivated,
        anchored::CapabilityUiAnchored,
    )
);

/// Defines capability set for Ui with bevy_ui_widgets support
///
/// If you want to use bevy feathers, you probably want to use [CapsUiFeathers]
pub struct CapsUiWidget;

#[cfg(feature = "bevy_ui_widgets")]
impl_capability_set!(
    CapsUiWidget,
    ImplCapsUiWidget > ImplCapsUi,
    (
        base::CapabilityUiBase,
        layout_order::CapabilityUiLayoutOrder,
        look::CapabilityUiLook,
        disabled::CapabilityUiDisabled,
        interaction::CapabilityUiInteraction,
        text::CapabilityUiText,
        selected::CapabilityUiSelectable,
        checked::CapabilityUiChecked,
        clicked::CapabilityUiClicked,
        activated::CapabilityUiActivated,
        anchored::CapabilityUiAnchored,
        //
        slider_value::CapabilityUiSliderValue,
    )
);

/// Defined capability set for Ui with bevy_feathers and bevy_ui_widgets support
pub struct CapsUiFeathers;

#[cfg(feature = "bevy_feathers")]
impl_capability_set!(
    CapsUiFeathers,
    ImplCapsUiFeathers > ImplCapsUiWidget,
    (
        base::CapabilityUiBase,
        layout_order::CapabilityUiLayoutOrder,
        look::CapabilityUiLook,
        disabled::CapabilityUiDisabled,
        interaction::CapabilityUiInteraction,
        text::CapabilityUiText,
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

/// Implements capabilities for Selected marker component
pub mod selected;

/// Implements capabilities for checked status
pub mod checked;

/// Implements capabilities for detecting activated entity
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

/// Contains implementation to work around inconsistent
/// radio button implementation in bevy
#[cfg(feature = "bevy_ui_widgets")]
pub mod radio_button_fix_plugin;

/// Implements plugin for floating windows
pub mod floating_window_plugin;

/// Implements plugin for floating anchored entities
pub mod anchored_ui_plugin;

/// Implements plugin for floating entity ordering
pub mod floating_ui_ordering_plugin;

/// Implements plugin for handling user focus over floating ui hierarchies
pub mod floating_ui_focus_plugin;

/// Implements plugin for calculating when tooltip should be displayed
pub mod tooltip_plugin;

/// Helper functions for ui calculations
pub mod utils;
