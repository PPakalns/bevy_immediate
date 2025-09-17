use bevy_ecs::{
    entity::Entity,
    observer::Trigger,
    system::{Commands, ResMut},
    world::OnAdd,
};
use bevy_picking::{
    events::{Click, Pointer},
    pointer::PointerButton,
};
use bevy_platform::collections::HashMap;

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
    /// Primary button clicked
    fn primary_clicked(&mut self) -> bool;
    /// Secondary button clicked
    fn secondary_clicked(&mut self) -> bool;
    /// Middle button clicked
    fn middle_clicked(&mut self) -> bool;
    /// Pointer button that was used to click this entity
    fn clicked_by(&mut self) -> Option<PointerButton>;
    /// Access reference to stored pointer click event
    fn with_pointer_event<R>(&mut self, f: impl FnOnce(Option<&Pointer<Click>>) -> R) -> R;
}

impl<Cap: ImmCap> ImmUiClicked for ImmEntity<'_, '_, '_, Cap>
where
    Cap: ImmImplCap<ImmCapUiClicked>,
{
    fn clicked(&mut self) -> bool {
        self.with_pointer_event(|event| event.is_some())
    }

    fn primary_clicked(&mut self) -> bool {
        self.clicked_by() == Some(PointerButton::Primary)
    }

    fn secondary_clicked(&mut self) -> bool {
        self.clicked_by() == Some(PointerButton::Secondary)
    }

    fn middle_clicked(&mut self) -> bool {
        self.clicked_by() == Some(PointerButton::Middle)
    }

    fn clicked_by(&mut self) -> Option<PointerButton> {
        self.with_pointer_event(|event| event.map(|event| event.button))
    }

    fn with_pointer_event<R>(&mut self, f: impl FnOnce(Option<&Pointer<Click>>) -> R) -> R {
        'correct: {
            let Ok(entity) = self.get_entity() else {
                break 'correct;
            };

            if !entity.contains::<TrackClicked>() {
                break 'correct;
            }

            let entity_id = self.entity();

            let resource = self
                .ctx()
                .resources
                .get::<TrackClickedEntitiesResource>()
                .expect("Capability available");

            return f(resource.clicked.get(&entity_id));
        }

        self.entity_commands().insert_if_new(TrackClicked);
        f(None)
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
    resource.clicked.insert(entity, trigger.event().clone());
}

#[derive(bevy_ecs::resource::Resource, Default)]
struct TrackClickedEntitiesResource {
    pub clicked: HashMap<Entity, Pointer<Click>>,
}

fn reset_clicked_tracking(mut res: ResMut<TrackClickedEntitiesResource>) {
    res.clicked.clear();
}
