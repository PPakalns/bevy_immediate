use std::marker::PhantomData;

use bevy_ecs::{
    component::{Component, Components, Mutable},
    query::{With, Without},
    resource::{IsResource, Resource, ResourceEntities},
    system::Query,
    world::{FilteredEntityMut, Mut, Ref, error::ResourceFetchError},
};

use crate::{CapSet, ImmMarker};

/// You can retrieve components that were requested by capabilities. See [`FilteredEntityMut`]
///
/// [`SystemParam`] for immediate mode capability requests [`CapSet`]
#[derive(bevy_derive::Deref, bevy_derive::DerefMut)]
pub struct ImmCapQueryParam<'w, 's, Cap: CapSet> {
    /// Query for accessing entities being built by immediate mode system
    ///
    /// This query makes available requested components registered by capabilities
    ///
    /// Filtered with [`Without<IsResource>`] so it does not conflict with
    /// [`ImmCapResourcesParam`].
    ///
    /// In case of collision. Use [`super::ImmQuery`] or
    /// [`bevy_ecs::prelude::Without<ImmMarker<()>>`] (replace () with your used `Cap``)
    #[deref]
    pub query: Query<
        'w,
        's,
        FilteredEntityMut<'static, 'static>,
        (With<ImmMarker<Cap>>, Without<IsResource>),
    >,
}

/// You can retrieve resources that were registered by capability via a
/// [`FilteredEntityMut`] query over [`IsResource`] entities.
///
/// [`SystemParam`] for immediate mode capability requests [`CapSet`]
pub struct ImmCapResourcesParam<'w, 's, Cap: CapSet> {
    /// Query over resource entities with access declared by capabilities
    pub query: Query<'w, 's, FilteredEntityMut<'static, 'static>, With<IsResource>>,
    resource_entities: &'w ResourceEntities,
    components: &'w Components,
    _ph: PhantomData<Cap>,
}

impl<'w, 's, Cap: CapSet> ImmCapResourcesParam<'w, 's, Cap> {
    /// Gets a reference to the resource of the given type if it was requested and exists.
    pub fn get<R: Resource>(&self) -> Result<Ref<'_, R>, ResourceFetchError> {
        let component_id = self
            .components
            .valid_component_id::<R>()
            .ok_or(ResourceFetchError::NotRegistered)?;
        let entity = self
            .resource_entities
            .get(component_id)
            .ok_or(ResourceFetchError::DoesNotExist(component_id))?;
        let entity_ref = self
            .query
            .get(entity)
            .map_err(|_| ResourceFetchError::NoResourceAccess(component_id))?;
        entity_ref
            .get_ref::<R>()
            .ok_or(ResourceFetchError::NoResourceAccess(component_id))
    }

    /// Gets a mutable reference to the resource of the given type if it was requested and exists.
    pub fn get_mut<R: Resource + Component<Mutability = Mutable>>(
        &mut self,
    ) -> Result<Mut<'_, R>, ResourceFetchError> {
        let component_id = self
            .components
            .valid_component_id::<R>()
            .ok_or(ResourceFetchError::NotRegistered)?;
        let entity = self
            .resource_entities
            .get(component_id)
            .ok_or(ResourceFetchError::DoesNotExist(component_id))?;
        let entity_ref = self
            .query
            .get_mut(entity)
            .map_err(|_| ResourceFetchError::NoResourceAccess(component_id))?;
        entity_ref
            .into_mut::<R>()
            .ok_or(ResourceFetchError::NoResourceAccess(component_id))
    }
}

mod system_param_impl;
