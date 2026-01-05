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

    /// Retrieve raw inner id
    pub fn raw(&self) -> u64 {
        self.id
    }
}

impl<A> FromIterator<A> for ImmId
where
    A: std::hash::Hash,
{
    fn from_iter<T: IntoIterator<Item = A>>(iter: T) -> Self {
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
                const AUTO: u32 = 295847291;
                let id = sui
                    .current
                    .id
                    .with((AUTO, sui.current.id_pref, sui.current.auto_id_idx));
                sui.current.auto_id_idx += 1;
                id
            }
            ImmIdBuilder::Hierarchy(sui_id) => {
                const HIERARCHY: u32 = 958472831;
                sui.current
                    .id
                    .with((HIERARCHY, sui.current.id_pref, sui_id))
            }
            ImmIdBuilder::Unique(sui_id) => sui_id,
        }
    }
}
