#![warn(missing_docs)]
#![doc = include_str!("../README.md")]

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
