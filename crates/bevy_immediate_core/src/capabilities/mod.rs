/// Type implement support for set of [ImmCapability]
///
/// Marks types that can be used inside [`crate::ImmCtx`]
pub trait CapSet: Send + Sync + 'static {
    /// Logic to initialize all capabilities
    fn initialize<Caps: CapSet>(app: &mut bevy_app::App, cap_req: &mut ImmCapAccessRequests<Caps>);
}

/// Marks types that are used to implement immediate mode capabilities
pub trait ImmCapability: Send + Sync + 'static {
    /// Function used to initialize necessary resources for capability to fully function
    fn build<Caps: CapSet>(app: &mut bevy_app::App, cap_req: &mut ImmCapAccessRequests<Caps>);
}

/// Trait that marks that CapSet implements given capability implementation
///
/// Capability can implement many sub-capabilities
pub trait ImplCap<T>: CapSet {}

/// Implements list of capabilities for given type
///
/// ```ignore
/// pub struct CapMy;
///
/// bevy_immediate::impl_capability_set!(CapMy, ImplCapMy > ImplChildSet, (Cap1, Cap2, Cap3));
/// ````
///
/// Defines trait `ImplCapMy` that can be used to easily check that CapSet has
/// at least all capabilities in `CapMy`.
///
/// Additionally at compile time checks that `ImplChildSet` is subset of `CapMy` capabilities.
///
/// See examples for correct use!
#[macro_export]
macro_rules! impl_capability_set {
    ($name:ty, $set_trait:ident > $subset_check:ident, ($($t:ty),+ $(,)?)) => {
        impl $crate::CapSet for $name {
            fn initialize<Caps: $crate::CapSet>(
                app: &mut $crate::bevy_app::App,
                cap_req: &mut $crate::ImmCapAccessRequests<Caps>,
            ) {
                $(<$t as $crate::ImmCapability>::build(app, cap_req);)+
            }
        }

        #[doc = "Trait to check for set of capabilities to be implemented"]
        pub trait $set_trait: $crate::CapSet $(+ $crate::ImplCap<$t>)+ {}
        impl<T> $set_trait for T
        where T: $crate::CapSet $(+ $crate::ImplCap<$t>)+
        {}

        $(
            impl $crate::ImplCap<$t> for $name {}
        )+

        $crate::paste::paste!{
            #[allow(warnings)]
            fn [<_subset_check_for $set_trait>]<T: $subset_check>() {}

            #[allow(warnings)]
            fn [< _check_ $set_trait _implements_at_least_provided_subset $subset_check >]() {
                [<_subset_check_for $set_trait>]::<$name>();
            }
        }
    };
}

/// Manualy implement empty capability set
impl CapSet for () {
    fn initialize<Caps: CapSet>(
        _app: &mut bevy_app::App,
        _cap_req: &mut ImmCapAccessRequests<Caps>,
    ) {
    }
}

/// All capability sets implement that they implement support for empty capability set
pub trait ImplCapsEmpty: CapSet {}
impl<T: CapSet> ImplCapsEmpty for T {}
impl<T: CapSet> ImplCap<()> for T {}

impl ImmCapability for () {
    fn build<Caps: CapSet>(_app: &mut bevy_app::App, _cap_req: &mut ImmCapAccessRequests<Caps>) {}
}

/// Implements logic for collecting requested components and resources
mod access_requests;
pub use access_requests::{ImmCapAccessRequests, ImmCapAccessRequestsResource};

/// Implements [`bevy_ecs::system::SystemParam`] for [`CapSystemParams`] that
/// allows to retrieve all requested data by capabilities
mod system_param;
pub use system_param::{ImmCapQueryParam, ImmCapResourcesParam};
