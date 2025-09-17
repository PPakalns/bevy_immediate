use bevy_ecs::{
    archetype::Archetype,
    query::{QueryData, ReadOnlyQueryData, With},
    system::{Query, QueryLens, QueryParamBuilder, SystemMeta, SystemParam, SystemParamBuilder},
    world::World,
};

use crate::{CapAccessRequestsResource, ImmCap, ImmMarker};

/// [`SystemParam`] for immediate mode capability requests [`ImmCap`]
pub struct CapSystemParams<'w, 's, Cap: ImmCap> {
    query: Query<'w, 's, (), With<ImmMarker<Cap>>>,
}
impl<'w, 's, Cap: ImmCap> CapSystemParams<'w, 's, Cap> {
    /// Get query with given [`ReadOnlyQueryData`]
    pub fn get_query<D>(&self) -> QueryLens<'_, D, With<ImmMarker<Cap>>>
    where
        D: ReadOnlyQueryData,
    {
        self.query.as_readonly().transmute_lens_filtered_inner()
    }

    /// Get query with given [`QueryData`]
    pub fn get_query_mut<D>(&mut self) -> QueryLens<'_, D, With<ImmMarker<Cap>>>
    where
        D: QueryData,
    {
        self.query.transmute_lens_filtered()
    }
}

#[expect(unsafe_code)]
unsafe impl<Cap: ImmCap> SystemParam for CapSystemParams<'_, '_, Cap> {
    type State = ImmCapSystemParamsState<Cap>;
    type Item<'world, 'state> = CapSystemParams<'world, 'state, Cap>;

    fn init_state(
        world: &mut World,
        system_meta: &mut bevy_ecs::system::SystemMeta,
    ) -> Self::State {
        let requested_access = world
            .get_resource::<CapAccessRequestsResource<Cap>>()
            .expect("bevy_immediate mode plugin not correctly added");
        let requested_access = requested_access.capabilities.clone();

        let params = QueryParamBuilder::new::<(), With<ImmMarker<Cap>>>(|builder| {
            for (component_id, mutability) in requested_access.requested_components().iter() {
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
