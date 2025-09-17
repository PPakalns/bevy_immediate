use std::marker::PhantomData;

use bevy_ecs::{
    entity::Entity,
    query::With,
    schedule::IntoScheduleConfigs,
    system::{Commands, Query, ResMut},
};

use crate::{ImmMarker, ImmediateSystemSet};

pub fn init<Cap: Send + Sync + 'static>(app: &mut bevy_app::App) {
    app.add_systems(
        bevy_app::PostUpdate,
        immediate_mode_tracked_entity_upkeep_system::<Cap>
            .in_set(ImmediateSystemSet::<()>::default()),
    );
    app.insert_resource(ImmediateModeStateResource::<Cap>::default());
}

#[derive(bevy_ecs::resource::Resource)]
pub(super) struct ImmediateModeStateResource<Cap: Send + Sync + 'static> {
    // Current iteration for unused entity removal
    pub(super) iteration: u32,

    _ph: PhantomData<Cap>,
}

impl<Cap: Send + Sync + 'static> Default for ImmediateModeStateResource<Cap> {
    fn default() -> Self {
        Self {
            iteration: Default::default(),
            _ph: Default::default(),
        }
    }
}

fn immediate_mode_tracked_entity_upkeep_system<Cap: Send + Sync + 'static>(
    query: Query<(Entity, &ImmMarker<Cap>), With<ImmMarker<Cap>>>,
    mut state: ResMut<ImmediateModeStateResource<Cap>>,
    mut commands: Commands,
) {
    for (entity, marker) in query {
        if marker.iteration == state.iteration {
            continue;
        }
        // Try is used because
        // it is expected that ancestor may have already removed this entity
        commands.entity(entity).try_despawn();
    }

    state.iteration = state.iteration.wrapping_add(1);
}
