use std::{any::TypeId, marker::PhantomData};

use bevy_ecs::{
    entity::Entity,
    resource::Resource,
    system::{Query, ResMut},
};
use bevy_platform::collections::{HashMap, hash_map::Entry};

use crate::ImmId;

pub fn init<Caps: Send + Sync + 'static>(app: &mut bevy_app::App) {
    app.add_systems(bevy_app::PostUpdate, clean_cached::<Caps>);
    app.insert_resource(CachedHash::<Caps>::default());
}

#[derive(Resource)]
pub(super) struct CachedHash<Caps> {
    values: HashMap<Key, ImmId>,
    _ph: PhantomData<Caps>,
}

fn inner_cache(this: &mut HashMap<Key, ImmId>, key: Key, value: ImmId) -> bool {
    match this.entry(key) {
        Entry::Vacant(entry) => {
            entry.insert(value);
            true
        }
        Entry::Occupied(mut entry) => {
            if *entry.get() == value {
                false
            } else {
                entry.insert(value);
                true
            }
        }
    }
}

fn inner_is_set(this: &HashMap<Key, ImmId>, key: Key) -> Option<ImmId> {
    this.get(&key).copied()
}

fn inner_remove(this: &mut HashMap<Key, ImmId>, key: Key) -> Option<ImmId> {
    this.remove(&key)
}

impl<Caps> CachedHash<Caps> {
    pub fn cache(&mut self, entity: Entity, key: ImmId, value: ImmId) -> bool {
        inner_cache(
            &mut self.values,
            Key {
                entity,
                cache_key: CacheKey::Id(key),
            },
            value,
        )
    }

    pub fn cache_typ<Marker: 'static>(&mut self, entity: Entity, value: ImmId) -> bool {
        let typeid = TypeId::of::<Marker>();
        inner_cache(
            &mut self.values,
            Key {
                entity,
                cache_key: CacheKey::TypeId(typeid),
            },
            value,
        )
    }

    pub fn get(&self, entity: Entity, key: ImmId) -> Option<ImmId> {
        inner_is_set(
            &self.values,
            Key {
                entity,
                cache_key: CacheKey::Id(key),
            },
        )
    }

    pub fn get_typ<Marker: 'static>(&self, entity: Entity) -> Option<ImmId> {
        let typeid = TypeId::of::<Marker>();
        inner_is_set(
            &self.values,
            Key {
                entity,
                cache_key: CacheKey::TypeId(typeid),
            },
        )
    }

    pub fn remove(&mut self, entity: Entity, key: ImmId) -> Option<ImmId> {
        inner_remove(
            &mut self.values,
            Key {
                entity,
                cache_key: CacheKey::Id(key),
            },
        )
    }

    pub fn remove_typ<Marker: 'static>(&mut self, entity: Entity) -> Option<ImmId> {
        let typeid = TypeId::of::<Marker>();
        inner_remove(
            &mut self.values,
            Key {
                entity,
                cache_key: CacheKey::TypeId(typeid),
            },
        )
    }
}

impl<Caps> Default for CachedHash<Caps> {
    fn default() -> Self {
        Self {
            values: Default::default(),
            _ph: Default::default(),
        }
    }
}

#[derive(Hash, PartialEq, Eq)]
struct Key {
    entity: Entity,
    cache_key: CacheKey,
}

#[derive(Hash, PartialEq, Eq)]
pub enum CacheKey {
    TypeId(TypeId),
    Id(ImmId),
}

fn clean_cached<Caps: Send + Sync + 'static>(
    mut cached: ResMut<CachedHash<Caps>>,
    query: Query<()>,
) {
    cached
        .values
        .retain(|key, _value| query.contains(key.entity));
}
