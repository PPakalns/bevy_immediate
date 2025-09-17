use std::{any::TypeId, sync::Arc};

use bevy_ecs::{
    component::{Component, ComponentId, Mutable},
    query::{QueryBuilder, With},
    resource::Resource,
    world::{FilteredEntityMut, FilteredResourcesMutBuilder, World},
};
use bevy_platform::collections::HashMap;

use crate::{ImmCap, ImmMarker};

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
    components: HashMap<ComponentId, ComponentRequests<Cap>>,
    resources: HashMap<TypeId, ResourceRequest>,
}

pub struct ResourceRequest {
    pub mutable: bool,
    #[allow(clippy::type_complexity)]
    pub builder: Box<dyn Fn(&mut FilteredResourcesMutBuilder, bool) + 'static + Send + Sync>,
}

impl<Cap: ImmCap> Default for CapAccessRequests<Cap> {
    fn default() -> Self {
        Self {
            // type_id_map: Default::default(),
            components: Default::default(),
            resources: Default::default(),
        }
    }
}

impl<Cap: ImmCap> CapAccessRequests<Cap> {
    /// Mark component for retrieval during immediate mode execution through context [`crate::ImmCtx`]
    /// method by querying `Option<&Component>` or `Option<&mut Component>` based on `mutable` argument value.
    pub fn request_optional_component<C: Component<Mutability = Mutable>>(
        &mut self,
        world: &mut World,
        mutable: bool,
    ) {
        let component_id = world.register_component::<C>();
        // self.type_id_map.insert(TypeId::of::<C>(), component_id);
        let value = self
            .components
            .entry(component_id)
            .or_insert_with(|| ComponentRequests {
                mutable,
                builder: Box::new(|builder, mutable| match mutable {
                    true => {
                        builder.data::<Option<&mut C>>();
                    }
                    false => {
                        builder.data::<Option<&C>>();
                    }
                }),
            });
        value.mutable |= mutable;
    }

    /// Mark resource for retrieval during immediate mode execution through context [`crate::ImmCtx`].
    pub fn request_resource<R: Resource>(&mut self, mutable: bool) {
        let value = self
            .resources
            .entry(TypeId::of::<R>())
            .or_insert_with(|| ResourceRequest {
                mutable,
                builder: Box::new(|builder, mutable| {
                    if mutable {
                        builder.add_write::<R>();
                    } else {
                        builder.add_read::<R>();
                    }
                }),
            });
        value.mutable |= mutable;
    }

    /// Returns requested component id and their mutability
    pub fn requested_components(&self) -> &HashMap<ComponentId, ComponentRequests<Cap>> {
        &self.components
    }

    /// Returns component_ids for requested resources and their mutability
    pub fn requested_resources(&self) -> &HashMap<TypeId, ResourceRequest> {
        &self.resources
    }
}

pub type ComponentRequestBuilderFn<Cap> = dyn Fn(&mut QueryBuilder<FilteredEntityMut, With<ImmMarker<Cap>>>, bool)
    + 'static
    + Send
    + Sync;

pub struct ComponentRequests<Cap: ImmCap> {
    /// Need mutable access for this component
    pub mutable: bool,
    /// Logic to register this component to tracking
    pub builder: Box<ComponentRequestBuilderFn<Cap>>,
}
