use std::marker::PhantomData;

use ahash::HashMap;
use bevy_ecs::{
    entity::Entity,
    observer::Trigger,
    system::{Query, ResMut},
    world::{OnAdd, OnRemove},
};

use crate::{ImmId, immediate::ImmTrackerComponent};

#[derive(bevy_ecs::resource::Resource)]
pub struct ImmediateModeEntityMapping<Marker: Send + Sync + 'static> {
    pub(super) id_to_entity: HashMap<ImmId, Entity>,
    _ph: PhantomData<Marker>,
}

impl<Marker: Send + Sync + 'static> Default for ImmediateModeEntityMapping<Marker> {
    fn default() -> Self {
        Self {
            id_to_entity: Default::default(),
            _ph: Default::default(),
        }
    }
}

pub fn init<Marker: Send + Sync + 'static>(app: &mut bevy_app::App) {
    app.insert_resource(ImmediateModeEntityMapping::<Marker>::default());
    app.add_observer(on_sui_marker_added::<Marker>)
        .add_observer(on_sui_marker_removed::<Marker>);
}

fn on_sui_marker_added<Marker: Send + Sync + 'static>(
    trigger: Trigger<OnAdd, ImmTrackerComponent<Marker>>,
    marker: Query<&ImmTrackerComponent<Marker>>,
    mut mapping: ResMut<ImmediateModeEntityMapping<Marker>>,
) {
    let entity = trigger.target();
    if let Ok(marker) = marker.get(entity) {
        mapping.id_to_entity.insert(marker.id, entity);
    }
}

fn on_sui_marker_removed<Marker: Send + Sync + 'static>(
    trigger: Trigger<OnRemove, ImmTrackerComponent<Marker>>,
    marker: Query<&ImmTrackerComponent<Marker>>,
    mut mapping: ResMut<ImmediateModeEntityMapping<Marker>>,
) {
    let entity = trigger.target();
    if let Ok(marker) = marker.get(entity) {
        mapping.id_to_entity.remove(&marker.id);
    }
}
