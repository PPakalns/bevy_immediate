use type_map::TypeMap;

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
