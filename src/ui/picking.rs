use bevy_ecs::{
    entity::Entity,
    observer::Trigger,
    system::{Query, ResMut},
};
use bevy_picking::events::{Click, Pointer};

use crate::{ImmCap, ImmCapabilitiesRequests, ImmEntity, ImmImplCap};

/// Immediate mode capability for `.clicked()`
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct ImmCapUiClicked;

impl ImmCap for ImmCapUiClicked {
    fn build<CM: ImmCap>(app: &mut bevy_app::App, cap_req: &mut ImmCapabilitiesRequests<CM>) {
        if !app.is_plugin_added::<TrackClickedPlugin>() {
            app.add_plugins(TrackClickedPlugin);
        }

        cap_req.request_optional_component::<TrackClicked>(app.world_mut(), false);
    }
}

/// Implements support for `.clicked()`
pub trait ImmUiClicked {
    /// Entity clicked during last frame
    fn clicked(&mut self) -> bool;
}

impl<Cap: ImmCap> ImmUiClicked for ImmEntity<'_, '_, '_, Cap>
where
    Cap: ImmImplCap<ImmCapUiClicked>,
{
    fn clicked(&mut self) -> bool {
        let entity = self.entity();

        let mut query = self.ctx().params.get_query::<Option<&TrackClicked>>();

        match query.query().get(entity) {
            Ok(Some(entity)) => entity.clicked,
            Ok(None) | Err(_) => {
                self.entity_commands()
                    .insert(TrackClicked::default())
                    .observe(on_click);
                false
            }
        }
    }
}

////////////////////////////////////////////////////////////////////////////////

/// Add click tracking related logic
pub struct TrackClickedPlugin;

impl bevy_app::Plugin for TrackClickedPlugin {
    fn build(&self, app: &mut bevy_app::App) {
        println!("Initializing track clicked plugin!");
        app.insert_resource(TrackClickedResetResource::default());
        app.add_systems(bevy_app::First, reset_clicked_tracking);
    }
}

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
        println!("Clicked");
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
            println!("Cleaned");
            comp.clicked = false;
        }
    }
}
