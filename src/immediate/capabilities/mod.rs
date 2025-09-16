use std::{any::TypeId, marker::PhantomData};

use bevy_ecs::{
    archetype::Archetype,
    component::{Component, ComponentId},
    query::{QueryData, With},
    system::{Query, QueryLens, QueryParamBuilder, SystemMeta, SystemParam, SystemParamBuilder},
    world::World,
};
use bevy_platform::collections::HashMap;

use crate::ImmMarker;

/// Marks types that implement immediate mode capabilities
pub trait ImmCap: Send + Sync + 'static {
    /// Function used to initialize necessary resources for capability to fully function
    fn build<Cap: ImmCap>(app: &mut bevy_app::App, cap_req: &mut ImmCapabilitiesRequests<Cap>);
}

/// Trait that marks what capabilities current capability implements
///
/// Capability can implement many sub-capabilities
pub trait ImmImplCap<T> {}
impl<T: ImmCap> ImmImplCap<T> for T {}

impl ImmCap for () {
    fn build<Cap: ImmCap>(app: &mut bevy_app::App, cap_req: &mut ImmCapabilitiesRequests<Cap>) {
        let _ = cap_req;
        let _ = app;
    }
}

/// Tracks what kind of query accesses capability has requested
#[derive(bevy_ecs::resource::Resource)]
pub struct ImmCapabilitiesRequests<Cap: ImmCap> {
    type_id_map: HashMap<TypeId, ComponentId>,
    components: HashMap<ComponentId, bool>,
    _ph: PhantomData<Cap>,
}

impl<Cap: ImmCap> Default for ImmCapabilitiesRequests<Cap> {
    fn default() -> Self {
        Self {
            type_id_map: Default::default(),
            components: Default::default(),
            _ph: Default::default(),
        }
    }
}

impl<Cap: ImmCap> ImmCapabilitiesRequests<Cap> {
    /// Mark component for retrieval during immediate mode execution through context [`crate::ImmCtx`]
    /// and [`crate::ImmCapSystemParams::get_query`] method by querying `Option<&Component>` or
    /// `Option<&mut Component>` based on `mutable` argument value.
    pub fn request_optional_component<C: Component>(&mut self, world: &mut World, mutable: bool) {
        let component_id = world.register_component::<C>();
        self.type_id_map.insert(TypeId::of::<C>(), component_id);
        let value = self.components.entry(component_id).or_default();
        *value |= mutable;
    }
}

/// [`SystemParam`] for immediate mode capability requests [`ImmCap`]
pub struct ImmCapSystemParams<'w, 's, Cap: ImmCap> {
    query: Query<'w, 's, (), With<ImmMarker<Cap>>>,
}
impl<'w, 's, Cap: ImmCap> ImmCapSystemParams<'w, 's, Cap> {
    /// Get query with given [`QueryData`]
    pub fn get_query<D>(&mut self) -> QueryLens<'_, D, With<ImmMarker<Cap>>>
    where
        D: QueryData,
    {
        self.query.transmute_lens_filtered()
    }
}

#[expect(unsafe_code)]
unsafe impl<Cap: ImmCap> SystemParam for ImmCapSystemParams<'_, '_, Cap> {
    type State = ImmCapSystemParamsState<Cap>;
    type Item<'world, 'state> = ImmCapSystemParams<'world, 'state, Cap>;

    fn init_state(
        world: &mut World,
        system_meta: &mut bevy_ecs::system::SystemMeta,
    ) -> Self::State {
        let requested_access = world
            .get_resource::<ImmCapabilitiesRequests<Cap>>()
            .expect("bevy_immediate mode plugin not correctly added");
        let requested_access =
            Vec::from_iter(requested_access.components.iter().map(|(k, v)| (*k, *v)));

        let params = QueryParamBuilder::new::<(), With<ImmMarker<Cap>>>(|builder| {
            for (component_id, mutability) in requested_access.iter() {
                builder.optional(|query_builder| match mutability {
                    true => {
                        query_builder.mut_id(*component_id);
                    }
                    false => {
                        query_builder.mut_id(*component_id);
                    }
                });
            }
        });

        let query_state = params.build(world, system_meta);

        ImmCapSystemParamsState { query_state }
    }

    unsafe fn new_archetype(
        state: &mut Self::State,
        archetype: &Archetype,
        system_meta: &mut SystemMeta,
    ) {
        unsafe { Query::new_archetype(&mut state.query_state, archetype, system_meta) };
    }

    unsafe fn get_param<'world, 'state>(
        state: &'state mut Self::State,
        system_meta: &bevy_ecs::system::SystemMeta,
        world: bevy_ecs::world::unsafe_world_cell::UnsafeWorldCell<'world>,
        change_tick: bevy_ecs::component::Tick,
    ) -> Self::Item<'world, 'state> {
        let query =
            unsafe { Query::get_param(&mut state.query_state, system_meta, world, change_tick) };

        Self::Item::<'world, 'state> { query }
    }
}

pub struct ImmCapSystemParamsState<Cap: ImmCap> {
    query_state: bevy_ecs::query::QueryState<(), With<ImmMarker<Cap>>>,
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
                cap_req: &mut $crate::ImmCapabilitiesRequests<Cap>,
            ) {
                $(<$t as $crate::ImmCap>::build(app, cap_req);)+
            }
        }

        $(
            impl<T: ImmImplCap<$name>> ImmImplCap<$t> for T {}
        )+
    };
}
