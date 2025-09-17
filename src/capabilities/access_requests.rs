use std::{marker::PhantomData, sync::Arc};

use bevy_ecs::{
    component::{Component, ComponentId, Mutable},
    resource::Resource,
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
    components: HashMap<ComponentId, ComponentRequests>,
    resources: HashMap<ComponentId, ResourceRequest>,
    _ph: PhantomData<Cap>,
}

impl<Cap: ImmCap> Default for CapAccessRequests<Cap> {
    fn default() -> Self {
        Self {
            components: Default::default(),
            resources: Default::default(),
            _ph: PhantomData,
        }
    }
}

impl<Cap: ImmCap> CapAccessRequests<Cap> {
    /// Mark that component will be immutably accessed
    pub fn request_component_read<C: Component<Mutability = Mutable>>(
        &mut self,
        world: &mut World,
    ) {
        self.request_component_inner::<C>(world, false)
    }

    /// Mark that component will be mutably accessed
    pub fn request_component_write<C: Component<Mutability = Mutable>>(
        &mut self,
        world: &mut World,
    ) {
        self.request_component_inner::<C>(world, true)
    }

    /// Mark that resource will be read during immediate mode
    pub fn request_resource_read<R: Resource>(&mut self, world: &mut World) {
        self.request_resource_inner::<R>(world, false)
    }

    /// Mark that resource will be written during immediate mode
    pub fn request_resource_write<R: Resource>(&mut self, world: &mut World) {
        self.request_resource_inner::<R>(world, true)
    }

    fn request_component_inner<C: Component>(&mut self, world: &mut World, mutable: bool) {
        let component_id = world.register_component::<C>();
        // self.type_id_map.insert(TypeId::of::<C>(), component_id);
        let value = self
            .components
            .entry(component_id)
            .or_insert_with(|| ComponentRequests { mutable });
        value.mutable |= mutable;
    }

    fn request_resource_inner<R: Resource>(&mut self, world: &mut World, mutable: bool) {
        let component_id = world.register_resource::<R>();
        let value = self
            .resources
            .entry(component_id)
            .or_insert_with(|| ResourceRequest { mutable });
        value.mutable |= mutable;
    }

    /// Returns requested component id and their mutability
    pub fn requested_components(&self) -> &HashMap<ComponentId, ComponentRequests> {
        &self.components
    }

    /// Returns component_ids for requested resources and their mutability
    pub fn requested_resources(&self) -> &HashMap<ComponentId, ResourceRequest> {
        &self.resources
    }
}

pub struct ComponentRequests {
    /// Need mutable access for this component
    pub mutable: bool,
}

pub struct ResourceRequest {
    /// Need mutable access for this resource
    pub mutable: bool,
}
