use bevy_ecs::{
    component::Component,
    observer::On,
    query::With,
    resource::Resource,
    system::{Query, Res, ResMut},
};
use bevy_picking::events::{Out, Over, Pointer};
use bevy_time::Time;

/// Implements functionality to calculate when tooltip should be displayed
pub struct TooltipPlugin;

impl bevy_app::Plugin for TooltipPlugin {
    fn build(&self, app: &mut bevy_app::App) {
        app.insert_resource(TooltipGlobalSettings {
            tooltip_delay: 0.5,
            reset_delay: 0.5,
        });

        app.insert_resource(TooltipGlobalState::default());

        app.add_observer(on_source)
            .add_observer(out_sorce)
            .add_systems(bevy_app::PostUpdate, update_tooltip_global_state);
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
    #[default]
    Nothing,
    Waiting {
        since: f64,
    },
    Tooltip,
    Reset {
        since: f64,
    },
}

impl TooltipGlobalState {
    /// Returns if tooltip can be shown based on global tooltip timers.
    pub fn show_tooltip(&self) -> bool {
        matches!(self.state, TooltipGlobalStateInner::Tooltip)
    }
}

fn on_source(
    pointer: On<Pointer<Over>>,
    query: Query<(), With<TooltipSource>>,
    mut res: ResMut<TooltipGlobalState>,
    time: Res<Time>,
) {
    if query.contains(pointer.entity) {
        match &res.state {
            TooltipGlobalStateInner::Nothing => {
                res.state = TooltipGlobalStateInner::Waiting {
                    since: time.elapsed_secs_f64(),
                };
            }
            TooltipGlobalStateInner::Waiting { since: _ } => {}
            TooltipGlobalStateInner::Tooltip => {
                res.state = TooltipGlobalStateInner::Tooltip;
            }
            TooltipGlobalStateInner::Reset { since: _ } => {
                res.state = TooltipGlobalStateInner::Tooltip;
            }
        }
    }
}

fn out_sorce(
    pointer: On<Pointer<Out>>,
    query: Query<(), With<TooltipSource>>,
    mut res: ResMut<TooltipGlobalState>,
    time: Res<Time>,
) {
    if query.contains(pointer.entity) {
        res.state = match res.state {
            TooltipGlobalStateInner::Nothing | TooltipGlobalStateInner::Waiting { since: _ } => {
                TooltipGlobalStateInner::Nothing
            }
            TooltipGlobalStateInner::Tooltip => TooltipGlobalStateInner::Reset {
                since: time.elapsed_secs_f64(),
            },
            TooltipGlobalStateInner::Reset { since } => TooltipGlobalStateInner::Reset { since },
        };
    }
}

fn update_tooltip_global_state(
    mut res: ResMut<TooltipGlobalState>,
    time: Res<Time>,
    global: Res<TooltipGlobalSettings>,
) {
    match &res.state {
        TooltipGlobalStateInner::Nothing => {}
        TooltipGlobalStateInner::Waiting { since } => {
            if (time.elapsed_secs_f64() - since) > global.tooltip_delay as f64 {
                res.state = TooltipGlobalStateInner::Tooltip;
            }
        }
        TooltipGlobalStateInner::Tooltip => {}
        TooltipGlobalStateInner::Reset { since } => {
            if (time.elapsed_secs_f64() - since) > global.reset_delay as f64 {
                res.state = TooltipGlobalStateInner::Nothing;
            }
        }
    }
}
