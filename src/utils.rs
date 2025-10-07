use std::marker::PhantomData;

use type_map::TypeMap;

use crate::{CapSet, ImmEntity, ImmId, imm_id};

/// Wrapper structure around [`TypeMap`] for capabilities to store temporary data.
///
/// Wrapper type allows interaction only with data for which users can access Types.
/// Capabilities can not easily impact each other.
pub struct ImmTypeMap {
    type_map: TypeMap,
}

impl ImmTypeMap {
    #[inline]
    pub(crate) fn new() -> Self {
        Self {
            type_map: TypeMap::new(),
        }
    }

    // pub fn clear(&mut self) {
    //     self.type_map.clear()
    // }
    //
    /// See [TypeMap::contains]
    #[inline]
    pub fn contains<T: 'static>(&self) -> bool {
        self.type_map.contains::<T>()
    }

    /// See [TypeMap::entry]
    #[inline]
    pub fn entry<T: 'static>(&mut self) -> type_map::Entry<'_, T> {
        self.type_map.entry()
    }

    /// See [TypeMap::get]
    #[inline]
    pub fn get<T: 'static>(&self) -> Option<&T> {
        self.type_map.get()
    }

    /// See [TypeMap::get_mut]
    #[inline]
    pub fn get_mut<T: 'static>(&mut self) -> Option<&mut T> {
        self.type_map.get_mut()
    }

    /// See [TypeMap::insert]
    #[inline]
    pub fn insert<T: 'static>(&mut self, val: T) -> Option<T> {
        self.type_map.insert(val)
    }

    /// See [TypeMap::remove]
    #[inline]
    pub fn remove<T: 'static>(&mut self) -> Option<T> {
        self.type_map.remove()
    }
}

/// Helper tool to store simple hashed state in immediate mode entity.
///
/// Useful for implementing toggle buttons, menus, dropdowns where
/// simple local state is enough.
#[must_use = "You want to store processed value"]
pub struct ImmLocalHashMemoryHelper<T> {
    key: ImmId,
    state: ImmId,
    changed: bool,
    _ph: PhantomData<T>,
}

impl<T> ImmLocalHashMemoryHelper<T>
where
    T: std::hash::Hash,
{
    pub fn new<Caps>(
        store_on_entity: &mut ImmEntity<Caps>,
        memory_key: impl std::hash::Hash,
        default_value: &T,
    ) -> ImmLocalHashMemoryHelper<T>
    where
        Caps: CapSet,
    {
        let key = imm_id(memory_key);

        let mut changed = false;

        let current_key = store_on_entity.hash_get(key).unwrap_or_else(|| {
            changed = true;
            imm_id(default_value)
        });

        ImmLocalHashMemoryHelper {
            key,
            state: current_key,
            changed,
            _ph: PhantomData,
        }
    }

    pub fn is_stored(&mut self, value: &T) -> bool {
        self.state == imm_id(value)
    }

    pub fn store(&mut self, value: &T) {
        let value = imm_id(value);
        if self.state != value {
            self.state = value;
            self.changed = true;
        }
    }

    pub fn finalize<Caps: CapSet>(self, mut store_on_entity: &mut ImmEntity<Caps>) {
        if self.changed {
            store_on_entity.hash_set(self.key, self.state);
        }
    }
}
