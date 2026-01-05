#![warn(missing_docs)]
#![doc = include_str!("../README.md")]

// Core logic
pub use bevy_immediate_core::immediate;
pub use bevy_immediate_core::immediate::*;

// Capability set logic (Extensions / Plugins)
pub use bevy_immediate_core::capabilities;
pub use bevy_immediate_core::impl_capability_set;
pub use bevy_immediate_core::{
    CapSet, ImmCapAccessRequests, ImmCapAccessRequestsResource, ImmCapQueryParam, ImmCapability,
    ImplCap, ImplCapsEmpty,
};

/// Reexport helper macros
pub use bevy_immediate_core::{lch, lid};

/// Reexport utils
pub use bevy_immediate_core::utils;

/// Reexport full bevy_immediate_core
pub use bevy_immediate_core;

/// Logic to provide immediate mode api
/// to attach entity tree to already existing entity
pub use bevy_immediate_attach as attach;

/// Extensions to provide ergonomic functionality for working with
/// creating UI ([`bevy_ui`]) in immediate mode
#[cfg(feature = "ui")]
pub mod ui;

/// For capability set macro
pub use paste;
