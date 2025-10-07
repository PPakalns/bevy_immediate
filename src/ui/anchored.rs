use bevy_app::{HierarchyPropagatePlugin, Propagate};
use bevy_picking::Pickable;
use bevy_ui::Node;

use crate::{
    CapSet, Imm, ImmCapability, ImmEntity, ImplCap,
    ui::{
        anchored_ui_plugin::{AnchorTarget, AnchoredUiPlugin},
        floating_ui_focus_plugin::{
            FloatingUiFocusPlugin, FocusDetectShouldClose, FocusParent, FocusShouldClose,
        },
        floating_ui_ordering_plugin::{FloatingUiOrderingPlugin, UiZOrderLayer},
        interaction::{CapabilityUiInteraction, ImmUiInteraction},
        tooltip_plugin::{TooltipGlobalState, TooltipPlugin, TooltipSource},
    },
};

/// Implements capability to create floatable, anchored elements
pub struct CapabilityUiAnchored;

impl ImmCapability for CapabilityUiAnchored {
    fn build<Cap: CapSet>(app: &mut bevy_app::App, cap_req: &mut crate::ImmCapAccessRequests<Cap>) {
        if !app.is_plugin_added::<AnchoredUiPlugin>() {
            app.add_plugins(AnchoredUiPlugin);
        }
        if !app.is_plugin_added::<FloatingUiFocusPlugin>() {
            app.add_plugins(FloatingUiFocusPlugin);
        }
        if !app.is_plugin_added::<FloatingUiOrderingPlugin>() {
            app.add_plugins(FloatingUiOrderingPlugin);
        }

        if !app.is_plugin_added::<HierarchyPropagatePlugin<Pickable>>() {
            app.add_plugins(HierarchyPropagatePlugin::<Pickable>::new(
                bevy_app::PostUpdate,
            ));
        }

        if !app.is_plugin_added::<TooltipPlugin>() {
            app.add_plugins(TooltipPlugin);
        }

        cap_req.request_resource_read::<TooltipGlobalState>(app.world_mut());
    }
}

/// Implements logic to create floating anchored elements
pub trait ImmUiAnchored<'w, 's, Caps: CapSet> {
    /// Show tooltip with given content
    fn with_tooltip(self, f: impl FnOnce(&mut Imm<'w, 's, Caps>)) -> Self
    where
        Caps: ImplCap<CapabilityUiInteraction>;

    /// Show tooltip with given content. In closure parent element is passed.
    ///
    /// Anchored entity itself is passed. It can be used to override default configuration.
    fn with_tooltip_container(self, f: impl FnOnce(ImmEntity<'_, 'w, 's, Caps>)) -> Self
    where
        Caps: ImplCap<CapabilityUiInteraction>;

    /// Add floating anchored entity that can be closed upon losing focus. Useful to implement dropdown.
    fn add_dropdown(self, on_close: impl FnOnce(), f: impl FnOnce(&mut Imm<'w, 's, Caps>)) -> Self;

    /// Add floating anchored entity that can be closed upon losing focus. Useful to implement dropdown.
    ///
    /// Anchored entity itself is passed. It can be used to override default configuration.
    fn add_dropdown_container(
        self,
        on_close: impl FnOnce(),
        f: impl FnOnce(ImmEntity<'_, 'w, 's, Caps>),
    ) -> Self;

    /// Add floating anchored  entity.
    ///
    /// Opening and closing should be fully managed manually.
    fn add_anchored(self, f: impl FnOnce(&mut Imm<'w, 's, Caps>)) -> Self;

    /// Add floating anchored  entity.
    ///
    /// Opening and closing should be fully managed manually.
    ///
    /// Anchored entity itself is passed. It can be used to override default configuration.
    fn add_anchored_container(self, f: impl FnOnce(ImmEntity<'_, 'w, 's, Caps>)) -> Self;
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
        if !self.cap_entity_contains::<TooltipSource>() {
            self.entity_commands().insert(TooltipSource);
        }

        if self.hovered()
            && self
                .cap_get_resource::<TooltipGlobalState>()
                .unwrap()
                .show_tooltip()
        {
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

    fn add_dropdown(self, on_close: impl FnOnce(), f: impl FnOnce(&mut Imm<'w, 's, Caps>)) -> Self {
        self.add_dropdown_container(on_close, |entity| {
            entity.add(|ui| {
                f(ui);
            });
        })
    }

    fn add_dropdown_container(
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

    fn add_anchored(self, f: impl FnOnce(&mut Imm<'w, 's, Caps>)) -> Self {
        self.add_anchored_container(|entity| {
            entity.add(|ui| {
                f(ui);
            });
        })
    }

    fn add_anchored_container(mut self, f: impl FnOnce(ImmEntity<'_, 'w, 's, Caps>)) -> Self {
        let entity = self.entity();
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
                    )
                });

                f(entity);
            });
        });

        self
    }
}
