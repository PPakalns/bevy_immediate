use bevy_ecs::{
    entity::Entity,
    observer::On,
    query::{Has, With},
    system::{Query, ResMut},
};
use bevy_input::{
    ButtonState,
    keyboard::{KeyCode, KeyboardInput},
};
use bevy_input_focus::FocusedInput;
use bevy_picking::events::{Click, Pointer};
use bevy_platform::collections::HashSet;
use bevy_ui::InteractionDisabled;

use crate::{CapSet, ImmCapAccessRequests, ImmCapability, ImmEntity, ImplCap};

/// Immediate mode capability for `.activated()`
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct CapabilityUiActivated;

impl ImmCapability for CapabilityUiActivated {
    fn build<CM: CapSet>(app: &mut bevy_app::App, cap_req: &mut ImmCapAccessRequests<CM>) {
        if !app.is_plugin_added::<TrackActivatedPlugin>() {
            app.add_plugins(TrackActivatedPlugin);
        }

        cap_req.request_component_read::<TrackActivated>(app.world_mut());
        cap_req.request_resource_read::<TrackActivetedEntitiesResource>(app.world_mut());
    }
}

/// Implements support for `.activated()`
pub trait ImmUiActivated {
    /// Was widget activated
    fn activated(&mut self) -> bool;
}

impl<Cap: CapSet> ImmUiActivated for ImmEntity<'_, '_, '_, Cap>
where
    Cap: ImplCap<CapabilityUiActivated>,
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
pub struct TrackActivatedPlugin;

impl bevy_app::Plugin for TrackActivatedPlugin {
    fn build(&self, app: &mut bevy_app::App) {
        app.insert_resource(TrackActivetedEntitiesResource::default());
        app.add_systems(bevy_app::First, reset_activated);
        app.add_observer(button_on_key_event)
            .add_observer(button_on_pointer_click);
    }
}

/// Tracks if entity has been activated in this frame.
#[derive(bevy_ecs::component::Component, Default)]
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
// Hopefully there will be Activated trigger in future.
// This should be implemented in bevy 0.18

fn button_on_key_event(
    event: On<FocusedInput<KeyboardInput>>,
    q_state: Query<Has<InteractionDisabled>, With<TrackActivated>>,
    mut activated: ResMut<TrackActivetedEntitiesResource>,
) {
    let Ok(disabled) = q_state.get(event.focused_entity) else {
        return;
    };

    if !disabled {
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
    mut q_state: Query<Has<InteractionDisabled>, With<TrackActivated>>,
    mut activated: ResMut<TrackActivetedEntitiesResource>,
) {
    let Ok(disabled) = q_state.get_mut(click.entity) else {
        return;
    };

    click.propagate(false);
    if !disabled {
        activated.activated.insert(click.entity);
    }
}
