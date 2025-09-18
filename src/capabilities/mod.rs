/// Marks types that implement immediate mode capabilities
pub trait ImmCap: UseAtYourOwnRisk_ImmSealed + Send + Sync + 'static {
    /// Function used to initialize necessary resources for capability to fully function
    fn build<Cap: ImmCap>(app: &mut bevy_app::App, cap_req: &mut ImmCapAccessRequests<Cap>);
}

impl ImmCap for () {
    fn build<Cap: ImmCap>(_app: &mut bevy_app::App, _cap_req: &mut ImmCapAccessRequests<Cap>) {}
}
impl UseAtYourOwnRisk_ImmSealed for () {}

/// Implements independent capability
#[macro_export]
macro_rules! impl_capability {
    ($name:ty) => {
        paste::paste! {
            #[doc="Trait for capabilities that implements at least $name capability"]
            pub trait [<Impl$name>] : $crate::UseAtYourOwnRisk_ImmSealed
            {}

            impl [<Impl$name>] for $name {}
            impl $crate::UseAtYourOwnRisk_ImmSealed for $name {
            }
        }
    };
}

/// Implements list of capabilities for given type
///
/// ```no_run
/// pub struct CapMy;
///
/// impl_capabilities!(CapMy, ((Cap1, ImplCap1), (Cap2, ImplCap2), (Cap3, ImplCap3)));
/// ````
///
#[macro_export]
macro_rules! impl_capabilities {
    ($name:ty, ($(($cap_type:ty, $cap_trait:ty)),+ $(,)?)) => {
        paste::paste!{
            impl $crate::ImmCap for $name {
                fn build<Cap: $crate::ImmCap>(
                    app: &mut bevy_app::App,
                    cap_req: &mut $crate::ImmCapAccessRequests<Cap>,
                ) {
                    $(<$cap_type as $crate::ImmCap>::build(app, cap_req);)+
                }
            }

            #[doc="Trait for capabilities that implements at least $name capability"]
            pub trait [<Impl$name>] : $crate::UseAtYourOwnRisk_ImmSealed $(+ $cap_trait)+
            {}

            impl [<Impl$name>] for $name {}
            impl $crate::UseAtYourOwnRisk_ImmSealed for $name {
            }

            $(
                impl<T> $cap_trait for T
                where T: [<Impl$name>] {}
            )+
        }
    };
}

/// Implements logic for collecting requested components and resources
mod access_requests;
pub use access_requests::{ImmCapAccessRequests, ImmCapAccessRequestsResource};

/// Implements [`bevy_ecs::system::SystemParam`] for [`CapSystemParams`] that
/// allows to retrieve all requested data by capabilities
mod system_param;
pub use system_param::{ImmCapQueryParam, ImmCapResourcesParam};

/// Use it at your own risk
///
/// Trait for helping programmers to correctly
/// implement their immediate mode logic with
/// capabilities that UI can access.
#[doc(hidden)]
#[allow(nonstandard_style)]
pub trait UseAtYourOwnRisk_ImmSealed {}
