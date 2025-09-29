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

/// For capability set macro
pub use paste;

/// Helper macro to create child nodes
/// with macro location used as source for id value generation   
///
/// Unique id is derived from line and column numbers
#[macro_export]
macro_rules! lch {
    ($ui:ident) => {
        $ui.ch_id($crate::lid!())
    };
    ($ui:ident, $id:ident) => {
        $ui.ch_id($crate::lid!($id))
    };
}

/// Helper macro to generate unique id for child nodes
/// with macro location used as source for id value generation   
///
/// Unique id is derived from line and column numbers
///
/// Contains large random constant to avoid manual collisions by mistake
#[macro_export]
macro_rules! lid {
    () => {
        (596784345354, line!(), column!())
    };
    ($id:ident) => {
        (596784345354, line!(), column!(), $id)
    };
}
