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
        'text_exists: {
            let Ok(mut entity) = self.get_entity_mut() else {
                break 'text_exists;
            };

            let Some(mut text_comp) = entity.get_mut::<Text>() else {
                break 'text_exists;
            };

            // No need to update text and trigger state change
            if text_comp.0 == text.deref() {
                return self;
            }
            *text_comp = Text(text.into());

            return self;
        }

        self.at_this_moment_apply_commands(|commands| {
            commands.insert_if_new(Text(text.into()));
        })
    }

    fn on_spawn_text(self, text: impl FnOnce() -> String) -> Self {
        self.on_spawn_insert(|| Text(text()))
    }
}
