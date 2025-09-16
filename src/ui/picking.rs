use bevy_ecs::{
    entity::Entity,
    observer::Trigger,
    system::{Query, ResMut},
};
use bevy_picking::events::{Click, Pointer};

/// Add UI related functionality to immediate mode API
pub struct BevyImmediateUiPickingExtensionPlugin;

impl bevy_app::Plugin for BevyImmediateUiPickingExtensionPlugin {
    fn build(&self, app: &mut bevy_app::App) {
        app.insert_resource(TrackClickedResetResource::default());
        app.add_systems(bevy_app::First, reset_clicked_tracking);
    }
}

// pub trait ImmediateClicked {
//     pub fn clicked(&mut self) -> bool {}
// }

/// Tracks if entity has been clicked in this frame.
#[derive(bevy_ecs::component::Component, Default)]
pub struct TrackClicked {
    clicked: bool,
}
impl TrackClicked {
    /// Retrieve whether entity has been clicked in this frame
    pub fn get(&self) -> bool {
        self.clicked
    }
}

// TODO: Remove pub(crate)
pub(crate) fn on_click(
    trigger: Trigger<Pointer<Click>>,
    mut query: Query<&mut TrackClicked>,
    mut resource: ResMut<TrackClickedResetResource>,
) {
    let entity = trigger.target();
    if let Ok(mut comp) = query.get_mut(entity) {
        comp.clicked = true;
        resource.clicked.push(entity);
    }
}

// TODO: Remove pub(crate)
#[derive(bevy_ecs::resource::Resource, Default)]
pub(crate) struct TrackClickedResetResource {
    clicked: Vec<Entity>,
}

fn reset_clicked_tracking(
    mut query: Query<&mut TrackClicked>,
    mut res: ResMut<TrackClickedResetResource>,
) {
    for entity in res.clicked.drain(..) {
        if let Ok(mut comp) = query.get_mut(entity) {
            comp.clicked = false;
        }
    }
}
