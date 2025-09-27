use bevy_ecs::schedule::IntoScheduleConfigs;

use crate::{CapSet, ImmCapability, ImmediateSystemSet};

/// Base capability for UI that sets up correct order of immediate system execution
pub struct CapabilityUiBase;

impl ImmCapability for CapabilityUiBase {
    fn build<Cap: CapSet>(app: &mut bevy_app::App, cap_req: &mut crate::ImmCapAccessRequests<Cap>) {
        app.configure_sets(
            bevy_app::PostUpdate,
            ImmediateSystemSet::<Cap>::default().before(bevy_ui::UiSystems::Prepare),
        );

        let _ = cap_req;
    }
}
