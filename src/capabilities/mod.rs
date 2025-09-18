/// Marks types that implement immediate mode capabilities
pub trait ImmCap: Send + Sync + 'static {
    /// Function used to initialize necessary resources for capability to fully function
    fn build<Cap: ImmCap>(app: &mut bevy_app::App, cap_req: &mut ImmCapAccessRequests<Cap>);
}

impl ImmCap for () {
    fn build<Cap: ImmCap>(_app: &mut bevy_app::App, _cap_req: &mut ImmCapAccessRequests<Cap>) {}
}
impl<Head, Tail> ImmCap for (Head, Tail)
where
    Head: ImmCap,
    Tail: ImmCap + TupleList,
{
    fn build<Cap: ImmCap>(app: &mut bevy_app::App, cap_req: &mut ImmCapAccessRequests<Cap>) {
        Head::build(app, cap_req);
        Tail::build(app, cap_req);
    }
}

/// Trait that marks what capabilities current capability implements
///
/// Capability can implement many sub-capabilities
pub trait ImplCap<T>: ImmCap {}
impl<Head, Tail> ImplCap<Head> for (Head, Tail)
where
    Head: ImmCap,
    Tail: TupleList + ImmCap,
{
}

impl<T, Head, Tail> ImplCap<T> for (Head, Tail)
where
    Head: ImmCap,
    Tail: TupleList + ImplCap<T>,
{
}

/// Implements list of capabilities for given type
///
/// ```no_run
/// pub struct CapMy;
///
/// merge_capabilities!(CapMy, (Cap1, Cap2, Cap3));
/// ````
///
#[macro_export]
macro_rules! merge_capabilities {
    ($name:ident, ($($t:ty),+ $(,)?)) => {
        #[doc = "Capability"]
        pub type $name = merge_capabilities!(- $($t),+);
    };
    (- $t1:ty$(,$t2:ty)+) => {
        ($t1, merge_capabilities!(- $($t2),+))
    };
    (- $t1:ty) => {
        ($t1, ())
    };
}

/// Implements logic for collecting requested components and resources
mod access_requests;
pub use access_requests::{ImmCapAccessRequests, ImmCapAccessRequestsResource};

/// Implements [`bevy_ecs::system::SystemParam`] for [`CapSystemParams`] that
/// allows to retrieve all requested data by capabilities
mod system_param;
pub use system_param::{ImmCapQueryParam, ImmCapResourcesParam};
use tuple_list::TupleList;
