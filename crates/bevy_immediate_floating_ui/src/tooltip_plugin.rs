use bevy_ecs::{
    component::Component,
    entity::Entity,
    hierarchy::ChildOf,
    observer::On,
    query::With,
    resource::Resource,
    system::{Query, Res, ResMut},
};
use bevy_picking::events::{Move, Pointer};
use bevy_time::{Real, Time};

/// Implements functionality to calculate when tooltip should be displayed
pub struct TooltipPlugin;

impl bevy_app::Plugin for TooltipPlugin {
    fn build(&self, app: &mut bevy_app::App) {
        app.insert_resource(TooltipGlobalSettings {
            tooltip_delay: 0.5,
            reset_delay: 0.5,
            pointer_changed_delay: 0.1,
        });

        app.insert_resource(TooltipGlobalState::default());

        app.add_observer(on_mouse_move);

        app.add_systems(bevy_app::PostUpdate, update_tooltip_global_state);
    }
}

/// Marks entity as tooltip source. When hovered,
///
/// it will count towards timers for displaying tooltip.
#[derive(Component)]
pub struct TooltipSource;

/// Global settings when tooltips should be displayed
#[derive(Resource)]
pub struct TooltipGlobalSettings {
    /// Delay in seconds until tooltip is shown when ui element with tooltip is hovered
    pub tooltip_delay: f32,

    /// When [`Self::tooltip_delay`] has elapsed. If pointer changes ui element with
    /// tooltip, show tooltip after pointer changed delay.
    ///
    /// Useful to avoid flickering when moving mouse.
    pub pointer_changed_delay: f32,

    /// If [`Self::tooltip_delay`] has elapsed. Tooltips will be shown immediatelly.
    /// Reset delay will reset tooltip delay, when no tooltip source is hovered for reset delay
    /// amount of seconds.
    pub reset_delay: f32,
}

/// Stores if tooltips should be shown
#[derive(Resource, Default)]
pub struct TooltipGlobalState {
    state: TooltipGlobalStateInner,
}

#[derive(Default)]
enum TooltipGlobalStateInner {
    /// Tooltip inactive
    #[default]
    Nothing,
    /// Cursor waiting on tooltip
    Waiting { since: f64, wait_entity: Entity },
    /// Tooltip was active a moment ago, new tooltip will be shown with different rule
    PointerWaiting { since: f64, wait_entity: Entity },
    /// Tooltip active for entity
    Tooltip { wait_entity: Entity },
    /// When cursor left tooltip area
    Reset { since: f64 },
}

impl TooltipGlobalState {
    /// Returns if tooltip can be shown based on global tooltip timers.
    pub fn show_tooltip(&self) -> Option<Entity> {
        match self.state {
            TooltipGlobalStateInner::Nothing => None,
            TooltipGlobalStateInner::Waiting {
                since: _,
                wait_entity: _,
            }
            | TooltipGlobalStateInner::PointerWaiting {
                since: _,
                wait_entity: _,
            }
            | TooltipGlobalStateInner::Reset { since: _ } => None,
            TooltipGlobalStateInner::Tooltip { wait_entity } => Some(wait_entity),
        }
    }
}

fn on_mouse_move(
    pointer: On<Pointer<Move>>,
    query: Query<(), With<TooltipSource>>,
    q_parents: Query<&ChildOf>,
    mut res: ResMut<TooltipGlobalState>,
    time: Res<Time<Real>>,
) {
    if pointer.entity != pointer.original_event_target() {
        return;
    }

    let tooltip_entity = std::iter::once(pointer.entity)
        .chain(q_parents.iter_ancestors(pointer.entity))
        .find(|entity| query.contains(*entity));

    if let Some(wait_entity) = tooltip_entity {
        match &mut res.state {
            TooltipGlobalStateInner::Nothing => {
                res.state = TooltipGlobalStateInner::Waiting {
                    since: time.elapsed_secs_f64(),
                    wait_entity,
                };
            }
            TooltipGlobalStateInner::Waiting {
                since,
                wait_entity: old_entity,
            } => {
                if *old_entity != wait_entity {
                    res.state = TooltipGlobalStateInner::Waiting {
                        since: time.elapsed_secs_f64(),
                        wait_entity,
                    };
                }
            }
            TooltipGlobalStateInner::PointerWaiting {
                since,
                wait_entity: old_entity,
            } => {
                if *old_entity != wait_entity {
                    res.state = TooltipGlobalStateInner::PointerWaiting {
                        since: time.elapsed_secs_f64(),
                        wait_entity,
                    };
                }
            }
            TooltipGlobalStateInner::Tooltip {
                wait_entity: old_entity,
            } => {
                if *old_entity != wait_entity {
                    // Pointer moved to another entity
                    res.state = TooltipGlobalStateInner::PointerWaiting {
                        since: time.elapsed_secs_f64(),
                        wait_entity,
                    };
                }
            }
            TooltipGlobalStateInner::Reset { since: _ } => {
                res.state = TooltipGlobalStateInner::PointerWaiting {
                    since: time.elapsed_secs_f64(),
                    wait_entity,
                };
            }
        }
    } else {
        match res.state {
            TooltipGlobalStateInner::Nothing
            | TooltipGlobalStateInner::Waiting {
                since: _,
                wait_entity: _,
            } => {
                res.state = TooltipGlobalStateInner::Nothing;
            }
            TooltipGlobalStateInner::PointerWaiting {
                since: _,
                wait_entity: _,
            }
            | TooltipGlobalStateInner::Tooltip { wait_entity: _ } => {
                res.state = TooltipGlobalStateInner::Reset {
                    since: time.elapsed_secs_f64(),
                };
            }
            TooltipGlobalStateInner::Reset { since: _ } => {}
        };
    }
}

fn update_tooltip_global_state(
    mut res: ResMut<TooltipGlobalState>,
    time: Res<Time<Real>>,
    global: Res<TooltipGlobalSettings>,
) {
    match &res.state {
        TooltipGlobalStateInner::Nothing => {}
        TooltipGlobalStateInner::Waiting { since, wait_entity } => {
            if (time.elapsed_secs_f64() - since) > global.tooltip_delay as f64 {
                res.state = TooltipGlobalStateInner::Tooltip {
                    wait_entity: *wait_entity,
                };
            }
        }
        TooltipGlobalStateInner::PointerWaiting { since, wait_entity } => {
            if (time.elapsed_secs_f64() - since) > global.pointer_changed_delay as f64 {
                res.state = TooltipGlobalStateInner::Tooltip {
                    wait_entity: *wait_entity,
                };
            }
        }
        TooltipGlobalStateInner::Tooltip { wait_entity: _ } => {}
        TooltipGlobalStateInner::Reset { since } => {
            if (time.elapsed_secs_f64() - since) > global.reset_delay as f64 {
                res.state = TooltipGlobalStateInner::Nothing;
            }
        }
    }
}
