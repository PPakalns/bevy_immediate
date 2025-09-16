#![forbid(unsafe_code)]
#![warn(missing_docs)]
#![doc = include_str!("../README.md")]

/// Base logic to provide immediate mode api
mod immediate;
pub use immediate::*;

/// Extensions to provide ergonomic functionality for working with
/// creating UI ([`bevy_ui`]) in immediate mode
#[cfg(feature = "ui")]
pub mod ui;
#[cfg(feature = "ui")]
pub use ui::BevyImmediateUiExtensionPlugin;
