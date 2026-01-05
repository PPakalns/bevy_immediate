use bevy_ecs::{message::MessageReader, resource::Resource, system::ResMut};

pub(super) fn init(app: &mut bevy_app::App) {
    if !app.world_mut().contains_resource::<HotpatchingCounter>() {
        app.insert_resource(HotpatchingCounter { hotpatch: 0 });
        app.add_systems(bevy_app::PreUpdate, hotpatch_listener);
    }
}

fn hotpatch_listener(
    mut listener: MessageReader<bevy_ecs::HotPatched>,
    mut hotpatch: ResMut<HotpatchingCounter>,
) {
    for _ in listener.read() {
        hotpatch.hotpatch += 1;
    }
}

/// Keeps count of how many times hotpatching has
/// been applied
#[cfg(feature = "hotpatching")]
#[derive(Resource)]
pub struct HotpatchingCounter {
    hotpatch: u32,
}

#[cfg(feature = "hotpatching")]
impl HotpatchingCounter {
    /// Returns how many times hotpatching has been applied
    #[inline]
    pub fn hotpatch(&self) -> u32 {
        self.hotpatch
    }
}
