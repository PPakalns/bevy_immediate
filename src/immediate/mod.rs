use std::marker::PhantomData;

use crate::{CapAccessRequests, CapAccessRequestsResource, ImmCap};
use bevy_ecs::{
    bundle::Bundle,
    component::{Component, Mutable},
    entity::Entity,
    event::Event,
    hierarchy::ChildOf,
    query::{QueryEntityError, With, Without},
    resource::Resource,
    system::{Commands, EntityCommands, IntoObserverSystem, Query},
    world::{FilteredEntityRef, error::ResourceFetchError},
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

/// Helper type to more easily write queries
pub type ImmQuery<'w, 's, Cap, D, F = ()> = Query<'w, 's, D, (Without<ImmMarker<Cap>>, F)>;
pub(crate) type ImmQueryInternal<'w, 's, Cap, D, F = ()> =
    Query<'w, 's, D, (With<ImmMarker<Cap>>, F)>;

/// Immediate mode in a state where child components can be added
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
    pub fn child_id<T: std::hash::Hash>(&mut self, id: T) -> ImmEntity<'_, 'w, 's, Cap> {
        self.child_with_manual_id(ImmIdBuilder::Hierarchy(ImmId::new(id)))
    }

    /// Build new entity with provided id.
    ///
    /// Read more [`ImmId`], [`ImmIdBuilder`].
    pub fn child_with_manual_id(&mut self, id: ImmIdBuilder) -> ImmEntity<'_, 'w, 's, Cap> {
        let id = id.resolve(self);

        let mut will_be_spawned = false;

        let entity = 'entity_retrieval: {
            'entity_full_reuse: {
                let Some(entity) = self.ctx.mapping.id_to_entity.get(&id).copied() else {
                    break 'entity_full_reuse;
                };

                let Ok(mut qentity) = self.ctx.entity_query.get_mut(entity) else {
                    break 'entity_full_reuse;
                };

                // Update iteration for entity upkeep tracking
                qentity.tracker.iteration = self.ctx.state.iteration;

                if qentity.child_of.map(|ch| ch.parent()) != self.current.entity {
                    // Parent changed
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

                break 'entity_retrieval entity;
            }

            // Spawn entity by default if valid entity not found
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
            will_be_spawned = true;
            commands.id()
        };

        ImmEntity {
            imm: self,
            id,
            entity,
            will_be_spawned,
        }
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

    /// Manage entity with provided [`ImmId`] and [`Entity`] attributes with provided closure
    fn add<R>(
        &mut self,
        id: ImmId,
        entity: Entity,
        f: impl FnOnce(&mut Imm<'w, 's, Cap>) -> R,
    ) -> R {
        self.add_dyn(id, entity, Box::new(f))
    }

    /// Manage entity with provided [`ImmId`] and [`Entity`] attributes with provided closure
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
}

/// Entity during construction in immediate mode approach
///
/// Can be used to issue commands and check such conditions as `.clicked()`.
pub struct ImmEntity<'r, 'w, 's, Cap: ImmCap> {
    imm: &'r mut Imm<'w, 's, Cap>,
    id: ImmId,
    entity: Entity,
    will_be_spawned: bool,
}

impl<'r, 'w, 's, Cap: ImmCap> ImmEntity<'r, 'w, 's, Cap> {
    /// Build descendants of this entity
    ///
    /// If closure return value is needed, use `[Self::add_with_return]``
    #[allow(clippy::should_implement_trait)]
    pub fn add(self, f: impl FnOnce(&mut Imm<'w, 's, Cap>)) -> Self {
        self.imm.add(self.id, self.entity, f);
        self
    }

    /// Build descendants of this entity and retrieve return value of inner closure.
    pub fn add_with_return<R>(self, f: impl FnOnce(&mut Imm<'w, 's, Cap>) -> R) -> (Self, R) {
        let value = self.imm.add(self.id, self.entity, f);
        (self, value)
    }

    /// Retrieve system param ctx for immediate mode
    pub fn ctx(&self) -> &ImmCtx<'w, 's, Cap> {
        &self.imm.ctx
    }

    /// Retrieve system param ctx for immediate mode
    pub fn ctx_mut(&mut self) -> &mut ImmCtx<'w, 's, Cap> {
        &mut self.imm.ctx
    }

    /// Helper method to simplify entity retrieval
    pub fn get_entity(&self) -> Result<FilteredEntityRef<'_>, bevy_ecs::query::QueryEntityError> {
        self.ctx().entities.get(self.entity())
    }

    /// Helper method to simplify entity retrieval
    pub fn get_entity_mut(
        &mut self,
    ) -> Result<bevy_ecs::world::FilteredEntityMut<'_>, bevy_ecs::query::QueryEntityError> {
        let entity = self.entity();
        self.ctx_mut().entities.get_mut(entity)
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
        self.will_be_spawned
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
        if self.will_be_spawned {
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
        if self.will_be_spawned {
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
        if self.will_be_spawned || changed {
            let mut entity_commands = self.imm.ctx.commands.entity(self.entity);
            entity_commands.insert(f());
            self
        } else {
            self
        }
    }

    /// Check if current entity contains component in capability requirements
    ///
    /// Useful in implementing capabilities [`ImmCap`]
    pub fn cap_entity_contains<T: Component>(&self) -> bool {
        let Ok(entity) = self.get_entity() else {
            return false;
        };

        entity.contains::<T>()
    }

    /// Retrieve component for entity that was requested by capabilities
    ///
    /// Useful in implementing capabilities [`ImmCap`]
    pub fn cap_get_component<T: Component>(&self) -> Result<Option<&T>, QueryEntityError> {
        let entity = self.get_entity()?;
        Ok(entity.get::<T>())
    }

    /// Retrieve component for entity that was requested by capabilities
    ///
    /// Useful in implementing capabilities [`ImmCap`]
    pub fn cap_get_component_mut<'a, T: Component<Mutability = Mutable>>(
        &'a mut self,
    ) -> Result<Option<bevy_ecs::world::Mut<'a, T>>, QueryEntityError> {
        let entity = self.get_entity_mut()?;
        Ok(entity.into_mut::<T>())
    }

    /// Retrieve resource from capabilities
    ///
    /// Useful in implementing capabilities [`ImmCap`]
    pub fn cap_get_resource<R: Resource>(
        &self,
    ) -> Result<bevy_ecs::world::Ref<'_, R>, ResourceFetchError> {
        self.ctx().resources.get::<R>()
    }

    /// Retrieve mutable resource from capabilities
    ///
    /// Useful in implementing capabilities [`ImmCap`]
    pub fn cap_get_resource_mut<R: Resource>(
        &mut self,
    ) -> Result<bevy_ecs::world::Mut<'_, R>, ResourceFetchError> {
        self.ctx_mut().resources.get_mut::<R>()
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
