use std::marker::PhantomData;

use bevy_ecs::{
    query::With,
    system::{
        FilteredResourcesMutParamBuilder, Query, QueryParamBuilder, SystemMeta, SystemParam,
        SystemParamBuilder,
    },
    world::{FilteredEntityMut, FilteredResourcesMut, World},
};

use crate::{
    CapSet, ImmCapAccessRequestsResource, ImmCapQueryParam, ImmMarker,
    capabilities::ImmCapResourcesParam,
};

#[expect(unsafe_code)]
unsafe impl<Caps: CapSet> SystemParam for ImmCapQueryParam<'_, '_, Caps> {
    type State = CapQueryState<Caps>;
    type Item<'world, 'state> = ImmCapQueryParam<'world, 'state, Caps>;

    fn init_state(world: &mut World) -> Self::State {
        let requested_access = world
            .get_resource::<ImmCapAccessRequestsResource<Caps>>()
            .expect("bevy_immediate mode plugin not correctly added");
        let requested_access = requested_access.capabilities.clone();

        let params =
            QueryParamBuilder::new::<FilteredEntityMut, With<ImmMarker<Caps>>>(|builder| {
                builder.with::<ImmMarker<Caps>>();

                for (&component_id, request) in requested_access.requested_components().iter() {
                    builder.optional(|builder| match request.mutable {
                        true => {
                            builder.mut_id(component_id);
                        }
                        false => {
                            builder.ref_id(component_id);
                        }
                    });
                }
            });

        let query_state = params.build(world);

        CapQueryState { state: query_state }
    }

    fn init_access(
        state: &Self::State,
        system_meta: &mut SystemMeta,
        component_access_set: &mut bevy_ecs::query::FilteredAccessSet,
        world: &mut World,
    ) {
        Query::init_access(&state.state, system_meta, component_access_set, world)
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
        state: &mut Self::State,
        system_meta: &SystemMeta,
        world: bevy_ecs::world::unsafe_world_cell::UnsafeWorldCell,
    ) -> Result<(), bevy_ecs::system::SystemParamValidationError> {
        unsafe { Query::validate_param(&mut state.state, system_meta, world) }
    }

    unsafe fn get_param<'world, 'state>(
        state: &'state mut Self::State,
        system_meta: &bevy_ecs::system::SystemMeta,
        world: bevy_ecs::world::unsafe_world_cell::UnsafeWorldCell<'world>,
        change_tick: bevy_ecs::change_detection::Tick,
    ) -> Self::Item<'world, 'state> {
        let query = unsafe { Query::get_param(&mut state.state, system_meta, world, change_tick) };

        Self::Item::<'world, 'state> { query }
    }
}

pub struct CapQueryState<Caps: CapSet> {
    state: bevy_ecs::query::QueryState<FilteredEntityMut<'static, 'static>, With<ImmMarker<Caps>>>,
}

#[expect(unsafe_code)]
unsafe impl<Caps: CapSet> SystemParam for ImmCapResourcesParam<'_, '_, Caps> {
    type State = CapResourceState<Caps>;
    type Item<'world, 'state> = ImmCapResourcesParam<'world, 'state, Caps>;

    fn init_state(world: &mut World) -> Self::State {
        let requested_access = world
            .get_resource::<ImmCapAccessRequestsResource<Caps>>()
            .expect("bevy_immediate mode plugin not correctly added");
        let requested_access = requested_access.capabilities.clone();

        let builder = FilteredResourcesMutParamBuilder::new(|builder| {
            for (&component_id, res) in requested_access.requested_resources().iter() {
                match res.mutable {
                    true => {
                        builder.add_write_by_id(component_id);
                    }
                    false => {
                        builder.add_read_by_id(component_id);
                    }
                }
            }
        });
        let state = builder.build(world);

        CapResourceState {
            access: state,
            _ph: PhantomData,
        }
    }

    unsafe fn get_param<'world, 'state>(
        state: &'state mut Self::State,
        system_meta: &bevy_ecs::system::SystemMeta,
        world: bevy_ecs::world::unsafe_world_cell::UnsafeWorldCell<'world>,
        change_tick: bevy_ecs::change_detection::Tick,
    ) -> Self::Item<'world, 'state> {
        let resources = unsafe {
            FilteredResourcesMut::get_param(&mut state.access, system_meta, world, change_tick)
        };

        Self::Item::<'world, 'state> {
            resources,
            _ph: PhantomData,
        }
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
        state: &mut Self::State,
        system_meta: &SystemMeta,
        world: bevy_ecs::world::unsafe_world_cell::UnsafeWorldCell,
    ) -> Result<(), bevy_ecs::system::SystemParamValidationError> {
        unsafe { FilteredResourcesMut::validate_param(&mut state.access, system_meta, world) }
    }

    fn init_access(
        state: &Self::State,
        system_meta: &mut SystemMeta,
        component_access_set: &mut bevy_ecs::query::FilteredAccessSet,
        world: &mut World,
    ) {
        FilteredResourcesMut::init_access(&state.access, system_meta, component_access_set, world)
    }
}

pub struct CapResourceState<Caps: CapSet> {
    _ph: PhantomData<Caps>,
    access: bevy_ecs::query::Access,
}
