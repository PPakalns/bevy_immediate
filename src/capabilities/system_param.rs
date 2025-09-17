use std::marker::PhantomData;

use bevy_ecs::{
    query::With,
    system::Query,
    world::{FilteredEntityMut, FilteredResourcesMut},
};

use crate::{ImmCap, ImmMarker};

/// You can retrieve components that were requested by capabilities. See [`FilteredEntityMut`]
///
/// [`SystemParam`] for immediate mode capability requests [`ImmCap`]
#[derive(bevy_derive::Deref, bevy_derive::DerefMut)]
pub struct CapQueryParam<'w, 's, Cap: ImmCap> {
    /// Query for accessing entities being built by immediate mode system
    ///
    /// This query makes available requested components registered by capabilities
    ///
    /// In case of collision. Use [`super::ImmQuery`] or
    /// [`bevy_ecs::prelude::Without<ImmMarker<()>>`] (replace () with your used `Cap``)
    #[deref]
    pub query: Query<'w, 's, FilteredEntityMut<'static>, With<ImmMarker<Cap>>>,
}

/// You can retrieve resources that were registered by capability. See [`FilteredResourcesMut`]
///
/// [`SystemParam`] for immediate mode capability requests [`ImmCap`]
#[derive(bevy_derive::Deref, bevy_derive::DerefMut)]
pub struct CapResourcesParam<'w, 's, Cap: ImmCap> {
    /// Can be used to access resources requested by capabilities
    #[deref]
    pub resources: FilteredResourcesMut<'w, 's>,
    _ph: PhantomData<Cap>,
}

mod system_param_impl;
