use std::marker::PhantomData;

use bevy_ecs::{
    hierarchy::ChildOf,
    system::{Commands, Query, Res},
};

use crate::{
    Imm, ImmCM, ImmId,
    immediate::{
        CMMarker, Current, ImmMarker, ImmTrackerComponent, ImmediateModeStateResource,
        entity_mapping::ImmediateModeEntityMapping,
    },
};
#[cfg(feature = "picking")]
use crate::{immediate::WithImmMarker, ui::picking};

/// Immediate mode ctx
#[derive(bevy_ecs::system::SystemParam)]
pub struct ImmCtx<'w, 's, CM: ImmCM> {
    pub(super) query: Query<'w, 's, ImmEntityQuery<CMMarker<CM>>, WithImmMarker<CM>>,

    #[cfg(feature = "picking")]
    pub(super) track_clicked_query:
        Query<'w, 's, &'static picking::TrackClicked, WithImmMarker<CM>>,

    pub(super) state: Res<'w, ImmediateModeStateResource<CMMarker<CM>>>,
    pub(super) mapping: Res<'w, ImmediateModeEntityMapping<CMMarker<CM>>>,
    pub(super) commands: Commands<'w, 's>,

    ph: PhantomData<CM>,
}

impl<'w, 's, CM> ImmCtx<'w, 's, CM>
where
    CM: ImmCM,
{
    /// Initialize entity hierarchy managed by immediate mode
    pub fn init<T: std::hash::Hash>(self, root_id: T) -> Imm<'w, 's, CM> {
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
pub(super) struct ImmEntityQuery<Marker: Send + Sync + 'static> {
    pub(super) tracker: &'static mut ImmTrackerComponent<Marker>,
    pub(super) child_of: Option<&'static ChildOf>,
}
