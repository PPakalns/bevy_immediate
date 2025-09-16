use bevy_ecs::schedule::IntoScheduleConfigs;

use crate::{ImmCap, ImmImplCap, ImmediateSystemSet, impl_capabilities};

#[cfg(feature = "picking")]
pub struct ImmCapUiTest2;

#[cfg(feature = "picking")]
impl_capabilities!(ImmCapUiTest2, (ImmCapUi));

////////////////////////////////////////////////////////////////////////////////

/// Defines capability that contains all Ui capabilities from this crate
pub struct ImmCapUi;

#[cfg(feature = "picking")]
impl_capabilities!(ImmCapUi, (ImmCapUiBase, picking::ImmCapUiClicked));

#[cfg(not(feature = "picking"))]
impl_capabilities!(ImmUiCap, (ImmUiBaseCap));

////////////////////////////////////////////////////////////////////////////////

/// Base capability for UI that sets up correct order of immediate system execution
pub struct ImmCapUiBase;

impl ImmCap for ImmCapUiBase {
    fn build<Cap: ImmCap>(
        app: &mut bevy_app::App,
        cap_req: &mut crate::ImmCapabilitiesRequests<Cap>,
    ) {
        app.configure_sets(
            bevy_app::PostUpdate,
            ImmediateSystemSet::<Cap>::default().before(bevy_ui::UiSystem::Prepare),
        );

        let _ = cap_req;
    }
}

/// Contains API extensions for ergonomic API that use [`bevy_picking`]
#[cfg(feature = "picking")]
pub mod picking;
