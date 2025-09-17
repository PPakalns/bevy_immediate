#![warn(missing_docs)]
#![doc = include_str!("../README.md")]

/// Base logic to provide immediate mode api
mod immediate;
pub use immediate::*;

/// Exstention support for immediate mode. Logic to implement additional capabilities.
mod capabilities;
pub use capabilities::{
    CapAccessRequests, CapAccessRequestsResource, CapQueryParam, ImmCap, ImmImplCap,
};

/// Extensions to provide ergonomic functionality for working with
/// creating UI ([`bevy_ui`]) in immediate mode
#[cfg(feature = "ui")]
pub mod ui;
