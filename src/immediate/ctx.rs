use bevy_ecs::{
    entity::Entity,
    hierarchy::ChildOf,
    system::{Commands, Res},
};

use crate::{
    CapQueryParam, Imm, ImmCap, ImmId, ImmQueryInternal,
    capabilities::CapResourcesParam,
    immediate::{
        Current, ImmMarker, entity_mapping::ImmediateModeEntityMapping,
        upkeep::ImmediateModeStateResource,
    },
};

/// Immediate mode ctx
#[derive(bevy_ecs::system::SystemParam)]
pub struct ImmCtx<'w, 's, Cap: ImmCap> {
    /// Access data from entities for components that were requested in extensions
    pub entities: CapQueryParam<'w, 's, Cap>,

    /// Access requested resources
    pub resources: CapResourcesParam<'w, 's, Cap>,

    /// World commands
    pub commands: Commands<'w, 's>,

    pub(super) state: Res<'w, ImmediateModeStateResource<Cap>>,
    pub(super) mapping: Res<'w, ImmediateModeEntityMapping<Cap>>,
    pub(super) entity_query: ImmQueryInternal<'w, 's, Cap, ImmEntityQuery<Cap>>,
}

impl<'w, 's, CM> ImmCtx<'w, 's, CM>
where
    CM: ImmCap,
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

    /// Initialize entity hierarchy managed by immediate mode starting from given entity
    pub fn build_immediate_from<T: std::hash::Hash>(
        self,
        root_id: T,
        entity: Entity,
    ) -> Imm<'w, 's, CM> {
        Imm {
            ctx: self,
            current: Current {
                id: ImmId::new(root_id),
                entity: Some(entity),
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
