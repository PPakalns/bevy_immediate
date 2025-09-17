use std::ops::Deref;

use bevy_ui::widget::Text;

use crate::{ImmCap, ImmEntity, ImmImplCap};

/// Capability that makes [`bevy_ui::Interaction`] accessible from immediate UI
pub struct ImmCapUiText;

impl ImmCap for ImmCapUiText {
    fn build<Cap: ImmCap>(app: &mut bevy_app::App, cap_req: &mut crate::CapAccessRequests<Cap>) {
        cap_req.request_optional_component::<Text>(app.world_mut(), true);
    }
}

/// Implements methods to access [`bevy_ui::Interaction`] in immediate mode
pub trait ImmUiText {
    /// Insert [`Text`] on entity spawn and update to given text upon change
    fn text(self, text: impl Deref<Target = str> + Into<String>) -> Self;

    /// On entity spawn insert given text into [`Text`]
    fn on_spawn_text(self, text: impl FnOnce() -> String) -> Self;
}

impl<Cap> ImmUiText for ImmEntity<'_, '_, '_, Cap>
where
    Cap: ImmImplCap<ImmCapUiText>,
{
    fn text(mut self, text: impl Deref<Target = str> + Into<String>) -> Self {
        if self.will_be_spawned() {
            return self.on_spawn_insert_if_new(|| Text(text.into()));
        }

        let entity = self.entity();

        let mut query = self.ctx_mut().query.get_query_mut::<Option<&mut Text>>();
        let mut query = query.query();

        let Ok(Some(mut text_comp)) = query.get_mut(entity) else {
            return self.at_this_moment_apply_commands(|commands| {
                commands.insert_if_new(Text(text.into()));
            });
        };

        // No need to update text and trigger state change
        if text_comp.0 == text.deref() {
            return self;
        }
        *text_comp = Text(text.into());
        self
    }

    fn on_spawn_text(self, text: impl FnOnce() -> String) -> Self {
        self.on_spawn_insert(|| Text(text()))
    }
}
