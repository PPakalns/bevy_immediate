use std::ops::{Deref, DerefMut};

use bevy_ecs::{
    bundle::Bundle,
    entity::Entity,
    event::Event,
    hierarchy::ChildOf,
    schedule::IntoScheduleConfigs,
    system::{Commands, EntityCommands, IntoObserverSystem, Query, ResMut},
};

/// System set for systems that power `bevy_immediate` immediate mode functionality
#[derive(bevy_ecs::schedule::SystemSet, PartialEq, Eq, Clone, Debug, Hash)]
pub struct ImmediateSystemSet;

/// Plugin for immediate mode functionality in bevy
pub struct BevyImmediatePlugin;

impl bevy_app::Plugin for BevyImmediatePlugin {
    fn build(&self, app: &mut bevy_app::App) {
        app.add_systems(
            bevy_app::PostUpdate,
            immediate_mode_tracked_entity_upkeep_system.in_set(ImmediateSystemSet),
        );

        app.insert_resource(ImmediateModeStateResource::default());

        entity_mapping::init(app);
    }
}

mod entity_mapping;

mod immctx;
pub use immctx::ImmCtx;

mod immid;
pub use immid::{ImmId, ImmIdBuilder, imm_id};

/// Immediate mode manager that manages entity [`Self::current`]
///
/// Can be used to build new child entities with [`Self::build`] and similar methods.
pub struct Imm<'w, 's> {
    ctx: ImmCtx<'w, 's>,
    current: Current,
}

#[derive(Clone, Copy)]
struct Current {
    id: ImmId,
    entity: Option<Entity>,
    idx: usize,
}

impl<'w, 's> Imm<'w, 's> {
    /// Build new entity with auto generated id.
    ///
    /// Use [`Self::build_id`] if building entities that may not always exist when parent entity exists.
    ///
    /// Read more [`ImmId`], [`ImmIdBuilder`].
    pub fn child(&mut self) -> ImmEntityBuilder<'_, 'w, 's> {
        self.child_with_manual_id(ImmIdBuilder::Auto)
    }

    /// Build new entity with manually provided id that will be combined with parent entity id to
    /// make truly unique id.
    ///
    /// Read more [`ImmId`], [`ImmIdBuilder`].
    pub fn child_with_id<T: std::hash::Hash>(&mut self, id: T) -> ImmEntityBuilder<'_, 'w, 's> {
        self.child_with_manual_id(ImmIdBuilder::Hierarchy(ImmId::new(id)))
    }

    /// Build new entity with provided id.
    ///
    /// Read more [`ImmId`], [`ImmIdBuilder`].
    pub fn child_with_manual_id(&mut self, id: ImmIdBuilder) -> ImmEntityBuilder<'_, 'w, 's> {
        let id = id.resolve(self);

        let mut currently_creating = false;

        let entity = match self.ctx.mapping.id_to_entity.get(&id).copied() {
            Some(entity) => {
                if let Ok(mut qentity) = self.ctx.query.get_mut(entity) {
                    qentity.tracker.iteration = self.ctx.state.iteration;
                    if qentity.child_of.map(|ch| ch.parent()) != self.current.entity {
                        let mut entity_commands = self.ctx.commands.entity(entity);
                        match self.current.entity {
                            Some(entity) => {
                                entity_commands.insert(ChildOf(entity));
                            }
                            None => {
                                entity_commands.remove::<ChildOf>();
                            }
                        }
                    }
                }
                entity
            }
            None => {
                let mut commands = self.ctx.commands.spawn(ImmediateModeTrackerComponent {
                    id,
                    iteration: self.ctx.state.iteration,
                });
                if let Some(entity) = self.current.entity {
                    commands.insert(ChildOf(entity));
                }
                currently_creating = true;
                commands.id()
            }
        };

        ImmEntityBuilder {
            sui: self,
            id,
            currently_creating,
            entity,
        }
    }

    /// Manage entity with provided [`ImmId`] and [`Entity`] attributes with following logic
    fn add<R>(&mut self, id: ImmId, entity: Entity, f: impl FnOnce(&mut Imm<'w, 's>) -> R) -> R {
        self.add_dyn(id, entity, Box::new(f))
    }

    /// Manage entity with provided [`ImmId`] and [`Entity`] attributes with following logic
    fn add_dyn<R>(
        &mut self,
        id: ImmId,
        entity: Entity,
        f: Box<dyn FnOnce(&mut Imm<'w, 's>) -> R + '_>,
    ) -> R {
        let stored_current = self.current;

        self.current = Current {
            id: id,
            entity: Some(entity),
            idx: 0,
        };

        let resp = f(self);

        self.current = stored_current;

        resp
    }

    /// Entity that is currently being managed
    ///
    /// If building root of entity tree, this value may be [`None`]
    #[inline]
    pub fn current_entity(&self) -> Option<Entity> {
        self.current.entity
    }
}

/// Builder to build new entity that is managed by immediate mode logic
///
/// Construction should end with calls to [`Self::add`] or [`Self::add_empty`].
#[must_use]
pub struct ImmEntityBuilder<'r, 'w, 's> {
    sui: &'r mut Imm<'w, 's>,
    id: ImmId,
    entity: Entity,
    currently_creating: bool,
}

impl<'r, 'w, 's> ImmEntityBuilder<'r, 'w, 's> {
    /// Issue [`EntityCommands`] at this moment
    pub fn at_this_moment_apply_commands<F>(self, f: F) -> Self
    where
        F: FnOnce(&mut EntityCommands),
    {
        let mut entity_commands = self.sui.ctx.commands.entity(self.entity);
        f(&mut entity_commands);
        self
    }

    /// Issue [`EntityCommands`]
    /// (issued only when entity is created).
    pub fn on_spawn_apply_commands<F>(self, f: F) -> Self
    where
        F: FnOnce(&mut EntityCommands),
    {
        if self.currently_creating {
            self.at_this_moment_apply_commands(f)
        } else {
            self
        }
    }

    /// Issue [`EntityCommands`] if condition is met.
    /// (issued only when entity is created).
    pub fn on_spawn_apply_commands_if<F>(self, f: F, condition: impl FnOnce() -> bool) -> Self
    where
        F: FnOnce(&mut EntityCommands),
    {
        if self.currently_creating {
            if condition() {
                return self.at_this_moment_apply_commands(f);
            }
        }
        self
    }

    /// Insert [`Bundle`] similarly to [`EntityCommands::insert`].
    /// (inserted only when entity is created).
    pub fn on_spawn_insert<F, B>(self, f: F) -> Self
    where
        F: FnOnce() -> B,
        B: Bundle,
    {
        self.on_spawn_apply_commands(|commands| {
            commands.insert(f());
        })
    }

    /// Insert [`Bundle`] similarly to [`EntityCommands::insert_if`]
    /// (inserted only when entity is created).
    pub fn on_spawn_insert_if<F, B, Cond>(self, f: F, condition: impl FnOnce() -> bool) -> Self
    where
        F: FnOnce() -> B,
        B: Bundle,
    {
        self.on_spawn_apply_commands_if(
            |commands| {
                commands.insert(f());
            },
            condition,
        )
    }

    /// Insert [`Bundle`] similarly to [`EntityCommands::insert_if_new`].
    /// (inserted only when entity is created).
    pub fn on_spawn_insert_if_new<F, B>(self, f: F) -> Self
    where
        F: FnOnce() -> B,
        B: Bundle,
    {
        self.on_spawn_apply_commands(|commands| {
            commands.insert_if_new(f());
        })
    }

    /// Insert [`Bundle`] similarly to [`EntityCommands::insert_if_new_and`].
    /// (inserted only when entity is created).
    pub fn on_spawn_insert_if_new_and<F, B>(self, f: F, condition: impl FnOnce() -> bool) -> Self
    where
        F: FnOnce() -> B,
        B: Bundle,
    {
        self.on_spawn_apply_commands_if(
            |commands| {
                commands.insert_if_new(f());
            },
            condition,
        )
    }

    /// Observe with [`bevy_ecs::system::ObserverSystem`]
    /// (added only when entity is created).
    pub fn on_spawn_observe<E: Event, B: Bundle, M>(
        self,
        observer: impl IntoObserverSystem<E, B, M>,
    ) -> Self {
        self.on_spawn_apply_commands(|commands| {
            commands.observe(observer);
        })
    }

    /// If changed, insert [`Bundle`] into entity
    pub fn on_change_insert<F, B>(self, changed: bool, f: F) -> Self
    where
        F: FnOnce() -> B,
        B: Bundle,
    {
        if self.currently_creating || changed {
            let mut entity_commands = self.sui.ctx.commands.entity(self.entity);
            entity_commands.insert(f());
            self
        } else {
            self
        }
    }

    /// Finalize building of entity and provide immediate mode function to build descendants of this entity
    ///
    /// Function will return [`ImmReturn`] that can be used to check events
    pub fn add<R>(self, f: impl FnOnce(&mut Imm<'w, 's>) -> R) -> ImmReturn<'r, 'w, 's, R> {
        let resp = self.sui.add(self.id, self.entity, f);

        ImmReturn {
            inner: resp,
            resp: ImmEntity {
                ui: self.sui,
                entity: self.entity,
                currently_creating: self.currently_creating,
            },
        }
    }

    /// Finalize building of entity
    ///
    /// Function will return [`ImmReturn`] that can be used to check events
    pub fn add_empty(self) -> ImmEntity<'r, 'w, 's> {
        ImmEntity {
            ui: self.sui,
            entity: self.entity,
            currently_creating: self.currently_creating,
        }
    }
}

/// Stores return value of closure and builded entity response
pub struct ImmReturn<'r, 'w, 's, Inner> {
    /// Return value of closure that was provided to [`ImmBuilder::add`].
    pub inner: Inner,
    /// Stores information about entity that was built
    pub resp: ImmEntity<'r, 'w, 's>,
}

impl<'r, 'w, 's, Inner> Deref for ImmReturn<'r, 'w, 's, Inner> {
    type Target = ImmEntity<'r, 'w, 's>;

    fn deref(&self) -> &Self::Target {
        &self.resp
    }
}

impl<'r, 'w, 's, Inner> DerefMut for ImmReturn<'r, 'w, 's, Inner> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.resp
    }
}

/// Immediate mode response from entity that was built.
///
/// Can be used to look up relevant information:
/// * If entity was clicked,
/// * If entity was spawned,
/// * etc.
pub struct ImmEntity<'r, 'w, 's> {
    currently_creating: bool,
    entity: Entity,
    ui: &'r mut Imm<'w, 's>,
}

impl<'r, 'w, 's> ImmEntity<'r, 'w, 's> {
    /// Entity will be spawned when [`Commands`] will be processed.
    pub fn will_be_spawned(&self) -> bool {
        self.currently_creating
    }

    /// Entity clicked during last frame
    #[cfg(feature = "picking")]
    pub fn clicked(&mut self) -> bool {
        if let Ok(clicked) = self.ui.ctx.track_clicked_query.get(self.entity) {
            clicked.get()
        } else {
            use crate::ui::picking;

            let mut entity_commands = self.ui.ctx.commands.entity(self.entity);
            entity_commands
                .insert(picking::TrackClicked::default())
                .observe(picking::on_click);
            false
        }
    }
}

/// Component that is added to entities that are managed
/// by immediate mode system
#[derive(bevy_ecs::component::Component)]
pub struct ImmediateModeTrackerComponent {
    id: ImmId,
    iteration: u32,
}

#[derive(bevy_ecs::resource::Resource, Default)]
struct ImmediateModeStateResource {
    // Current iteration for unused entity removal
    iteration: u32,
}

fn immediate_mode_tracked_entity_upkeep_system(
    query: Query<(Entity, &ImmediateModeTrackerComponent)>,
    mut state: ResMut<ImmediateModeStateResource>,
    mut commands: Commands,
) {
    for (entity, marker) in query {
        if marker.iteration == state.iteration {
            continue;
        }
        commands.entity(entity).despawn();
    }

    state.iteration = state.iteration.wrapping_add(1);
}
