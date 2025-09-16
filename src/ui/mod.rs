use bevy_ecs::schedule::IntoScheduleConfigs;

use crate::ImmediateSystemSet;

/// Plugin that adds support for
///
/// ```no_run
///   resp.clicked();
///   resp.hovered();
///   resp.primary_clicked();
///   resp.secondary_clicked();
/// ````
pub struct BevyImmediateUiExtensionPlugin;

impl bevy_app::Plugin for BevyImmediateUiExtensionPlugin {
    fn build(&self, app: &mut bevy_app::App) {
        app.configure_sets(
            bevy_app::PostUpdate,
            ImmediateSystemSet.before(bevy_ui::UiSystem::Prepare),
        );

        #[cfg(feature = "picking")]
        app.add_plugins(picking::BevyImmediateUiPickingExtensionPlugin);
    }
}

/// Contains API extensions for ergonomic API that use [`bevy_picking`]
#[cfg(feature = "picking")]
pub mod picking;
