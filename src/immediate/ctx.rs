use bevy_ecs::{
    entity::Entity,
    hierarchy::ChildOf,
    system::{Commands, Res, SystemChangeTick},
};

use crate::{
    CapSet, Imm, ImmCapQueryParam, ImmId, ImmQueryInternal,
    capabilities::ImmCapResourcesParam,
    immediate::{
        Current, CurrentEntity, ImmMarker, entity_mapping::ImmediateModeEntityMapping,
        upkeep::ImmediateModeStateResource,
    },
};

/// Immediate mode ctx
#[derive(bevy_ecs::system::SystemParam)]
pub struct ImmCtx<'w, 's, Cap: CapSet> {
    /// Access data from entities for components that were requested by capabilities
    ///
    /// In case of collision. Use [`super::ImmQuery`] or
    /// [`bevy_ecs::prelude::Without<ImmMarker<()>>`] (replace () with your used `Cap``)
    pub cap_entities: ImmCapQueryParam<'w, 's, Cap>,

    /// Access requested resources that were requested by capabilities
    pub cap_resources: ImmCapResourcesParam<'w, 's, Cap>,

    /// World commands
    pub commands: Commands<'w, 's>,

    /// System execution ticks
    ///
    /// Useful for state change detection
    pub system_change_tick: SystemChangeTick,

    pub(super) state: Res<'w, ImmediateModeStateResource<Cap>>,
    pub(super) mapping: Res<'w, ImmediateModeEntityMapping<Cap>>,
    pub(super) entity_query: ImmQueryInternal<'w, 's, Cap, ImmEntityQuery<Cap>>,
}

impl<'w, 's, CM> ImmCtx<'w, 's, CM>
where
    CM: CapSet,
{
    /// Initialize entity hierarchy managed by immediate mode
    pub fn build_immediate_root<T: std::hash::Hash>(self, root_id: T) -> Imm<'w, 's, CM> {
        Imm {
            ctx: self,
            current: Current {
                id: ImmId::new(root_id),
                entity: None,
                idx: 0,
            },
        }
    }

    /// Initialize entity hierarchy managed by immediate mode starting from given **existing** entity
    pub fn build_immediate_from<T: std::hash::Hash>(
        self,
        root_id: T,
        entity: Entity,
    ) -> Imm<'w, 's, CM> {
        Imm {
            ctx: self,
            current: Current {
                id: ImmId::new(root_id),
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
