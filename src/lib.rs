#![warn(missing_docs)]
#![doc = include_str!("../README.md")]

/// Base logic to provide immediate mode api
pub mod immediate;
pub use immediate::*;

/// Logic to provide immediate mode api
/// to attach entity tree to already existing entity
pub mod attach;

/// Exstention support for immediate mode. Logic to implement additional capabilities.
mod capabilities;
pub use capabilities::{
    CapSet, ImmCapAccessRequests, ImmCapAccessRequestsResource, ImmCapQueryParam, ImmCapability,
    ImplCap,
};

/// Extensions to provide ergonomic functionality for working with
/// creating UI ([`bevy_ui`]) in immediate mode
#[cfg(feature = "ui")]
pub mod ui;

/// Utility types to simplify implementation
pub mod utils;
