use std::marker::PhantomData;

use bevy_ecs::{
    archetype::Archetype,
    query::With,
    system::{
        FilteredResourcesMutParamBuilder, Query, QueryParamBuilder, SystemMeta, SystemParam,
        SystemParamBuilder,
    },
    world::{FilteredEntityMut, FilteredResourcesMut, World},
};

use crate::{
    CapAccessRequestsResource, CapQueryParam, ImmCap, ImmMarker, capabilities::CapResourcesParam,
};

#[expect(unsafe_code)]
unsafe impl<Cap: ImmCap> SystemParam for CapQueryParam<'_, '_, Cap> {
    type State = CapQueryState<Cap>;
    type Item<'world, 'state> = CapQueryParam<'world, 'state, Cap>;

    fn init_state(
        world: &mut World,
        system_meta: &mut bevy_ecs::system::SystemMeta,
    ) -> Self::State {
        let requested_access = world
            .get_resource::<CapAccessRequestsResource<Cap>>()
            .expect("bevy_immediate mode plugin not correctly added");
        let requested_access = requested_access.capabilities.clone();

        let params = QueryParamBuilder::new::<FilteredEntityMut, With<ImmMarker<Cap>>>(|builder| {
            builder.with::<ImmMarker<Cap>>();

            for (_component_id, request) in requested_access.requested_components().iter() {
                (request.builder)(builder, request.mutable);
            }
        });

        let query_state = params.build(world, system_meta);

        CapQueryState { state: query_state }
    }

    unsafe fn new_archetype(
        state: &mut Self::State,
        archetype: &Archetype,
        system_meta: &mut SystemMeta,
    ) {
        unsafe { Query::new_archetype(&mut state.state, archetype, system_meta) };
    }

    unsafe fn get_param<'world, 'state>(
        state: &'state mut Self::State,
        system_meta: &bevy_ecs::system::SystemMeta,
        world: bevy_ecs::world::unsafe_world_cell::UnsafeWorldCell<'world>,
        change_tick: bevy_ecs::component::Tick,
    ) -> Self::Item<'world, 'state> {
        let query = unsafe { Query::get_param(&mut state.state, system_meta, world, change_tick) };

        Self::Item::<'world, 'state> { query }
    }

    fn apply(state: &mut Self::State, system_meta: &SystemMeta, world: &mut World) {
        Query::apply(&mut state.state, system_meta, world)
    }

    fn queue(
        state: &mut Self::State,
        system_meta: &SystemMeta,
        world: bevy_ecs::world::DeferredWorld,
    ) {
        Query::queue(&mut state.state, system_meta, world)
    }

    unsafe fn validate_param(
        state: &Self::State,
        system_meta: &SystemMeta,
        world: bevy_ecs::world::unsafe_world_cell::UnsafeWorldCell,
    ) -> Result<(), bevy_ecs::system::SystemParamValidationError> {
        unsafe { Query::validate_param(&state.state, system_meta, world) }
    }
}

pub struct CapQueryState<Cap: ImmCap> {
    state: bevy_ecs::query::QueryState<FilteredEntityMut<'static>, With<ImmMarker<Cap>>>,
}

#[expect(unsafe_code)]
unsafe impl<Cap: ImmCap> SystemParam for CapResourcesParam<'_, '_, Cap> {
    type State = CapResourceState<Cap>;
    type Item<'world, 'state> = CapResourcesParam<'world, 'state, Cap>;

    fn init_state(
        world: &mut World,
        system_meta: &mut bevy_ecs::system::SystemMeta,
    ) -> Self::State {
        let requested_access = world
            .get_resource::<CapAccessRequestsResource<Cap>>()
            .expect("bevy_immediate mode plugin not correctly added");
        let requested_access = requested_access.capabilities.clone();

        let builder = FilteredResourcesMutParamBuilder::new(|builder| {
            for res in requested_access.requested_resources().values() {
                (res.builder)(builder, res.mutable);
            }
        });
        let state = builder.build(world, system_meta);

        CapResourceState {
            access: state,
            _ph: PhantomData,
        }
    }

    unsafe fn get_param<'world, 'state>(
        state: &'state mut Self::State,
        system_meta: &bevy_ecs::system::SystemMeta,
        world: bevy_ecs::world::unsafe_world_cell::UnsafeWorldCell<'world>,
        change_tick: bevy_ecs::component::Tick,
    ) -> Self::Item<'world, 'state> {
        let resources = unsafe {
            FilteredResourcesMut::get_param(&mut state.access, system_meta, world, change_tick)
        };

        Self::Item::<'world, 'state> {
            resources,
            _ph: PhantomData,
        }
    }

    unsafe fn new_archetype(
        state: &mut Self::State,
        archetype: &Archetype,
        system_meta: &mut SystemMeta,
    ) {
        unsafe { FilteredResourcesMut::new_archetype(&mut state.access, archetype, system_meta) }
    }

    fn apply(state: &mut Self::State, system_meta: &SystemMeta, world: &mut World) {
        FilteredResourcesMut::apply(&mut state.access, system_meta, world)
    }

    fn queue(
        state: &mut Self::State,
        system_meta: &SystemMeta,
        world: bevy_ecs::world::DeferredWorld,
    ) {
        FilteredResourcesMut::queue(&mut state.access, system_meta, world)
    }

    unsafe fn validate_param(
        state: &Self::State,
        system_meta: &SystemMeta,
        world: bevy_ecs::world::unsafe_world_cell::UnsafeWorldCell,
    ) -> Result<(), bevy_ecs::system::SystemParamValidationError> {
        unsafe { FilteredResourcesMut::validate_param(&state.access, system_meta, world) }
    }
}

pub struct CapResourceState<Cap: ImmCap> {
    _ph: PhantomData<Cap>,
    access: bevy_ecs::query::Access<bevy_ecs::component::ComponentId>,
}
