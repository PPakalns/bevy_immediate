use bevy_ecs::{
    component::Component,
    observer::On,
    query::With,
    resource::Resource,
    system::{Query, Res, ResMut},
};
use bevy_picking::events::{Out, Over, Pointer};
use bevy_time::Time;

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

#[derive(Component)]
pub struct TooltipSource;

#[derive(Resource)]
pub struct TooltipGlobalSettings {
    pub tooltip_delay: f32,
    pub reset_delay: f32,
}

#[derive(Resource, Default)]
pub enum TooltipGlobalState {
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
    pub fn show_tooltip(&self) -> bool {
        matches!(self, TooltipGlobalState::Tooltip)
    }
}

fn on_source(
    pointer: On<Pointer<Over>>,
    query: Query<(), With<TooltipSource>>,
    mut res: ResMut<TooltipGlobalState>,
    time: Res<Time>,
) {
    if query.contains(pointer.entity) {
        match &*res {
            TooltipGlobalState::Nothing => {
                *res = TooltipGlobalState::Waiting {
                    since: time.elapsed_secs_f64(),
                };
            }
            TooltipGlobalState::Waiting { since: _ } => {}
            TooltipGlobalState::Tooltip => {
                *res = TooltipGlobalState::Tooltip;
            }
            TooltipGlobalState::Reset { since: _ } => {
                *res = TooltipGlobalState::Tooltip;
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
        *res = match *res {
            TooltipGlobalState::Nothing | TooltipGlobalState::Waiting { since: _ } => {
                TooltipGlobalState::Nothing
            }
            TooltipGlobalState::Tooltip => TooltipGlobalState::Reset {
                since: time.elapsed_secs_f64(),
            },
            TooltipGlobalState::Reset { since } => TooltipGlobalState::Reset { since },
        };
    }
}

fn update_tooltip_global_state(
    mut res: ResMut<TooltipGlobalState>,
    time: Res<Time>,
    global: Res<TooltipGlobalSettings>,
) {
    match &*res {
        TooltipGlobalState::Nothing => {}
        TooltipGlobalState::Waiting { since } => {
            if (time.elapsed_secs_f64() - since) > global.tooltip_delay as f64 {
                *res = TooltipGlobalState::Tooltip;
            }
        }
        TooltipGlobalState::Tooltip => {}
        TooltipGlobalState::Reset { since } => {
            if (time.elapsed_secs_f64() - since) > global.reset_delay as f64 {
                *res = TooltipGlobalState::Nothing;
            }
        }
    }
}
