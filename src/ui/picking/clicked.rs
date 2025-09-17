use bevy_ecs::{
    entity::Entity,
    observer::Trigger,
    system::{Commands, Query, ResMut},
    world::OnAdd,
};
use bevy_picking::events::{Click, Pointer};

use crate::{CapAccessRequests, ImmCap, ImmEntity, ImmImplCap};

/// Immediate mode capability for `.clicked()`
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct ImmCapUiClicked;

impl ImmCap for ImmCapUiClicked {
    fn build<CM: ImmCap>(app: &mut bevy_app::App, cap_req: &mut CapAccessRequests<CM>) {
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

        let mut query = self.ctx_mut().params.get_query::<Option<&TrackClicked>>();

        match query.query().get(entity) {
            Ok(Some(entity)) => entity.clicked,
            Ok(None) | Err(_) => {
                self.entity_commands()
                    .insert_if_new(TrackClicked::default());
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
        app.insert_resource(TrackClickedResetResource::default());
        app.add_systems(bevy_app::First, reset_clicked_tracking);
        app.add_observer(track_clicked_insert);
    }
}

// Insert on_click picking observer only once
fn track_clicked_insert(trigger: Trigger<OnAdd, TrackClicked>, mut commands: Commands) {
    let entity = trigger.target();
    commands.entity(entity).observe(on_click);
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

fn on_click(
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

#[derive(bevy_ecs::resource::Resource, Default)]
struct TrackClickedResetResource {
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
