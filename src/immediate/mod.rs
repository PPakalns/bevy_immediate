use std::{
    marker::PhantomData,
    ops::{Deref, DerefMut},
};

use bevy_ecs::{
    bundle::Bundle,
    entity::Entity,
    event::Event,
    hierarchy::ChildOf,
    query::{With, Without},
    schedule::IntoScheduleConfigs,
    system::{Commands, EntityCommands, IntoObserverSystem, Query, ResMut},
};

/// System set for systems that power `bevy_immediate` immediate mode functionality
#[derive(bevy_ecs::schedule::SystemSet, PartialEq, Eq, Clone, Debug, Hash)]
pub struct ImmediateSystemSet;

/// Plugin for immediate mode functionality in bevy
pub struct BevyImmediatePlugin<Marker = ()>(PhantomData<Marker>);

impl<Marker> Default for BevyImmediatePlugin<Marker> {
    fn default() -> Self {
        Self(Default::default())
    }
}

impl<Marker> bevy_app::Plugin for BevyImmediatePlugin<Marker>
where
    Marker: Send + Sync + 'static,
{
    fn build(&self, app: &mut bevy_app::App) {
        app.add_systems(
            bevy_app::PostUpdate,
            immediate_mode_tracked_entity_upkeep_system::<Marker>.in_set(ImmediateSystemSet),
        );
        app.insert_resource(ImmediateModeStateResource::<Marker>::default());

        entity_mapping::init::<Marker>(app);
    }
}

mod entity_mapping;

mod ctx;
pub use ctx::ImmCtx;

mod id;
pub use id::{ImmId, ImmIdBuilder, imm_id};

mod capabilities;
pub use capabilities::ImmCapabilities;

/// Trait used to simplify passing around immediate mode
/// extension capabilities and marker type to use in query filter.
///
/// See [`ImmCapabilities`] and [`ImmMarker`], [`ImmAvoid`].
pub trait ImmCM: 'static + Send + Sync {
    type Capabilities: 'static + Send + Sync + ImmCapabilities;
    type Marker: 'static + Send + Sync;
}

impl<Cap> ImmCM for Cap
where
    Cap: 'static + Send + Sync + ImmCapabilities,
{
    type Capabilities = Cap;
    type Marker = ();
}

impl<Cap, Marker> ImmCM for (Cap, Marker)
where
    Cap: 'static + Send + Sync + ImmCapabilities,
    Marker: 'static + Send + Sync,
{
    type Capabilities = Cap;
    type Marker = Marker;
}

/// Immediate mode manager that manages entity [`Self::current`]
///
/// Can be used to build new child entities with [`Self::build`] and similar methods.
pub struct Imm<'w, 's, CM: ImmCM> {
    ctx: ImmCtx<'w, 's, CM>,
    current: Current,
}

#[derive(Clone, Copy)]
struct Current {
    id: ImmId,
    entity: Option<Entity>,
    idx: usize,
}

impl<'w, 's, CM: ImmCM> Imm<'w, 's, CM> {
    /// Build new entity with auto generated id.
    ///
    /// Use [`Self::build_id`] if building entities that may not always exist when parent entity exists.
    ///
    /// Read more [`ImmId`], [`ImmIdBuilder`].
    pub fn child(&mut self) -> ImmEntityBuilder<'_, 'w, 's, CM> {
        self.child_with_manual_id(ImmIdBuilder::Auto)
    }

    /// Build new entity with manually provided id that will be combined with parent entity id to
    /// make truly unique id.
    ///
    /// Read more [`ImmId`], [`ImmIdBuilder`].
    pub fn child_with_id<T: std::hash::Hash>(&mut self, id: T) -> ImmEntityBuilder<'_, 'w, 's, CM> {
        self.child_with_manual_id(ImmIdBuilder::Hierarchy(ImmId::new(id)))
    }

    /// Build new entity with provided id.
    ///
    /// Read more [`ImmId`], [`ImmIdBuilder`].
    pub fn child_with_manual_id(&mut self, id: ImmIdBuilder) -> ImmEntityBuilder<'_, 'w, 's, CM> {
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
                let mut commands = self.ctx.commands.spawn((
                    ImmTrackerComponent::<CM::Marker> {
                        id,
                        iteration: self.ctx.state.iteration,
                        _ph: PhantomData,
                    },
                    // Add marker component that users can use in QueryFilter Without statements
                    ImmMarker::<CM::Marker>::new(),
                ));

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
    fn add<R>(
        &mut self,
        id: ImmId,
        entity: Entity,
        f: impl FnOnce(&mut Imm<'w, 's, CM>) -> R,
    ) -> R {
        self.add_dyn(id, entity, Box::new(f))
    }

    /// Manage entity with provided [`ImmId`] and [`Entity`] attributes with following logic
    fn add_dyn<R>(
        &mut self,
        id: ImmId,
        entity: Entity,
        f: Box<dyn FnOnce(&mut Imm<'w, 's, CM>) -> R + '_>,
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
}

/// Builder to build new entity that is managed by immediate mode logic
///
/// Construction should end with calls to [`Self::add`] or [`Self::add_empty`].
#[must_use]
pub struct ImmEntityBuilder<'r, 'w, 's, CM: ImmCM> {
    sui: &'r mut Imm<'w, 's, CM>,
    id: ImmId,
    entity: Entity,
    currently_creating: bool,
}

impl<'r, 'w, 's, CM: ImmCM> ImmEntityBuilder<'r, 'w, 's, CM> {
    /// Issue [`EntityCommands`] at this moment
    pub fn at_this_moment_apply_commands<F>(self, f: F) -> Self
    where
        F: FnOnce(&mut EntityCommands),
    {
        let mut entity_commands = self.sui.ctx.commands.entity(self.entity);
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
    #[allow(clippy::should_implement_trait)]
    pub fn add<R>(self, f: impl FnOnce(&mut Imm<'w, 's, CM>) -> R) -> ImmReturn<'r, 'w, 's, R, CM> {
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
    pub fn add_empty(self) -> ImmEntity<'r, 'w, 's, CM> {
        ImmEntity {
            ui: self.sui,
            entity: self.entity,
            currently_creating: self.currently_creating,
        }
    }
}

/// Stores return value of closure and builded entity response
pub struct ImmReturn<'r, 'w, 's, Inner, CM: ImmCM> {
    /// Return value of closure that was provided to [`ImmBuilder::add`].
    pub inner: Inner,
    /// Stores information about entity that was built
    pub resp: ImmEntity<'r, 'w, 's, CM>,
}

impl<'r, 'w, 's, Inner, CM: ImmCM> Deref for ImmReturn<'r, 'w, 's, Inner, CM> {
    type Target = ImmEntity<'r, 'w, 's, CM>;

    fn deref(&self) -> &Self::Target {
        &self.resp
    }
}

impl<'r, 'w, 's, Inner, CM: ImmCM> DerefMut for ImmReturn<'r, 'w, 's, Inner, CM> {
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
pub struct ImmEntity<'r, 'w, 's, CM: ImmCM> {
    currently_creating: bool,
    entity: Entity,
    ui: &'r mut Imm<'w, 's, CM>,
}

impl<'r, 'w, 's, CM: ImmCM> ImmEntity<'r, 'w, 's, CM> {
    /// Retrieve [`Entity`] value for this entity
    pub fn entity(&self) -> Entity {
        self.entity
    }

    /// Gain access to [`Commands`]
    pub fn commands(&mut self) -> &mut Commands<'w, 's> {
        self.ui.commands_mut()
    }

    /// Gain access to [`EntityCommands`] for this entity
    pub fn entity_commands<R>(&mut self) -> EntityCommands<'_> {
        let commands = &mut self.ui.ctx.commands;
        commands.entity(self.entity)
    }

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
///
/// Useful to add query filter [`ImmAvoid<Marker>`] or [`bevy_ecs::query::Without<ImmMarker>`] to your queries.
/// In cases where you want to access entities that are constructed by immediate mode.
/// Use other type inside marker than () `[ImmMarker<()>]` in your system and queries.
#[derive(bevy_ecs::component::Component)]
pub struct ImmMarker<Marker = ()> {
    _ph: PhantomData<Marker>,
}

impl<M> ImmMarker<M> {
    fn new() -> Self {
        Self { _ph: PhantomData }
    }
}

/// Type to use in QueryFilter to avoid query collisions
pub type ImmAvoid<Marker = ()> = Without<ImmMarker<Marker>>;

type CMMarker<CM> = <CM as ImmCM>::Marker;
type WithImmMarker<CM> = With<ImmMarker<<CM as ImmCM>::Marker>>;

/// Component that is added to entities that are managed
/// by immediate mode system
#[derive(bevy_ecs::component::Component)]
struct ImmTrackerComponent<Marker> {
    id: ImmId,
    iteration: u32,
    _ph: PhantomData<Marker>,
}

#[derive(bevy_ecs::resource::Resource)]
struct ImmediateModeStateResource<Marker: Send + Sync + 'static> {
    // Current iteration for unused entity removal
    iteration: u32,
    _ph: PhantomData<Marker>,
}

impl<Marker: Send + Sync + 'static> Default for ImmediateModeStateResource<Marker> {
    fn default() -> Self {
        Self {
            iteration: Default::default(),
            _ph: Default::default(),
        }
    }
}

fn immediate_mode_tracked_entity_upkeep_system<Marker: Send + Sync + 'static>(
    query: Query<(Entity, &ImmTrackerComponent<Marker>), With<ImmMarker<Marker>>>,
    mut state: ResMut<ImmediateModeStateResource<Marker>>,
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
