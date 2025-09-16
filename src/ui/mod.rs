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
        #[cfg(feature = "picking")]
        app.add_plugins(picking::BevyImmediateUiPickingExtensionPlugin);

        let _ = app;
    }
}

#[cfg(feature = "picking")]
pub mod picking;
