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

use crate::{CapSet, ImmCapAccessRequests, ImmCapability, ImmEntity, ImplCap};

/// Immediate mode capability for `.clicked()`
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct CapabilityUiClicked;

impl ImmCapability for CapabilityUiClicked {
    fn build<CM: CapSet>(app: &mut bevy_app::App, cap_req: &mut ImmCapAccessRequests<CM>) {
        if !app.is_plugin_added::<TrackClickedPlugin>() {
            app.add_plugins(TrackClickedPlugin);
        }

        cap_req.request_component_read::<TrackClicked>(app.world_mut());
        cap_req.request_resource_read::<TrackClickedEntitiesResource>(app.world_mut());
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
    fn pointer_click(&mut self) -> Option<&Pointer<Click>>;
}

impl<Cap: CapSet> ImmUiClicked for ImmEntity<'_, '_, '_, Cap>
where
    Cap: ImplCap<CapabilityUiClicked>,
{
    fn clicked(&mut self) -> bool {
        self.pointer_click().is_some()
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
        self.pointer_click().map(|event| event.button)
    }

    fn pointer_click(&mut self) -> Option<&Pointer<Click>> {
        'correct: {
            if !self.cap_entity_contains::<TrackClicked>() {
                break 'correct;
            }

            let click = self
                .cap_get_resource::<TrackClickedEntitiesResource>()
                .expect("Capability should be available")
                .into_inner()
                .clicked
                .get(&self.entity());
            return click;
        }

        // Fallback
        self.entity_commands().insert_if_new(TrackClicked);
        None
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
