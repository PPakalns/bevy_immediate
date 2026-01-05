use bevy_ecs::{
    entity::Entity,
    observer::On,
    query::{Has, With, Without},
    system::{Commands, Query},
};
use bevy_input::{
    ButtonState,
    keyboard::{KeyCode, KeyboardInput},
};
use bevy_input_focus::FocusedInput;
use bevy_picking::events::{Click, Pointer};
use bevy_ui::{Checked, InteractionDisabled};
use bevy_ui_widgets::{RadioButton, RadioGroup, ValueChange};

/// Plugin adds code to make RadioButtons functional independently
/// and consistently with all other widgets
pub struct RadioButtonFixPlugin;

impl bevy_app::Plugin for RadioButtonFixPlugin {
    fn build(&self, app: &mut bevy_app::App) {
        app.add_observer(radio_button_on_click)
            .add_observer(radio_button_on_key_input)
            .add_observer(radio_group_to_radio_button_event);
    }
}

fn radio_group_to_radio_button_event(
    ev: On<ValueChange<Entity>>,
    q_radio_group: Query<(), With<RadioGroup>>,
    mut commands: Commands,
) {
    if !q_radio_group.contains(ev.source) {
        // Not a radio group
        return;
    };

    commands.trigger(ValueChange {
        source: ev.value,
        value: true,
    });
}

// Provides functionality for standalone focusable [`RadioButton`] to react
// on `Space` or `Enter` key press.
fn radio_button_on_key_input(
    ev: On<FocusedInput<KeyboardInput>>,
    q_radio_button: Query<Has<Checked>, (With<RadioButton>, Without<InteractionDisabled>)>,
    mut commands: Commands,
) {
    let Ok(checked) = q_radio_button.get(ev.focused_entity) else {
        // Not a radio button
        return;
    };

    // Radio button already checked
    if checked {
        return;
    }

    let event = &ev.event().input;
    if event.state == ButtonState::Pressed
        && !event.repeat
        && (event.key_code == KeyCode::Enter || event.key_code == KeyCode::Space)
    {
        commands.trigger(ValueChange::<bool> {
            source: ev.focused_entity,
            value: true,
        });
    }
}

fn radio_button_on_click(
    ev: On<Pointer<Click>>,
    q_radio: Query<Has<Checked>, (With<RadioButton>, Without<InteractionDisabled>)>,
    mut commands: Commands,
) {
    let Ok(checked) = q_radio.get(ev.entity) else {
        // Not a radio button
        return;
    };

    // Radio button is already checked
    if checked {
        return;
    }

    commands.trigger(ValueChange::<bool> {
        source: ev.entity,
        value: true,
    });
}
