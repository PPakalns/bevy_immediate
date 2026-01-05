use std::marker::PhantomData;

use bevy_ecs::{component::Component, lifecycle, observer::On, system::Commands, world::Mut};

/// Add click tracking related logic
pub struct TrackValueChangePlugin<T> {
    _ph: PhantomData<T>,
}

impl<T> Default for TrackValueChangePlugin<T> {
    fn default() -> Self {
        Self {
            _ph: Default::default(),
        }
    }
}

impl<T> bevy_app::Plugin for TrackValueChangePlugin<T>
where
    T: Sync + Send + 'static + Copy + Clone,
{
    fn build(&self, app: &mut bevy_app::App) {
        app.add_observer(track_inserted::<T>);
    }
}

/// Stores unconsumed value from last value change entity event for this component
#[derive(Component)]
pub struct NewValueChange<T> {
    value: Option<T>,
}

impl<T> Default for NewValueChange<T> {
    fn default() -> Self {
        Self { value: None }
    }
}

impl<T> NewValueChange<T> {
    /// Consume value from last value change entity event
    #[inline]
    pub fn take(this: &mut Mut<'_, Self>) -> Option<T> {
        if this.value.is_some() {
            this.value.take()
        } else {
            None
        }
    }

    /// Retrieve unconsumed stored value from last value change entity event
    pub fn value(&self) -> Option<&T> {
        self.value.as_ref()
    }
}

// Insert on_click picking observer only once
#[allow(unused_mut)]
fn track_inserted<T>(trigger: On<lifecycle::Add, NewValueChange<T>>, mut commands: Commands)
where
    T: Sync + Send + 'static + Copy + Clone,
{
    #[allow(unused)]
    let entity = trigger.event().entity;

    #[cfg(feature = "bevy_ui_widgets")]
    commands
        .entity(entity)
        .observe(bevy_ui_widgets_impl::on_value_change::<T>);

    let _ = commands;
}

#[cfg(feature = "bevy_ui_widgets")]
mod bevy_ui_widgets_impl {
    use bevy_ecs::{observer::On, system::Query};

    use crate::track_value_change_plugin::NewValueChange;

    pub fn on_value_change<T>(
        trigger: On<bevy_ui_widgets::ValueChange<T>>,
        mut query: Query<&mut NewValueChange<T>>,
    ) where
        T: Sync + Send + 'static + Copy + Clone,
    {
        let entity = trigger.event().source;
        if let Ok(mut comp) = query.get_mut(entity) {
            comp.value = Some(trigger.value);
        }
    }
}
