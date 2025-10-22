use crate::{CapSet, Imm};

/// Unique id for immediate mode entities.
///
/// Ids are used to track entities managed by immediate mode logic.
#[derive(Hash, Clone, Copy, PartialEq, Eq)]
pub struct ImmId {
    id: u64,
}

/// Helper function to easily construct [`ImmId`]
///
/// Needed because [`ImmId`] can not implement [`From<std::hash::Hash>`]
pub fn imm_id<T: std::hash::Hash>(val: T) -> ImmId {
    ImmId::new(val)
}

impl ImmId {
    /// Construct new id
    pub fn new(source: impl std::hash::Hash) -> Self {
        Self {
            id: ahash::RandomState::with_seeds(1, 2, 3, 4).hash_one(source),
        }
    }

    /// Combine current unique id with provided value
    pub fn with(self, child: impl std::hash::Hash) -> Self {
        use std::hash::{BuildHasher as _, Hasher as _};
        let mut hasher = ahash::RandomState::with_seeds(1, 2, 3, 4).build_hasher();
        hasher.write_u64(self.id);
        child.hash(&mut hasher);
        Self {
            id: hasher.finish(),
        }
    }

    /// Initialize id value from iterator over hashable elements
    pub fn from_iter<T: std::hash::Hash>(iter: impl Iterator<Item = T>) -> Self {
        use std::hash::{BuildHasher as _, Hasher as _};
        let mut hasher = ahash::RandomState::with_seeds(1, 2, 3, 4).build_hasher();
        for item in iter {
            item.hash(&mut hasher);
        }
        Self {
            id: hasher.finish(),
        }
    }
}

/// Can be used to construct unique id for new entity
pub enum ImmIdBuilder {
    /// Use auto generated id ()
    Auto,
    /// Combine parent node id with provided id
    Hierarchy(ImmId),
    /// Provide truly unique id
    ///
    /// Use with caution. Use only when it is guruanteed
    /// that only one entity with this id will exist.
    Unique(ImmId),
}

impl ImmIdBuilder {
    pub(super) fn resolve<Caps: CapSet>(self, sui: &mut Imm<Caps>) -> ImmId {
        match self {
            ImmIdBuilder::Auto => {
                // Auto increment id only when adding entity with auto generated id
                // All entities that are not permanent children of parent should have
                // non auto id
                //
                const AUTO_UNIQUE_PREFIX: i32 = 813754928;
                let id = sui.current.id.with((AUTO_UNIQUE_PREFIX, sui.current.idx));
                sui.current.idx += 1;
                id
            }
            ImmIdBuilder::Hierarchy(sui_id) => sui.current.id.with(sui_id),
            ImmIdBuilder::Unique(sui_id) => sui_id,
        }
    }
}
