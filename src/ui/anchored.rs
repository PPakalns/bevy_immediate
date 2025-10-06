use bevy_app::{HierarchyPropagatePlugin, Propagate};
use bevy_picking::Pickable;
use bevy_ui::Node;

use crate::{
    CapSet, Imm, ImmCapability, ImmEntity, ImplCap,
    ui::{
        anchored_entity_plugin::{AnchorTarget, AnchoredEntityPlugin},
        floating_entity_focus_plugin::{
            FloatingEntityFocusPlugin, FocusDetectShouldClose, FocusParent, FocusShouldClose,
        },
        floating_entity_plugin::{FloatingEntityPlugin, UiZOrderLayer},
        interaction::{CapabilityUiInteraction, ImmUiInteraction},
    },
};

/// Implements capability to create floatable, anchored elements
pub struct CapabilityUiAnchored;

impl ImmCapability for CapabilityUiAnchored {
    fn build<Cap: CapSet>(
        app: &mut bevy_app::App,
        _cap_req: &mut crate::ImmCapAccessRequests<Cap>,
    ) {
        if !app.is_plugin_added::<AnchoredEntityPlugin>() {
            app.add_plugins(AnchoredEntityPlugin);
        }
        if !app.is_plugin_added::<FloatingEntityFocusPlugin>() {
            app.add_plugins(FloatingEntityFocusPlugin);
        }
        if !app.is_plugin_added::<FloatingEntityPlugin>() {
            app.add_plugins(FloatingEntityPlugin);
        }

        if !app.is_plugin_added::<HierarchyPropagatePlugin<Pickable>>() {
            app.add_plugins(HierarchyPropagatePlugin::<Pickable>::new(
                bevy_app::PostUpdate,
            ));
        }
    }
}

pub trait ImmUiAnchored<'w, 's, Caps: CapSet> {
    fn with_tooltip(self, f: impl FnOnce(&mut Imm<'w, 's, Caps>)) -> Self
    where
        Caps: ImplCap<CapabilityUiInteraction>;
    fn with_tooltip_container(self, f: impl FnOnce(ImmEntity<'_, 'w, 's, Caps>)) -> Self
    where
        Caps: ImplCap<CapabilityUiInteraction>;

    fn with_dropdown(self, on_close: impl FnOnce(), f: impl FnOnce(&mut Imm<'w, 's, Caps>))
    -> Self;
    fn with_dropdown_container(
        self,
        on_close: impl FnOnce(),
        f: impl FnOnce(ImmEntity<'_, 'w, 's, Caps>),
    ) -> Self;
}

impl<'w, 's, Caps> ImmUiAnchored<'w, 's, Caps> for ImmEntity<'_, 'w, 's, Caps>
where
    Caps: ImplCap<CapabilityUiAnchored>,
{
    fn with_tooltip(self, f: impl FnOnce(&mut Imm<'w, 's, Caps>)) -> Self
    where
        Caps: ImplCap<CapabilityUiInteraction>,
    {
        self.with_tooltip_container(|entity| {
            entity.add(|ui| {
                f(ui);
            });
        })
    }

    fn with_tooltip_container(mut self, f: impl FnOnce(ImmEntity<'_, 'w, 's, Caps>)) -> Self
    where
        Caps: ImplCap<CapabilityUiInteraction>,
    {
        if self.hovered() {
            let entity = self.entity();
            self = self.add(|ui| {
                ui.unrooted("with_tooltip", |ui| {
                    let entity = ui.ch().on_spawn_insert(|| {
                        (
                            Node {
                                position_type: bevy_ui::PositionType::Absolute,
                                ..Default::default()
                            },
                            AnchorTarget::Entity(entity),
                            UiZOrderLayer::Tooltip,
                            FocusParent(entity),
                            Propagate(Pickable {
                                should_block_lower: false,
                                is_hoverable: false,
                            }),
                        )
                    });
                    f(entity);
                });
            });
        }
        self
    }

    fn with_dropdown(
        self,
        on_close: impl FnOnce(),
        f: impl FnOnce(&mut Imm<'w, 's, Caps>),
    ) -> Self {
        self.with_dropdown_container(on_close, |entity| {
            entity.add(|ui| {
                f(ui);
            });
        })
    }

    fn with_dropdown_container(
        mut self,
        on_close: impl FnOnce(),
        f: impl FnOnce(ImmEntity<'_, 'w, 's, Caps>),
    ) -> Self {
        let entity = self.entity();
        let mut should_close = false;
        self = self.add(|ui| {
            ui.unrooted("with_dropdown", |ui| {
                let entity = ui.ch().on_spawn_insert(|| {
                    (
                        Node {
                            position_type: bevy_ui::PositionType::Absolute,
                            ..Default::default()
                        },
                        UiZOrderLayer::Dropdown,
                        AnchorTarget::Entity(entity),
                        FocusParent(entity),
                        FocusDetectShouldClose,
                    )
                });

                if entity.cap_entity_contains::<FocusShouldClose>() {
                    should_close = true;
                }

                f(entity);
            });
        });

        // Should close is called at the end (1 frame delay)
        // to process possible updates in dropdown that could
        // have called dropdown to close.
        if should_close {
            on_close();
        }

        self
    }
}
