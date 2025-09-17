use bevy_ecs::schedule::IntoScheduleConfigs;

use crate::{ImmCap, ImmEntity, ImmImplCap, ImmediateSystemSet, impl_capabilities};

/// Defines capability that contains all Ui capabilities from this crate
pub struct ImmCapUi;

#[cfg(feature = "picking")]
impl_capabilities!(
    ImmCapUi,
    (
        ImmCapUiBase,
        ImmCapUiInteraction,
        picking::ImmCapUiPickingAll
    )
);

#[cfg(not(feature = "picking"))]
impl_capabilities!(ImmUiCap, (ImmUiBaseCap, ImmCapUiInteraction));

////////////////////////////////////////////////////////////////////////////////

/// Base capability for UI that sets up correct order of immediate system execution
pub struct ImmCapUiBase;

impl ImmCap for ImmCapUiBase {
    fn build<Cap: ImmCap>(app: &mut bevy_app::App, cap_req: &mut crate::CapAccessRequests<Cap>) {
        app.configure_sets(
            bevy_app::PostUpdate,
            ImmediateSystemSet::<Cap>::default().before(bevy_ui::UiSystem::Prepare),
        );

        let _ = cap_req;
    }
}

/// Contains API extensions for ergonomic API that use [`bevy_picking`]
#[cfg(feature = "picking")]
pub mod picking;

////////////////////////////////////////////////////////////////////////////////

/// Capability that makes [`bevy_ui::Interaction`] accessible from immediate UI
pub struct ImmCapUiInteraction;

impl ImmCap for ImmCapUiInteraction {
    fn build<Cap: ImmCap>(app: &mut bevy_app::App, cap_req: &mut crate::CapAccessRequests<Cap>) {
        cap_req.request_optional_component::<bevy_ui::Interaction>(app.world_mut(), false);
    }
}

/// Implements methods to access [`bevy_ui::Interaction`] in immediate mode
pub trait ImmUiInteraction {
    /// Retrieve current [`bevy_ui::Interaction`] state for entity
    fn interaction(&mut self) -> bevy_ui::Interaction;

    /// Is [`bevy_ui::Interaction::Pressed`]
    fn pressed(&mut self) -> bool;

    /// Is [`bevy_ui::Interaction::Hovered`]
    fn hovered(&mut self) -> bool;
}

impl<Cap> ImmUiInteraction for ImmEntity<'_, '_, '_, Cap>
where
    Cap: ImmImplCap<ImmCapUiInteraction>,
{
    fn interaction(&mut self) -> bevy_ui::Interaction {
        let entity = self.entity();

        let mut query = self
            .ctx()
            .params
            .get_query::<Option<&bevy_ui::Interaction>>();

        match query.query().get(entity) {
            Ok(Some(entity)) => *entity,
            Ok(None) | Err(_) => {
                self.entity_commands()
                    .insert_if_new(bevy_ui::Interaction::default());
                bevy_ui::Interaction::None
            }
        }
    }

    fn pressed(&mut self) -> bool {
        self.interaction() == bevy_ui::Interaction::Pressed
    }

    fn hovered(&mut self) -> bool {
        self.interaction() == bevy_ui::Interaction::Hovered
    }
}
