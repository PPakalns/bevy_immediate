use std::{marker::PhantomData, sync::Arc};

use crate::{CapSet, ImmCapAccessRequests, ImmCapAccessRequestsResource};
use bevy_ecs::{
    bundle::Bundle,
    change_detection::DetectChanges,
    component::{Component, Mutable},
    entity::Entity,
    event::EntityEvent,
    hierarchy::ChildOf,
    query::{QueryEntityError, Without},
    resource::Resource,
    system::{Commands, EntityCommands, IntoObserverSystem, Query},
    world::{FilteredEntityRef, Mut, error::ResourceFetchError},
};

/// Plugin for immediate mode functionality in bevy
///
/// Can be initialized multiple times without problems
pub struct BevyImmediatePlugin<Caps = ()>(PhantomData<Caps>);

impl<Caps> BevyImmediatePlugin<Caps> {
    /// Construct plugin
    pub fn new() -> Self {
        Self(PhantomData)
    }
}

impl<Caps> Default for BevyImmediatePlugin<Caps> {
    fn default() -> Self {
        Self::new()
    }
}

impl<Caps> bevy_app::Plugin for BevyImmediatePlugin<Caps>
where
    Caps: CapSet,
{
    fn build(&self, app: &mut bevy_app::App) {
        if app.is_plugin_added::<Self>() {
            return;
        }

        entity_mapping::init::<Caps>(app);
        upkeep::init::<Caps>(app);

        let mut capabilities = ImmCapAccessRequests::<Caps>::default();
        Caps::initialize(app, &mut capabilities);
        app.insert_resource(ImmCapAccessRequestsResource::new(capabilities));

        #[cfg(feature = "hotpatching")]
        hotpatching::init(app);
    }

    fn is_unique(&self) -> bool {
        // Users do not need to track
        // if this plugin has been initialized in only one place
        false
    }
}

/// Adds support for bevy inbuilt hotpatching mechanism
#[cfg(feature = "hotpatching")]
pub mod hotpatching;

mod system_set;
pub use system_set::ImmediateSystemSet;

mod ctx;
pub use ctx::ImmCtx;

mod id;
use crate::utils::ImmTypeMap;
pub use id::{ImmId, ImmIdBuilder, imm_id};

mod entity_mapping;
mod upkeep;

/// Helper type to more easily write queries
pub type ImmQuery<'w, 's, Caps, D, F = ()> = Query<'w, 's, D, (Without<ImmMarker<Caps>>, F)>;

/// Immediate mode in a state where child components can be added
///
/// We can not be sure that:
/// * there is parent entity,
/// * if it even exists or is currently being spawned,
/// * if it even is managed by current immediate mode logic
///
/// Use [`Self::reinterpret_as_entity`] if you are sure about
/// what kind of data can be accessed about parent entity.
pub struct Imm<'w, 's, Caps: CapSet> {
    ctx: ImmCtx<'w, 's, Caps>,
    current: Current,
}

#[derive(Clone, Copy)]
struct Current {
    id: ImmId,
    entity: Option<CurrentEntity>,
    idx: usize,
}

#[derive(Clone, Copy)]
struct CurrentEntity {
    entity: Entity,
    will_be_spawned: bool,
}

impl<'w, 's, Caps: CapSet> Imm<'w, 's, Caps> {
    /// Build new entity with auto generated id.
    ///
    /// Use [`Self::ch_id`] if building entities that may not always exist when parent entity exists.
    ///
    /// Read more [`ImmId`], [`ImmIdBuilder`].
    pub fn ch(&mut self) -> ImmEntity<'_, 'w, 's, Caps> {
        self.ch_with_manual_id(ImmIdBuilder::Auto)
    }

    /// Build new entity with manually provided id that will be combined with parent entity id to
    /// make truly unique id.
    ///
    /// Ids must be unique between sibling entities.
    ///
    /// Read more [`ImmId`], [`ImmIdBuilder`].
    pub fn ch_id<T: std::hash::Hash>(&mut self, id: T) -> ImmEntity<'_, 'w, 's, Caps> {
        self.ch_with_manual_id(ImmIdBuilder::Hierarchy(ImmId::new(id)))
    }

    /// Build new entity with provided id.
    ///
    /// Read more [`ImmId`], [`ImmIdBuilder`].
    pub fn ch_with_manual_id(&mut self, id: ImmIdBuilder) -> ImmEntity<'_, 'w, 's, Caps> {
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

                if qentity.child_of.map(|ch| ch.parent()) != self.current.entity.map(|e| e.entity) {
                    // Parent changed
                    let mut entity_commands = self.ctx.commands.entity(entity);
                    match self.current.entity {
                        Some(entity) => {
                            entity_commands.insert(ChildOf(entity.entity));
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
                ImmMarker::<Caps> {
                    id,
                    iteration: self.ctx.state.iteration,
                    _ph: PhantomData,
                },
            ));

            if let Some(entity) = self.current.entity {
                commands.insert(ChildOf(entity.entity));
            }
            will_be_spawned = true;
            commands.id()
        };

        // TODO: Avoid clone
        let access_requests: Arc<_> = self.ctx.access_requests.capabilities.clone();

        let mut entity = ImmEntity {
            imm: self,
            e: EntityParams {
                id,
                entity,
                will_be_spawned,
            },
            tmp_store: ImmTypeMap::new(),
        };

        for on_children in access_requests.on_children.iter() {
            (on_children)(&mut entity);
        }

        entity
    }

    /// Useful to spawn new unrooted entity trees
    ///
    /// In context of UI, useful for tooltips, popups.
    pub fn unrooted<T: std::hash::Hash>(&mut self, id: T, f: impl FnOnce(&mut Imm<'w, 's, Caps>)) {
        let id = ImmIdBuilder::Hierarchy(ImmId::new(id)).resolve(self);

        // Create new unrooted context
        let mut current = Current {
            id,
            entity: None,
            idx: 0,
        };

        std::mem::swap(&mut self.current, &mut current);
        f(self);

        self.current = current;
    }

    /// Entity that is currently being managed
    ///
    /// If building root of entity tree, this value may be [`None`]
    #[inline]
    pub fn current_entity(&self) -> Option<Entity> {
        self.current.entity.map(|e| e.entity)
    }

    /// Immediate mode unique id
    #[inline]
    pub fn current_imm_id(&self) -> ImmId {
        self.current.id
    }

    /// Retrieve access to commands
    #[inline]
    pub fn commands_mut(&mut self) -> &mut Commands<'w, 's> {
        &mut self.ctx.commands
    }

    /// Access underlaying context
    ///
    /// Useful for implementing additional [crate::ImmCapability]
    #[inline]
    pub fn ctx_mut(&mut self) -> &mut ImmCtx<'w, 's, Caps> {
        &mut self.ctx
    }

    /// Retrieve [`ImmCtx`] from which immediate mode entity tree was built
    pub fn deconstruct(self) -> ImmCtx<'w, 's, Caps> {
        self.ctx
    }

    /// Manage entity with provided [`ImmId`] and [`Entity`] attributes with provided closure
    fn add_child_entity<R>(
        &mut self,
        params: EntityParams,
        f: impl FnOnce(&mut Imm<'w, 's, Caps>) -> R,
    ) -> R {
        self.add_child_entity_dyn(params, Box::new(f))
    }

    /// Manage entity with provided [`ImmId`] and [`Entity`] attributes with provided closure
    #[allow(clippy::type_complexity)]
    fn add_child_entity_dyn<R>(
        &mut self,
        EntityParams {
            id,
            entity,
            will_be_spawned,
        }: EntityParams,
        f: Box<dyn FnOnce(&mut Imm<'w, 's, Caps>) -> R + '_>,
    ) -> R {
        let stored_current = self.current;

        self.current = Current {
            id,
            entity: Some(CurrentEntity {
                entity,
                will_be_spawned,
            }),
            idx: 0,
        };

        let resp = f(self);

        self.current = stored_current;

        resp
    }

    /// Manage current context as entity
    ///
    /// It returns `None` only in cases where Imm is not rooted to existing entity.
    ///
    /// Useful in rare cases where access to parent entity is needed.
    /// If parent entity is not managed by Immediate mode, may result in panic
    /// when capabilities try to access data from queries that query only immediate mode entities.
    /// Capabilities will have access to empty temporary store.
    pub fn reinterpret_as_entity(&mut self) -> Option<ImmEntity<'_, 'w, 's, Caps>> {
        if let Some(current_entity) = self.current.entity {
            let e = EntityParams {
                id: self.current.id,
                entity: current_entity.entity,
                will_be_spawned: current_entity.will_be_spawned,
            };
            Some(ImmEntity {
                imm: self,
                e,
                tmp_store: ImmTypeMap::new(),
            })
        } else {
            None
        }
    }

    /// Helper function to correctly detect changes that could have happened even in this system
    pub fn has_changed<T>(&self, value: &Mut<'_, T>) -> bool {
        self.change_detector().has_changed(value)
    }

    /// Helper object for state change detection
    pub fn change_detector(&self) -> ChangeDetector {
        ChangeDetector {
            last_run: self.ctx.system_change_tick.last_run(),
        }
    }
}

/// Entity during construction in immediate mode approach
///
/// Can be used to issue commands and check such conditions as `.clicked()`.
pub struct ImmEntity<'r, 'w, 's, Caps: CapSet> {
    imm: &'r mut Imm<'w, 's, Caps>,
    /// Entity managed by this instance
    e: EntityParams,
    tmp_store: ImmTypeMap,
}

#[derive(Clone, Copy)]
struct EntityParams {
    id: ImmId,
    entity: Entity,
    will_be_spawned: bool,
}

impl<'r, 'w, 's, Caps: CapSet> ImmEntity<'r, 'w, 's, Caps> {
    /// Build descendants of this entity
    ///
    /// If closure return value is needed, use `[Self::add_with_return]``
    #[allow(clippy::should_implement_trait)]
    pub fn add(self, f: impl FnOnce(&mut Imm<'w, 's, Caps>)) -> Self {
        self.imm.add_child_entity(self.e, f);
        self
    }

    /// Build descendants of this entity and retrieve return value of inner closure.
    pub fn add_with_return<R>(self, f: impl FnOnce(&mut Imm<'w, 's, Caps>) -> R) -> (Self, R) {
        let value = self.imm.add_child_entity(self.e, f);
        (self, value)
    }

    /// Spawn new entity trees (unrooted)
    ///
    /// In UI useful for Tooltips, Popups, etc.
    pub fn unrooted<T: std::hash::Hash>(
        self,
        id: T,
        f: impl FnOnce(&mut Imm<'w, 's, Caps>),
    ) -> Self {
        self.add(|ui| {
            ui.unrooted(id, f);
        })
    }

    /// Retrieve system param ctx for immediate mode
    pub fn ctx(&self) -> &ImmCtx<'w, 's, Caps> {
        &self.imm.ctx
    }

    /// Retrieve system param ctx for immediate mode
    pub fn ctx_mut(&mut self) -> &mut ImmCtx<'w, 's, Caps> {
        &mut self.imm.ctx
    }

    /// Helper method to simplify entity retrieval
    pub fn cap_get_entity(
        &self,
    ) -> Result<FilteredEntityRef<'_, 's>, bevy_ecs::query::QueryEntityError> {
        self.ctx().cap_entities.get(self.entity())
    }

    /// Helper method to simplify entity retrieval
    pub fn cap_get_entity_mut(
        &mut self,
    ) -> Result<bevy_ecs::world::FilteredEntityMut<'_, 's>, bevy_ecs::query::QueryEntityError> {
        let entity = self.entity();
        self.ctx_mut().cap_entities.get_mut(entity)
    }

    /// Retrieve [`Entity`] value for this entity
    pub fn entity(&self) -> Entity {
        self.e.entity
    }

    /// Immediate mode unique id for this entity
    #[inline]
    pub fn imm_id(&self) -> ImmId {
        self.e.id
    }

    /// Gain access to [`Commands`]
    pub fn commands(&mut self) -> &mut Commands<'w, 's> {
        &mut self.imm.ctx.commands
    }

    /// Gain access to [`EntityCommands`] for this entity
    pub fn entity_commands(&mut self) -> EntityCommands<'_> {
        self.imm.ctx.commands.entity(self.e.entity)
    }

    /// Entity will be spawned when [`Commands`] will be processed.
    pub fn will_be_spawned(&self) -> bool {
        self.e.will_be_spawned
    }

    /// Issue [`EntityCommands`] at this moment
    pub fn at_this_moment_apply_commands<F>(mut self, f: F) -> Self
    where
        F: FnOnce(&mut EntityCommands<'_>),
    {
        let mut entity_commands = self.entity_commands();
        f(&mut entity_commands);
        self
    }

    /// Issue [`EntityCommands`] at this moment if condition is met
    pub fn at_this_moment_apply_commands_if<F>(self, f: F, condition: impl FnOnce() -> bool) -> Self
    where
        F: FnOnce(&mut EntityCommands<'_>),
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
        F: FnOnce(&mut EntityCommands<'_>),
    {
        if self.e.will_be_spawned {
            self.at_this_moment_apply_commands(f)
        } else {
            self
        }
    }

    /// Issue [`EntityCommands`] if condition is met.
    /// (issued only when entity is created).
    pub fn on_spawn_apply_commands_if<F>(self, f: F, condition: impl FnOnce() -> bool) -> Self
    where
        F: FnOnce(&mut EntityCommands<'_>),
    {
        if self.e.will_be_spawned {
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
    pub fn on_spawn_observe<E: EntityEvent, B: Bundle, M>(
        self,
        observer: impl IntoObserverSystem<E, B, M>,
    ) -> Self {
        self.on_spawn_apply_commands(|commands| {
            commands.observe(observer);
        })
    }

    /// If entity spawned or changed value is `true`, insert [`Bundle`] into entity
    pub fn on_change_insert<F, B>(mut self, changed: bool, f: F) -> Self
    where
        F: FnOnce() -> B,
        B: Bundle,
    {
        if self.e.will_be_spawned || changed {
            self.entity_commands().insert(f());
            self
        } else {
            self
        }
    }

    /// Check if current entity contains component in capability requirements
    ///
    /// Useful in implementing capabilities [`crate::ImmCapabiility`]
    pub fn cap_entity_contains<T: Component>(&self) -> bool {
        let Ok(entity) = self.cap_get_entity() else {
            return false;
        };

        entity.contains::<T>()
    }

    /// Retrieve component for entity that was requested by capabilities
    ///
    /// Useful in implementing capabilities [`crate::ImmCapabiility`]
    pub fn cap_get_component<T: Component>(&self) -> Result<Option<&T>, QueryEntityError> {
        let entity = self.cap_get_entity()?;
        Ok(entity.get::<T>())
    }

    /// Retrieve component for entity that was requested by capabilities
    ///
    /// Useful in implementing capabilities [`crate::ImmCapabiility`]
    pub fn cap_get_component_mut<'a, T: Component<Mutability = Mutable>>(
        &'a mut self,
    ) -> Result<Option<bevy_ecs::world::Mut<'a, T>>, QueryEntityError> {
        let entity = self.cap_get_entity_mut()?;
        Ok(entity.into_mut::<T>())
    }

    /// Retrieve resource from capabilities
    ///
    /// Useful in implementing capabilities [`crate::ImmCapabiility`]
    pub fn cap_get_resource<R: Resource>(
        &self,
    ) -> Result<bevy_ecs::world::Ref<'_, R>, ResourceFetchError> {
        self.ctx().cap_resources.get::<R>()
    }

    /// Retrieve mutable resource from capabilities
    ///
    /// Useful in implementing capabilities [`crate::ImmCapabiility`]
    pub fn cap_get_resource_mut<R: Resource>(
        &mut self,
    ) -> Result<bevy_ecs::world::Mut<'_, R>, ResourceFetchError> {
        self.ctx_mut().cap_resources.get_mut::<R>()
    }

    /// Helper function to correctly detect changes that could have happened even in this system
    pub fn changed_for<T>(&self, value: &Mut<'_, T>) -> bool {
        value.is_changed() || value.last_changed() == self.ctx().system_change_tick.last_run()
    }

    /// Helper function to correctly detect changes that could have happened even in this system
    pub fn has_changed<T>(&self, value: &Mut<'_, T>) -> bool {
        self.imm.has_changed(value)
    }

    /// Access data store for current entity.
    ///
    /// It exists only during entity construction
    pub fn cap_entity_tmp_store(&self) -> &ImmTypeMap {
        &self.tmp_store
    }

    /// Access data store for current entity.
    ///
    /// It exists only during entity construction
    pub fn cap_entity_tmp_store_mut(&mut self) -> &mut ImmTypeMap {
        &mut self.tmp_store
    }

    /// Get [`Entity`] for parent entity of this entity
    pub fn parent_entity(&self) -> Option<Entity> {
        self.imm.current_entity()
    }
}

/// Component that is added to entities that are managed by immediate mode system
///
/// Useful to add query filter [`WithoutImm<()>`] or [`bevy_ecs::query::Without<ImmMarker<()>>`]
/// to your queries. Replace `()` with `Cap` that you use.
#[derive(bevy_ecs::component::Component)]
pub struct ImmMarker<Caps> {
    id: ImmId,
    iteration: u32,
    _ph: PhantomData<Caps>,
}

/// Type to use in QueryFilter to avoid query collisions
pub type WithoutImm<Caps = ()> = Without<ImmMarker<Caps>>;

/// Helper structure for immediate mode compatbile state change detection
pub struct ChangeDetector {
    last_run: bevy_ecs::component::Tick,
}

impl ChangeDetector {
    /// Has state changed since and including last time system was executed
    pub fn has_changed<T>(&self, value: &Mut<'_, T>) -> bool {
        value.is_changed() || value.last_changed() == self.last_run
    }
}
