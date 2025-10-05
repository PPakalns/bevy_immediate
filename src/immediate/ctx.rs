use bevy_ecs::{
    entity::Entity,
    hierarchy::ChildOf,
    query::With,
    system::{Commands, Query, Res, ResMut, SystemChangeTick},
};

use crate::{
    CapSet, Imm, ImmCapAccessRequestsResource, ImmCapQueryParam, ImmId,
    capabilities::ImmCapResourcesParam,
    immediate::{
        Current, CurrentEntity, ImmMarker, cached_hash::CachedHash,
        entity_mapping::ImmediateModeEntityMapping, upkeep::ImmediateModeStateResource,
    },
};

/// Immediate mode ctx
#[derive(bevy_ecs::system::SystemParam)]
pub struct ImmCtx<'w, 's, Caps: CapSet> {
    /// Access data from entities for components that were requested by capabilities
    ///
    /// In case of collision. Use [`super::ImmQuery`] or
    /// [`bevy_ecs::prelude::Without<ImmMarker<()>>`] (replace () with your used `Cap``)
    pub cap_entities: ImmCapQueryParam<'w, 's, Caps>,

    /// Access requested resources that were requested by capabilities
    pub cap_resources: ImmCapResourcesParam<'w, 's, Caps>,

    /// World commands
    pub commands: Commands<'w, 's>,

    /// System execution ticks
    ///
    /// Useful for state change detection
    pub system_change_tick: SystemChangeTick,

    pub(super) access_requests: Res<'w, ImmCapAccessRequestsResource<Caps>>,
    pub(super) state: Res<'w, ImmediateModeStateResource<Caps>>,
    pub(super) mapping: Res<'w, ImmediateModeEntityMapping<Caps>>,
    pub(super) entity_query: Query<'w, 's, ImmEntityQuery<Caps>, (With<ImmMarker<Caps>>, ())>,
    pub(super) cached_hash: ResMut<'w, CachedHash<Caps>>,

    #[cfg(feature = "hotpatching")]
    pub(super) hotpatching: Res<'w, super::hotpatching::HotpatchingCounter>,
}

impl<'w, 's, Caps> ImmCtx<'w, 's, Caps>
where
    Caps: CapSet,
{
    /// Initialize entity hierarchy managed by immediate mode
    ///
    /// When `hotpatching` feature is enabled. Will combine id with last time when hotpatching was triggered
    pub fn build_immediate_root<T: std::hash::Hash>(self, root_id: T) -> Imm<'w, 's, Caps> {
        let id = ImmId::new(root_id);

        #[cfg(feature = "hotpatching")]
        let id = ImmId::new((id, self.hotpatching.hotpatch()));

        Imm {
            ctx: self,
            current: Current {
                id,
                entity: None,
                idx: 0,
            },
        }
    }

    /// Initialize entity hierarchy managed by immediate mode starting from given **existing** entity
    ///
    /// When `hotpatching` feature is enabled. Will combine id with last time when hotpatching was triggered
    pub fn build_immediate_from<T: std::hash::Hash>(
        self,
        root_id: T,
        entity: Entity,
    ) -> Imm<'w, 's, Caps> {
        let id = ImmId::new(root_id);

        #[cfg(feature = "hotpatching")]
        let id = ImmId::new((id, self.hotpatching.hotpatch()));

        Imm {
            ctx: self,
            current: Current {
                id,
                entity: Some(CurrentEntity {
                    entity,
                    will_be_spawned: false,
                }),
                idx: 0,
            },
        }
    }

    /// Access commands
    pub fn commands_mut(&mut self) -> &mut Commands<'w, 's> {
        &mut self.commands
    }
}

#[derive(bevy_ecs::query::QueryData)]
#[query_data(mutable)]
pub(super) struct ImmEntityQuery<Marker: Send + Sync + 'static> {
    pub(super) tracker: &'static mut ImmMarker<Marker>,
    pub(super) child_of: Option<&'static ChildOf>,
}
