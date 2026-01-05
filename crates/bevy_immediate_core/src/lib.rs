/// Base logic to provide immediate mode api
pub mod immediate;
pub use immediate::*;

/// Exstention support for immediate mode. Logic to implement additional capabilities.
pub mod capabilities;
pub use capabilities::{
    CapSet, ImmCapAccessRequests, ImmCapAccessRequestsResource, ImmCapQueryParam, ImmCapability,
    ImplCap, ImplCapsEmpty,
};

/// Utility types to simplify implementation
pub mod utils;

/// Required by impl_capability_set macro
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
    ($ui:ident, $id:expr) => {
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
    ($id:expr) => {
        (596784345354, line!(), column!(), $id)
    };
}
