use bevy_ecs::schedule::IntoScheduleConfigs;

use crate::{ImmCap, ImmediateSystemSet, ImplCap};

/// Base capability for UI that sets up correct order of immediate system execution
pub struct CapUiBase;

impl ImplCap<CapUiBase> for CapUiBase {}
impl ImmCap for CapUiBase {
    fn build<Cap: ImmCap>(app: &mut bevy_app::App, cap_req: &mut crate::ImmCapAccessRequests<Cap>) {
        app.configure_sets(
            bevy_app::PostUpdate,
            ImmediateSystemSet::<Cap>::default().before(bevy_ui::UiSystem::Prepare),
        );

        let _ = cap_req;
    }
}
