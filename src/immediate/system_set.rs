use std::marker::PhantomData;

/// System set for systems that power `bevy_immediate` immediate mode functionality
#[derive(bevy_ecs::schedule::SystemSet)]
pub struct ImmediateSystemSet<Caps>(PhantomData<Caps>);

impl<Caps> Default for ImmediateSystemSet<Caps> {
    fn default() -> Self {
        Self(PhantomData)
    }
}

impl<Caps> std::hash::Hash for ImmediateSystemSet<Caps> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.0.hash(state);
    }
}

impl<Caps> std::fmt::Debug for ImmediateSystemSet<Caps> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("ImmediateSystemSet").field(&self.0).finish()
    }
}

impl<Caps> Clone for ImmediateSystemSet<Caps> {
    fn clone(&self) -> Self {
        Self(self.0)
    }
}

impl<Caps> Eq for ImmediateSystemSet<Caps> {}

impl<Caps> PartialEq for ImmediateSystemSet<Caps> {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}
