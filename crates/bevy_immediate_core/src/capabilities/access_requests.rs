use std::sync::Arc;

use bevy_ecs::{
    component::{Component, ComponentId, Mutable},
    resource::Resource,
    world::World,
};
use bevy_platform::collections::HashMap;

use crate::{CapSet, ImmEntity};

/// Stores requested capabilities for given immediate mode request
#[derive(bevy_ecs::resource::Resource)]
pub struct ImmCapAccessRequestsResource<Caps: CapSet> {
    /// Information about access requests issued by provided `Cap`
    pub capabilities: Arc<ImmCapAccessRequests<Caps>>,
}

impl<Cap: CapSet> ImmCapAccessRequestsResource<Cap> {
    pub(crate) fn new(capabilities: ImmCapAccessRequests<Cap>) -> Self {
        Self {
            capabilities: Arc::new(capabilities),
        }
    }
}

/// Tracks what kind of query accesses capability has requested
pub struct ImmCapAccessRequests<Caps: CapSet> {
    // type_id_map: HashMap<TypeId, ComponentId>,
    components: HashMap<ComponentId, ComponentRequests>,
    resources: HashMap<ComponentId, ResourceRequest>,

    #[allow(clippy::type_complexity)]
    pub(crate) on_children: Vec<Box<dyn Fn(&mut ImmEntity<'_, '_, '_, Caps>) + Send + Sync>>,
}

impl<Caps: CapSet> Default for ImmCapAccessRequests<Caps> {
    fn default() -> Self {
        Self {
            components: Default::default(),
            resources: Default::default(),
            on_children: Default::default(),
        }
    }
}

impl<Caps: CapSet> ImmCapAccessRequests<Caps> {
    /// Mark that component will be immutably accessed
    pub fn request_component_read<C: Component>(&mut self, world: &mut World) {
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

    /// Add listener that will be called after child entity is created in UI
    #[allow(clippy::type_complexity)]
    pub fn add_on_children_event_listener(
        &mut self,
        listener: Box<dyn Fn(&mut ImmEntity<Caps>) + Send + Sync>,
    ) {
        self.on_children.push(listener);
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
