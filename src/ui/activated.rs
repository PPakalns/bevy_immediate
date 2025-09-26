use bevy_ecs::{
    entity::Entity,
    lifecycle,
    observer::On,
    query::Has,
    system::{Commands, Query, ResMut},
};
use bevy_input::{
    ButtonState,
    keyboard::{KeyCode, KeyboardInput},
};
use bevy_input_focus::FocusedInput;
use bevy_picking::events::{Click, Pointer};
use bevy_platform::collections::HashSet;
use bevy_ui::{InteractionDisabled, Pressed};

use crate::{CapSet, ImmCapAccessRequests, ImmCapability, ImmEntity, ImplCap};

/// Immediate mode capability for `.clicked()`
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct CapabilityUiClicked;

impl ImmCapability for CapabilityUiClicked {
    fn build<CM: CapSet>(app: &mut bevy_app::App, cap_req: &mut ImmCapAccessRequests<CM>) {
        if !app.is_plugin_added::<TrackClickedPlugin>() {
            app.add_plugins(TrackClickedPlugin);
        }

        cap_req.request_component_read::<TrackActivated>(app.world_mut());
        cap_req.request_resource_read::<TrackActivetedEntitiesResource>(app.world_mut());
    }
}

/// Implements support for `.clicked()`
pub trait ImmUiActivated {
    /// Was button activated
    fn activated(&mut self) -> bool;
}

impl<Cap: CapSet> ImmUiActivated for ImmEntity<'_, '_, '_, Cap>
where
    Cap: ImplCap<CapabilityUiClicked>,
{
    fn activated(&mut self) -> bool {
        'correct: {
            if !self.cap_entity_contains::<TrackActivated>() {
                break 'correct;
            }

            let activated = self
                .cap_get_resource::<TrackActivetedEntitiesResource>()
                .expect("Capability should be available")
                .into_inner()
                .activated
                .contains(&self.entity());
            return activated;
        }

        // Fallback
        self.entity_commands().insert_if_new(TrackActivated);
        false
    }
}

////////////////////////////////////////////////////////////////////////////////

/// Add click tracking related logic
pub struct TrackClickedPlugin;

impl bevy_app::Plugin for TrackClickedPlugin {
    fn build(&self, app: &mut bevy_app::App) {
        app.insert_resource(TrackActivetedEntitiesResource::default());
        app.add_systems(bevy_app::First, reset_activated);
        app.add_observer(track_clicked_insert);
    }
}

// Insert on_click picking observer only once
fn track_clicked_insert(trigger: On<lifecycle::Add, TrackActivated>, mut commands: Commands) {
    let entity = trigger.event().entity;
    commands
        .entity(entity)
        .observe(button_on_key_event)
        .observe(button_on_pointer_click);
}

/// Tracks if entity has been clicked in this frame.
#[derive(bevy_ecs::component::Component, Default)]
#[component(storage = "SparseSet")]
pub struct TrackActivated;

#[derive(bevy_ecs::resource::Resource, Default)]
struct TrackActivetedEntitiesResource {
    pub activated: HashSet<Entity>,
}

fn reset_activated(mut res: ResMut<TrackActivetedEntitiesResource>) {
    res.activated.clear();
}

// Code duplicated from
// https://docs.rs/bevy_ui_widgets/latest/src/bevy_ui_widgets/button.rs.html#26-30
//
// Hopefully there will be Activated trigger in future

fn button_on_key_event(
    event: On<FocusedInput<KeyboardInput>>,
    q_state: Query<Has<InteractionDisabled>>,
    mut activated: ResMut<TrackActivetedEntitiesResource>,
) {
    if let Ok(disabled) = q_state.get(event.focused_entity)
        && !disabled
    {
        let input_event = &event.input;
        if !input_event.repeat
            && input_event.state == ButtonState::Pressed
            && (input_event.key_code == KeyCode::Enter || input_event.key_code == KeyCode::Space)
        {
            activated.activated.insert(event.focused_entity);
        }
    }
}

fn button_on_pointer_click(
    mut click: On<Pointer<Click>>,
    mut q_state: Query<(Has<Pressed>, Has<InteractionDisabled>)>,
    mut activated: ResMut<TrackActivetedEntitiesResource>,
) {
    if let Ok((pressed, disabled)) = q_state.get_mut(click.entity) {
        click.propagate(false);
        if pressed && !disabled {
            activated.activated.insert(click.entity);
        }
    }
}
