/// Marks types that implement immediate mode capabilities
pub trait ImmCap: Send + Sync + 'static {
    /// Function used to initialize necessary resources for capability to fully function
    fn build<Cap: ImmCap>(app: &mut bevy_app::App, cap_req: &mut CapAccessRequests<Cap>);
}

/// Trait that marks what capabilities current capability implements
///
/// Capability can implement many sub-capabilities
pub trait ImmImplCap<T>: ImmCap {}
impl<T: ImmCap> ImmImplCap<T> for T {}

impl ImmCap for () {
    fn build<Cap: ImmCap>(app: &mut bevy_app::App, cap_req: &mut CapAccessRequests<Cap>) {
        let _ = cap_req;
        let _ = app;
    }
}

/// Implements list of capabilities for given type
///
/// ```no_run
/// pub struct MyCapability;
///
/// impl_capabilities!(MyCapability, (Cap1, Cap2, Cap3));
/// ````
///
#[macro_export]
macro_rules! impl_capabilities {
    ($name:ty, ($($t:ty),+ $(,)?)) => {
        impl $crate::ImmCap for $name {
            fn build<Cap: $crate::ImmCap>(
                app: &mut bevy_app::App,
                cap_req: &mut $crate::CapAccessRequests<Cap>,
            ) {
                $(<$t as $crate::ImmCap>::build(app, cap_req);)+
            }
        }

        $(
            impl<T: ImmImplCap<$name>> ImmImplCap<$t> for T {}
        )+
    };
}

/// Implements logic for collecting requested components and resources
mod access_requests;
pub use access_requests::{CapAccessRequests, CapAccessRequestsResource};

/// Implements [`bevy_ecs::system::SystemParam`] for [`CapSystemParams`] that
/// allows to retrieve all requested data by capabilities
mod system_param;
pub use system_param::CapSystemParams;
