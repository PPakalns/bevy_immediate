use std::marker::PhantomData;

use crate::{CapAccessRequests, CapAccessRequestsResource, ImmCap};
use bevy_ecs::{
    bundle::Bundle,
    entity::Entity,
    event::Event,
    hierarchy::ChildOf,
    query::Without,
    system::{Commands, EntityCommands, IntoObserverSystem},
};

/// Plugin for immediate mode functionality in bevy
pub struct BevyImmediatePlugin<Cap = ()>(PhantomData<Cap>);

impl<Cap> BevyImmediatePlugin<Cap> {
    /// Construct plugin
    pub fn new() -> Self {
        Self(PhantomData)
    }
}

impl<Cap> Default for BevyImmediatePlugin<Cap> {
    fn default() -> Self {
        Self::new()
    }
}

impl<Cap> bevy_app::Plugin for BevyImmediatePlugin<Cap>
where
    Cap: ImmCap,
{
    fn build(&self, app: &mut bevy_app::App) {
        entity_mapping::init::<Cap>(app);
        upkeep::init::<Cap>(app);

        let mut capabilities = CapAccessRequests::<Cap>::default();
        Cap::build(app, &mut capabilities);
        app.insert_resource(CapAccessRequestsResource::new(capabilities));
    }
}

mod system_set;
pub use system_set::ImmediateSystemSet;

mod ctx;
pub use ctx::ImmCtx;

mod id;
pub use id::{ImmId, ImmIdBuilder, imm_id};

mod entity_mapping;
mod upkeep;

/// Immediate mode manager that manages entity [`Self::current`]
///
/// Can be used to build new child entities with [`Self::build`] and similar methods.
pub struct Imm<'w, 's, Cap: ImmCap> {
    ctx: ImmCtx<'w, 's, Cap>,
    current: Current,
}

#[derive(Clone, Copy)]
struct Current {
    id: ImmId,
    entity: Option<Entity>,
    idx: usize,
}

impl<'w, 's, Cap: ImmCap> Imm<'w, 's, Cap> {
    /// Build new entity with auto generated id.
    ///
    /// Use [`Self::build_id`] if building entities that may not always exist when parent entity exists.
    ///
    /// Read more [`ImmId`], [`ImmIdBuilder`].
    pub fn child(&mut self) -> ImmEntity<'_, 'w, 's, Cap> {
        self.child_with_manual_id(ImmIdBuilder::Auto)
    }

    /// Build new entity with manually provided id that will be combined with parent entity id to
    /// make truly unique id.
    ///
    /// Read more [`ImmId`], [`ImmIdBuilder`].
    pub fn child_with_id<T: std::hash::Hash>(&mut self, id: T) -> ImmEntity<'_, 'w, 's, Cap> {
        self.child_with_manual_id(ImmIdBuilder::Hierarchy(ImmId::new(id)))
    }

    /// Build new entity with provided id.
    ///
    /// Read more [`ImmId`], [`ImmIdBuilder`].
    pub fn child_with_manual_id(&mut self, id: ImmIdBuilder) -> ImmEntity<'_, 'w, 's, Cap> {
        let id = id.resolve(self);

        let mut currently_creating = false;

        let entity = match self.ctx.mapping.id_to_entity.get(&id).copied() {
            Some(entity) => {
                if let Ok(mut qentity) = self.ctx.entity_query.get_mut(entity) {
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
                let mut commands = self.ctx.commands.spawn((
                    // Add marker component that users can use in QueryFilter Without statements
                    ImmMarker::<Cap> {
                        id,
                        iteration: self.ctx.state.iteration,
                        _ph: PhantomData,
                    },
                ));

                if let Some(entity) = self.current.entity {
                    commands.insert(ChildOf(entity));
                }
                currently_creating = true;
                commands.id()
            }
        };

        ImmEntity {
            imm: self,
            id,
            entity,
            currently_creating,
        }
    }

    /// Manage entity with provided [`ImmId`] and [`Entity`] attributes with following logic
    fn add<R>(
        &mut self,
        id: ImmId,
        entity: Entity,
        f: impl FnOnce(&mut Imm<'w, 's, Cap>) -> R,
    ) -> R {
        self.add_dyn(id, entity, Box::new(f))
    }

    /// Manage entity with provided [`ImmId`] and [`Entity`] attributes with following logic
    #[allow(clippy::type_complexity)]
    fn add_dyn<R>(
        &mut self,
        id: ImmId,
        entity: Entity,
        f: Box<dyn FnOnce(&mut Imm<'w, 's, Cap>) -> R + '_>,
    ) -> R {
        let stored_current = self.current;

        self.current = Current {
            id,
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

    /// Retrieve access to commands
    #[inline]
    pub fn commands_mut(&mut self) -> &mut Commands<'w, 's> {
        &mut self.ctx.commands
    }

    /// Access underlaying context
    ///
    /// Useful for implementing additional [ImmCap]
    #[inline]
    pub fn ctx_mut(&mut self) -> &mut ImmCtx<'w, 's, Cap> {
        &mut self.ctx
    }

    /// Retrieve [`ImmCtx`] from which immediate mode entity tree was built
    pub fn deconstruct(self) -> ImmCtx<'w, 's, Cap> {
        self.ctx
    }
}

/// Immediate mode response from entity that was built.
///
/// Can be used to look up relevant information:
/// * If entity was clicked,
/// * If entity was spawned,
/// * etc.
pub struct ImmEntity<'r, 'w, 's, Cap: ImmCap> {
    imm: &'r mut Imm<'w, 's, Cap>,
    id: ImmId,
    entity: Entity,
    currently_creating: bool,
}

impl<'r, 'w, 's, Cap: ImmCap> ImmEntity<'r, 'w, 's, Cap> {
    /// Retrieve system param ctx for immediate mode
    pub fn ctx(&self) -> &ImmCtx<'w, 's, Cap> {
        &self.imm.ctx
    }

    /// Retrieve system param ctx for immediate mode
    pub fn ctx_mut(&mut self) -> &mut ImmCtx<'w, 's, Cap> {
        &mut self.imm.ctx
    }

    /// Retrieve [`Entity`] value for this entity
    pub fn entity(&self) -> Entity {
        self.entity
    }

    /// Gain access to [`Commands`]
    pub fn commands(&mut self) -> &mut Commands<'w, 's> {
        &mut self.imm.ctx.commands
    }

    /// Gain access to [`EntityCommands`] for this entity
    pub fn entity_commands(&mut self) -> EntityCommands<'_> {
        let commands = &mut self.imm.ctx.commands;
        commands.entity(self.entity)
    }

    /// Entity will be spawned when [`Commands`] will be processed.
    pub fn will_be_spawned(&self) -> bool {
        self.currently_creating
    }

    /// Issue [`EntityCommands`] at this moment
    pub fn at_this_moment_apply_commands<F>(self, f: F) -> Self
    where
        F: FnOnce(&mut EntityCommands),
    {
        let mut entity_commands = self.imm.ctx.commands.entity(self.entity);
        f(&mut entity_commands);
        self
    }

    /// Issue [`EntityCommands`] at this moment if condition is met
    pub fn at_this_moment_apply_commands_if<F>(self, f: F, condition: impl FnOnce() -> bool) -> Self
    where
        F: FnOnce(&mut EntityCommands),
    {
        if condition() {
            self.at_this_moment_apply_commands(f)
        } else {
            self
        }
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
            self.at_this_moment_apply_commands_if(f, condition)
        } else {
            self
        }
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
            let mut entity_commands = self.imm.ctx.commands.entity(self.entity);
            entity_commands.insert(f());
            self
        } else {
            self
        }
    }

    /// Finalize building of entity and provide immediate mode function to build descendants of this entity
    ///
    /// Function will return [`ImmReturn`] that can be used to check events
    #[allow(clippy::should_implement_trait)]
    pub fn add(self, f: impl FnOnce(&mut Imm<'w, 's, Cap>)) -> Self {
        self.imm.add(self.id, self.entity, f);
        self
    }

    /// Finalize building of entity and provide immediate mode function to build descendants of this entity
    ///
    /// Function will return [`ImmReturn`] that can be used to check events
    #[allow(clippy::should_implement_trait)]
    pub fn add_with_return<R>(self, f: impl FnOnce(&mut Imm<'w, 's, Cap>) -> R) -> (Self, R) {
        let value = self.imm.add(self.id, self.entity, f);

        (self, value)
    }
}

/// Component that is added to entities that are managed
/// by immediate mode system
///
/// Useful to add query filter [`WithoutImm<Cap>`] or [`bevy_ecs::query::Without<ImmMarker<Cap>>`]
/// to your queries.
#[derive(bevy_ecs::component::Component)]
pub struct ImmMarker<Cap> {
    id: ImmId,
    iteration: u32,
    _ph: PhantomData<Cap>,
}

/// Type to use in QueryFilter to avoid query collisions
pub type WithoutImm<Cap = ()> = Without<ImmMarker<Cap>>;
