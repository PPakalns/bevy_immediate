use std::marker::PhantomData;

/// System set for systems that power `bevy_immediate` immediate mode functionality
#[derive(bevy_ecs::schedule::SystemSet)]
pub struct ImmediateSystemSet<CM>(PhantomData<CM>);

impl<CM> Default for ImmediateSystemSet<CM> {
    fn default() -> Self {
        Self(PhantomData)
    }
}

impl<CM> std::hash::Hash for ImmediateSystemSet<CM> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.0.hash(state);
    }
}

impl<CM> std::fmt::Debug for ImmediateSystemSet<CM> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("ImmediateSystemSet").field(&self.0).finish()
    }
}

impl<CM> Clone for ImmediateSystemSet<CM> {
    fn clone(&self) -> Self {
        Self(self.0)
    }
}

impl<CM> Eq for ImmediateSystemSet<CM> {}

impl<CM> PartialEq for ImmediateSystemSet<CM> {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}
