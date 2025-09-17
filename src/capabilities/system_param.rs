use std::{any::TypeId, marker::PhantomData};

use bevy_ecs::{
    archetype::Archetype,
    entity::Entity,
    query::{QueryData, ReadOnlyQueryData, With},
    resource::Resource,
    system::{
        DynSystemParam, DynSystemParamState, Query, QueryLens, QueryParamBuilder, Res, ResMut,
        SystemMeta, SystemParam, SystemParamBuilder,
    },
    world::World,
};
use bevy_platform::collections::HashMap;

use crate::{CapAccessRequestsResource, ImmCap, ImmMarker};

/// [`SystemParam`] for immediate mode capability requests [`ImmCap`]
pub struct CapQueryParam<'w, 's, Cap: ImmCap> {
    query:
        Query<'w, 's, ImmediateModeCapabilityRequestedComponentDynamicQuery, With<ImmMarker<Cap>>>,
}

/// Type for better error messages upon incorrectly requesting component access
#[derive(bevy_ecs::query::QueryData)]
pub struct ImmediateModeCapabilityRequestedComponentDynamicQuery {
    entity: Entity,
}

impl<'w, 's, Cap: ImmCap> CapQueryParam<'w, 's, Cap> {
    /// Get query with given [`ReadOnlyQueryData`]
    /// You will probably want to call `.query()` on returned value.
    pub fn get_query<D>(&self) -> QueryLens<'_, D, With<ImmMarker<Cap>>>
    where
        D: ReadOnlyQueryData,
    {
        self.query.as_readonly().transmute_lens_filtered_inner()
    }

    /// Get query with given [`QueryData`]
    /// You will probably want to call `.query()` on returned value.
    pub fn get_query_mut<D>(&mut self) -> QueryLens<'_, D, With<ImmMarker<Cap>>>
    where
        D: QueryData,
    {
        self.query.transmute_lens_filtered()
    }

    /// Underlaying query that can be used to access components queried by
    pub fn query(
        &self,
    ) -> &Query<'w, 's, ImmediateModeCapabilityRequestedComponentDynamicQuery, With<ImmMarker<Cap>>>
    {
        &self.query
    }

    /// Underlaying query that can be used to access components queried by
    pub fn query_mut(
        &mut self,
    ) -> &mut Query<
        'w,
        's,
        ImmediateModeCapabilityRequestedComponentDynamicQuery,
        With<ImmMarker<Cap>>,
    > {
        &mut self.query
    }
}

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

        let params = QueryParamBuilder::new::<
            ImmediateModeCapabilityRequestedComponentDynamicQuery,
            With<ImmMarker<Cap>>,
        >(|builder| {
            for (component_id, mutable) in requested_access.requested_components().iter() {
                builder.optional(|query_builder| match mutable {
                    true => {
                        query_builder.mut_id(*component_id);
                    }
                    false => {
                        query_builder.ref_id(*component_id);
                    }
                });
            }
        });

        let query_state = params.build(world, system_meta);

        CapQueryState { query_state }
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

pub struct CapQueryState<Cap: ImmCap> {
    query_state: bevy_ecs::query::QueryState<
        ImmediateModeCapabilityRequestedComponentDynamicQuery,
        With<ImmMarker<Cap>>,
    >,
}

/// [`SystemParam`] for immediate mode capability requests [`ImmCap`]
pub struct CapResourcesParam<'w, 's, Cap: ImmCap> {
    resources: HashMap<TypeId, ResourceAccess<'w, 's>>,
    _ph: PhantomData<Cap>,
}

impl<'w, 's, Cap: ImmCap> CapResourcesParam<'w, 's, Cap> {
    pub fn resource_mut<R: Resource>(&mut self) -> ResMut<'_, R> {
        let type_id = TypeId::of::<R>();
        let resource = self
            .resources
            .get_mut(&type_id)
            .expect("Resource not added to capabilities");
        if !resource.mutable {
            panic!("Resource not mutably added to capabilities");
        }

        // All resources should be correctly resolved
        resource.state.downcast_mut::<ResMut<R>>().unwrap()
    }

    pub fn with_resource<R: Resource, O>(&mut self, mut f: impl FnMut(&R) -> O) -> O {
        let type_id = TypeId::of::<R>();
        let resource = self
            .resources
            .get_mut(&type_id)
            .expect("Resource not added to capabilities");

        // All resources should be correctly resolved
        if resource.mutable {
            f(&resource.state.downcast_mut::<ResMut<R>>().unwrap())
        } else {
            f(&resource.state.downcast_mut::<Res<R>>().unwrap())
        }
    }
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

        let resource_states = requested_access
            .requested_resoruces()
            .iter()
            .map(|(id, res)| {
                let builder = (res.builder)(res.mutable);

                let state = builder.build(world, system_meta);

                (
                    *id,
                    RequestedResourceState {
                        state,
                        mutable: res.mutable,
                    },
                )
            })
            .collect();

        CapResourceState {
            resource_states,
            _ph: PhantomData,
        }
    }

    unsafe fn new_archetype(
        state: &mut Self::State,
        archetype: &Archetype,
        system_meta: &mut SystemMeta,
    ) {
        state
            .resource_states
            .iter_mut()
            .for_each(|(_, state)| unsafe {
                DynSystemParam::new_archetype(&mut state.state, archetype, system_meta)
            });
    }

    unsafe fn get_param<'world, 'state>(
        state: &'state mut Self::State,
        system_meta: &bevy_ecs::system::SystemMeta,
        world: bevy_ecs::world::unsafe_world_cell::UnsafeWorldCell<'world>,
        change_tick: bevy_ecs::component::Tick,
    ) -> Self::Item<'world, 'state> {
        let resources = state
            .resource_states
            .iter_mut()
            .map(|(id, state)| unsafe {
                (
                    *id,
                    ResourceAccess {
                        state: DynSystemParam::get_param(
                            &mut state.state,
                            system_meta,
                            world,
                            change_tick,
                        ),
                        mutable: state.mutable,
                    },
                )
            })
            .collect();

        Self::Item::<'world, 'state> {
            resources,
            _ph: PhantomData,
        }
    }
}
struct RequestedResourceState {
    state: DynSystemParamState,
    mutable: bool,
}

pub struct CapResourceState<Cap: ImmCap> {
    resource_states: HashMap<TypeId, RequestedResourceState>,
    _ph: PhantomData<Cap>,
}

struct ResourceAccess<'w, 's> {
    state: DynSystemParam<'w, 's>,
    mutable: bool,
}
