use std::marker::PhantomData;

use ahash::HashMap;
use bevy_ecs::{
    entity::Entity,
    observer::Trigger,
    system::{Query, ResMut},
    world::{OnAdd, OnRemove},
};

use crate::{ImmId, immediate::ImmMarker};

#[derive(bevy_ecs::resource::Resource)]
pub struct ImmediateModeEntityMapping<Cap: Send + Sync + 'static> {
    pub(super) id_to_entity: HashMap<ImmId, Entity>,
    _ph: PhantomData<Cap>,
}

impl<Cap: Send + Sync + 'static> Default for ImmediateModeEntityMapping<Cap> {
    fn default() -> Self {
        Self {
            id_to_entity: Default::default(),
            _ph: Default::default(),
        }
    }
}

pub fn init<Cap: Send + Sync + 'static>(app: &mut bevy_app::App) {
    app.insert_resource(ImmediateModeEntityMapping::<Cap>::default());
    app.add_observer(on_sui_marker_added::<Cap>)
        .add_observer(on_sui_marker_removed::<Cap>);
}

fn on_sui_marker_added<Cap: Send + Sync + 'static>(
    trigger: Trigger<OnAdd, ImmMarker<Cap>>,
    marker: Query<&ImmMarker<Cap>>,
    mut mapping: ResMut<ImmediateModeEntityMapping<Cap>>,
) {
    let entity = trigger.target();
    if let Ok(marker) = marker.get(entity) {
        let old = mapping.id_to_entity.insert(marker.id, entity);
        if let Some(old) = old {
            log::warn!(
                "Immediate mode entity id collision for entities {} and {}",
                entity,
                old
            );
        }
    }
}

fn on_sui_marker_removed<Cap: Send + Sync + 'static>(
    trigger: Trigger<OnRemove, ImmMarker<Cap>>,
    marker: Query<&ImmMarker<Cap>>,
    mut mapping: ResMut<ImmediateModeEntityMapping<Cap>>,
) {
    let entity = trigger.target();
    if let Ok(marker) = marker.get(entity) {
        mapping.id_to_entity.remove(&marker.id);
    }
}
