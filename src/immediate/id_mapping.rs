use ahash::HashMap;
use bevy_ecs::{
    entity::Entity,
    observer::Trigger,
    system::{Query, ResMut},
    world::{OnAdd, OnRemove},
};

use crate::{SuiId, SuiMarker};

#[derive(bevy_ecs::resource::Resource, Default)]
pub struct SimpleUiIdMapping {
    pub(super) id_to_entity: HashMap<SuiId, Entity>,
}

pub fn init(app: &mut bevy_app::App) {
    app.insert_resource(SimpleUiIdMapping::default());
    app.add_observer(on_sui_marker_added)
        .add_observer(on_sui_marker_removed);
}

fn on_sui_marker_added(
    trigger: Trigger<OnAdd, SuiMarker>,
    marker: Query<&SuiMarker>,
    mut mapping: ResMut<SimpleUiIdMapping>,
) {
    let entity = trigger.target();
    if let Ok(marker) = marker.get(entity) {
        mapping.id_to_entity.insert(marker.id, entity);
    }
}

fn on_sui_marker_removed(
    trigger: Trigger<OnRemove, SuiMarker>,
    marker: Query<&SuiMarker>,
    mut mapping: ResMut<SimpleUiIdMapping>,
) {
    let entity = trigger.target();
    if let Ok(marker) = marker.get(entity) {
        mapping.id_to_entity.remove(&marker.id);
    }
}
