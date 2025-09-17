use std::{any::TypeId, marker::PhantomData, sync::Arc};

use bevy_ecs::{
    component::{Component, ComponentId},
    resource::Resource,
    system::{DynParamBuilder, ParamBuilder},
    world::World,
};
use bevy_platform::collections::HashMap;

use crate::ImmCap;

/// Stores requested capabilities for given immediate mode request
#[derive(bevy_ecs::resource::Resource)]
pub struct CapAccessRequestsResource<Cap: ImmCap> {
    /// Information about access requests issued by provided `Cap`
    pub capabilities: Arc<CapAccessRequests<Cap>>,
}

impl<Cap: ImmCap> CapAccessRequestsResource<Cap> {
    pub(crate) fn new(capabilities: CapAccessRequests<Cap>) -> Self {
        Self {
            capabilities: Arc::new(capabilities),
        }
    }
}

/// Tracks what kind of query accesses capability has requested
pub struct CapAccessRequests<Cap: ImmCap> {
    // type_id_map: HashMap<TypeId, ComponentId>,
    components: HashMap<ComponentId, bool>,
    resources: HashMap<TypeId, ResourceRequest>,
    _ph: PhantomData<Cap>,
}

pub struct ResourceRequest {
    pub mutable: bool,
    pub builder: Box<dyn Fn(bool) -> DynParamBuilder<'static> + 'static + Send + Sync>,
}

impl<Cap: ImmCap> Default for CapAccessRequests<Cap> {
    fn default() -> Self {
        Self {
            // type_id_map: Default::default(),
            components: Default::default(),
            resources: Default::default(),
            _ph: Default::default(),
        }
    }
}

impl<Cap: ImmCap> CapAccessRequests<Cap> {
    /// Mark component for retrieval during immediate mode execution through context [`crate::ImmCtx`]
    /// and [`crate::ImmCapSystemParams::get_query`] method by querying `Option<&Component>` or
    /// `Option<&mut Component>` based on `mutable` argument value.
    pub fn request_optional_component<C: Component>(&mut self, world: &mut World, mutable: bool) {
        let component_id = world.register_component::<C>();
        // self.type_id_map.insert(TypeId::of::<C>(), component_id);
        let value = self.components.entry(component_id).or_default();
        *value |= mutable;
    }

    /// Mark resource for retrieval during immediate mode execution through context [`crate::ImmCtx`]
    /// and [`crate::ImmCapSystemParams::get_resource`] methods.
    pub fn request_resource<R: Resource>(&mut self, mutable: bool) {
        let value = self
            .resources
            .entry(TypeId::of::<R>())
            .or_insert_with(|| ResourceRequest {
                mutable,
                builder: Box::new(|mutable| {
                    if mutable {
                        let builder = ParamBuilder::resource_mut::<R>();
                        DynParamBuilder::new(builder)
                    } else {
                        let builder = ParamBuilder::resource::<R>();
                        DynParamBuilder::new(builder)
                    }
                }),
            });
        value.mutable |= mutable;
    }

    /// Returns requested component id and their mutability
    pub fn requested_components(&self) -> &HashMap<ComponentId, bool> {
        &self.components
    }

    /// Returns component_ids for requested resources and their mutability
    pub fn requested_resoruces(&self) -> &HashMap<TypeId, ResourceRequest> {
        &self.resources
    }
}
