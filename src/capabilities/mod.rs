/// Marks types that implement immediate mode capabilities
pub trait ImmCap: Send + Sync + 'static {
    /// Function used to initialize necessary resources for capability to fully function
    fn build<Cap: ImmCap>(app: &mut bevy_app::App, cap_req: &mut ImmCapAccessRequests<Cap>);
}

impl ImmCap for () {
    fn build<Cap: ImmCap>(_app: &mut bevy_app::App, _cap_req: &mut ImmCapAccessRequests<Cap>) {}
}

/// Trait that marks what capabilities current capability implements
///
/// Capability can implement many sub-capabilities
pub trait ImplCap<T>: ImmCap {}
impl<T: ImmCap> ImplCap<T> for T {}

/// Implements list of capabilities for given type
///
/// ```no_run
/// pub struct CapMy;
///
/// impl_capabilities!(CapMy, (Cap1, Cap2, Cap3));
/// ````
///
#[macro_export]
macro_rules! impl_capabilities {
    ($name:ty, ($($t:ty),+ $(,)?)) => {
        impl $crate::ImmCap for $name {
            fn build<Cap: $crate::ImmCap>(
                app: &mut bevy_app::App,
                cap_req: &mut $crate::ImmCapAccessRequests<Cap>,
            ) {
                $(<$t as $crate::ImmCap>::build(app, cap_req);)+
            }
        }

        $(
            impl<T: $crate::ImplCap<$name>> $crate::ImplCap<$t> for T {}
        )+
    };
}

/// Implements logic for collecting requested components and resources
mod access_requests;
pub use access_requests::{ImmCapAccessRequests, ImmCapAccessRequestsResource};

/// Implements [`bevy_ecs::system::SystemParam`] for [`CapSystemParams`] that
/// allows to retrieve all requested data by capabilities
mod system_param;
pub use system_param::{ImmCapQueryParam, ImmCapResourcesParam};
