use std::marker::PhantomData;

use bevy_ecs::{
    component::Components,
    query::{With, Without},
    resource::{IsResource, ResourceEntities},
    system::{
        Query, QueryParamBuilder, SystemMeta, SystemParam, SystemParamBuilder,
        SystemParamValidationError,
    },
    world::{FilteredEntityMut, World},
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

        let params = QueryParamBuilder::new::<
            FilteredEntityMut,
            (With<ImmMarker<Caps>>, Without<IsResource>),
        >(|builder| {
            builder.with::<ImmMarker<Caps>>();
            builder.without::<IsResource>();

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
        system_access: &mut bevy_ecs::system::SystemAccess,
        world: &mut World,
    ) {
        Query::init_access(&state.state, system_meta, system_access, world)
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

    unsafe fn get_param<'world, 'state>(
        state: &'state mut Self::State,
        system_meta: &bevy_ecs::system::SystemMeta,
        world: bevy_ecs::world::unsafe_world_cell::UnsafeWorldCell<'world>,
        change_tick: bevy_ecs::change_detection::Tick,
    ) -> Result<Self::Item<'world, 'state>, SystemParamValidationError> {
        let query = unsafe { Query::get_param(&mut state.state, system_meta, world, change_tick) }?;

        Ok(Self::Item::<'world, 'state> { query })
    }
}

pub struct CapQueryState<Caps: CapSet> {
    state: bevy_ecs::query::QueryState<
        FilteredEntityMut<'static, 'static>,
        (With<ImmMarker<Caps>>, Without<IsResource>),
    >,
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

        let params = QueryParamBuilder::new::<FilteredEntityMut, With<IsResource>>(|builder| {
            builder.with::<IsResource>();

            for (&component_id, res) in requested_access.requested_resources().iter() {
                builder.optional(|builder| match res.mutable {
                    true => {
                        builder.mut_id(component_id);
                    }
                    false => {
                        builder.ref_id(component_id);
                    }
                });
            }
        });

        CapResourceState {
            state: params.build(world),
            _ph: PhantomData,
        }
    }

    unsafe fn get_param<'world, 'state>(
        state: &'state mut Self::State,
        system_meta: &bevy_ecs::system::SystemMeta,
        world: bevy_ecs::world::unsafe_world_cell::UnsafeWorldCell<'world>,
        change_tick: bevy_ecs::change_detection::Tick,
    ) -> Result<Self::Item<'world, 'state>, SystemParamValidationError> {
        let query = unsafe { Query::get_param(&mut state.state, system_meta, world, change_tick) }?;

        Ok(Self::Item::<'world, 'state> {
            query,
            // SAFETY: SystemParam init_access requested shared access to ResourceEntities and Components
            resource_entities: unsafe { world.resource_entities() },
            components: world.components(),
            _ph: PhantomData,
        })
    }

    fn apply(state: &mut Self::State, system_meta: &SystemMeta, world: &mut World) {
        Query::apply(&mut state.state, system_meta, world);
        <&ResourceEntities as SystemParam>::apply(&mut (), system_meta, world);
        <&Components as SystemParam>::apply(&mut (), system_meta, world);
    }

    fn queue(
        state: &mut Self::State,
        system_meta: &SystemMeta,
        mut world: bevy_ecs::world::DeferredWorld,
    ) {
        Query::queue(&mut state.state, system_meta, world.reborrow());
        <&ResourceEntities as SystemParam>::queue(&mut (), system_meta, world.reborrow());
        <&Components as SystemParam>::queue(&mut (), system_meta, world);
    }

    fn init_access(
        state: &Self::State,
        system_meta: &mut SystemMeta,
        system_access: &mut bevy_ecs::system::SystemAccess,
        world: &mut World,
    ) {
        Query::init_access(&state.state, system_meta, system_access, world);
        // `get_param` also borrows `ResourceEntities` and `Components` from the world
        <&ResourceEntities as SystemParam>::init_access(&(), system_meta, system_access, world);
        <&Components as SystemParam>::init_access(&(), system_meta, system_access, world);
    }
}

pub struct CapResourceState<Caps: CapSet> {
    _ph: PhantomData<Caps>,
    state: bevy_ecs::query::QueryState<FilteredEntityMut<'static, 'static>, With<IsResource>>,
}
