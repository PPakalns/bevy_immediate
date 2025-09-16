use bevy_ecs::{
    hierarchy::ChildOf,
    system::{Commands, Query, Res},
};

#[cfg(feature = "picking")]
use crate::ui::picking;
use crate::{
    Imm, ImmId, ImmediateModeTrackerComponent,
    immediate::{Current, ImmediateModeStateResource, entity_mapping::ImmediateModeEntityMapping},
};

/// Immediate mode ctx
#[derive(bevy_ecs::system::SystemParam)]
pub struct ImmCtx<'w, 's> {
    pub(super) query: Query<'w, 's, EntityQuery>,

    #[cfg(feature = "picking")]
    pub(super) track_clicked_query: Query<'w, 's, &'static picking::TrackClicked>,

    pub(super) state: Res<'w, ImmediateModeStateResource>,
    pub(super) mapping: Res<'w, ImmediateModeEntityMapping>,
    pub(super) commands: Commands<'w, 's>,
}

impl<'w, 's> ImmCtx<'w, 's> {
    /// Initialize entity hierarchy managed by immediate mode
    pub fn init<T: std::hash::Hash>(self, root_id: T) -> Imm<'w, 's> {
        Imm {
            ctx: self,
            current: Current {
                id: ImmId::new(root_id),
                entity: None,
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
pub(super) struct EntityQuery {
    pub(super) tracker: &'static mut ImmediateModeTrackerComponent,
    pub(super) child_of: Option<&'static ChildOf>,
}
