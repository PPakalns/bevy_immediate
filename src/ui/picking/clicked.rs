use bevy_ecs::{
    entity::Entity,
    observer::Trigger,
    system::{Commands, ResMut},
    world::OnAdd,
};
use bevy_picking::events::{Click, Pointer};
use bevy_platform::collections::HashSet;

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
        cap_req.request_resource::<TrackClickedEntitiesResource>(false);
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

        let mut query = self.ctx_mut().query.get_query::<Option<&TrackClicked>>();

        if !query.query().contains(entity) {
            // Auto insert track clicked capability
            self.entity_commands().insert_if_new(TrackClicked);
            return false;
        }

        self.ctx_mut()
            .resources
            .with_resource::<TrackClickedEntitiesResource, _>(|res| res.clicked.contains(&entity))
    }
}

////////////////////////////////////////////////////////////////////////////////

/// Add click tracking related logic
pub struct TrackClickedPlugin;

impl bevy_app::Plugin for TrackClickedPlugin {
    fn build(&self, app: &mut bevy_app::App) {
        app.insert_resource(TrackClickedEntitiesResource::default());
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
#[component(storage = "SparseSet")]
pub struct TrackClicked;

fn on_click(trigger: Trigger<Pointer<Click>>, mut resource: ResMut<TrackClickedEntitiesResource>) {
    let entity = trigger.target();
    resource.clicked.insert(entity);
}

#[derive(bevy_ecs::resource::Resource, Default)]
struct TrackClickedEntitiesResource {
    pub clicked: HashSet<Entity>,
}

fn reset_clicked_tracking(mut res: ResMut<TrackClickedEntitiesResource>) {
    res.clicked.clear();
}
