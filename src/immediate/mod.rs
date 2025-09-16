use std::marker::PhantomData;

use bevy_ecs::{
    bundle::Bundle,
    entity::Entity,
    event::Event,
    hierarchy::ChildOf,
    schedule::IntoScheduleConfigs,
    system::{Commands, EntityCommands, IntoObserverSystem, Query, Res, ResMut},
};
use bevy_ui::UiSystem;
use id_mapping::SimpleUiIdMapping;

#[cfg(feature = "picking")]
use crate::ui::picking;

pub struct BevyImmediatePlugin;

impl bevy_app::Plugin for BevyImmediatePlugin {
    fn build(&self, app: &mut bevy_app::App) {
        app.add_systems(
            bevy_app::PostUpdate,
            upkeep_ui_system.before(UiSystem::Prepare),
        );

        app.insert_resource(SimpleUiState::default());

        id_mapping::init(app);
    }
}

mod id_mapping;

#[derive(bevy_ecs::system::SystemParam)]
pub struct SimpleUiCtx<'w, 's> {
    query: Query<'w, 's, SUiNodeQuery>,

    #[cfg(feature = "picking")]
    track_clicked_query: Query<'w, 's, &'static picking::TrackClicked>,

    state: Res<'w, SimpleUiState>,
    id_mapping: Res<'w, SimpleUiIdMapping>,
    commands: Commands<'w, 's>,

    _ph: PhantomData<&'s ()>,
}
impl<'w, 's> SimpleUiCtx<'w, 's> {
    pub fn init_ui<T: std::hash::Hash>(self, root_id: T) -> Sui<'w, 's> {
        Sui {
            ctx: self,
            current: Current {
                id: SuiId::new(root_id),
                entity: None,
                idx: 0,
            },
        }
    }
}

#[derive(bevy_ecs::query::QueryData)]
#[query_data(mutable)]
struct SUiNodeQuery {
    sui_marker: &'static mut SuiMarker,
    child_of: Option<&'static ChildOf>,
}

#[derive(bevy_ecs::resource::Resource, Default)]
struct SimpleUiState {
    iteration: u32,
}

#[derive(bevy_ecs::component::Component)]
pub struct SuiMarker {
    id: SuiId,
    iteration: u32,
}

pub enum SuiIdBuilder {
    Auto,
    Hierarchy(SuiId),
    Unique(SuiId),
}
impl SuiIdBuilder {
    fn resolve(self, sui: &Sui<'_, '_>) -> SuiId {
        match self {
            SuiIdBuilder::Auto => {
                let id = sui.current.id.with(sui.current.idx);
                id
            }
            SuiIdBuilder::Hierarchy(sui_id) => sui.current.id.with(sui_id),
            SuiIdBuilder::Unique(sui_id) => sui_id,
        }
    }
}

#[derive(Hash, Clone, Copy, PartialEq, Eq)]
pub struct SuiId {
    id: u64,
}
impl SuiId {
    pub fn new(source: impl std::hash::Hash) -> Self {
        Self {
            id: ahash::RandomState::with_seeds(1, 2, 3, 4).hash_one(source),
        }
    }

    pub fn with(self, child: impl std::hash::Hash) -> Self {
        use std::hash::{BuildHasher as _, Hasher as _};
        let mut hasher = ahash::RandomState::with_seeds(1, 2, 3, 4).build_hasher();
        hasher.write_u64(self.id);
        child.hash(&mut hasher);
        Self {
            id: hasher.finish(),
        }
    }
}

pub struct Sui<'w, 's> {
    ctx: SimpleUiCtx<'w, 's>,
    current: Current,
}

#[derive(Clone, Copy)]
struct Current {
    id: SuiId,
    entity: Option<Entity>,
    idx: usize,
}

impl<'w, 's> Sui<'w, 's> {
    pub fn build(&mut self) -> SuiBuilder<'_, 'w, 's> {
        self.build_id(SuiIdBuilder::Auto)
    }

    pub fn build_id(&mut self, id: SuiIdBuilder) -> SuiBuilder<'_, 'w, 's> {
        let id = id.resolve(self);
        self.current.idx += 1;

        let mut currently_creating = false;

        let entity = match self.ctx.id_mapping.id_to_entity.get(&id).copied() {
            Some(entity) => {
                if let Ok(mut qentity) = self.ctx.query.get_mut(entity) {
                    qentity.sui_marker.iteration = self.ctx.state.iteration;
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
                let mut commands = self.ctx.commands.spawn(SuiMarker {
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

        SuiBuilder {
            sui: self,
            id,
            currently_creating,
            entity,
        }
    }

    fn add<R>(&mut self, id: SuiId, entity: Entity, f: impl FnOnce(&mut Sui<'w, 's>) -> R) -> R {
        self.add_dyn(id, entity, Box::new(f))
    }

    fn add_dyn<R>(
        &mut self,
        id: SuiId,
        entity: Entity,
        f: Box<dyn FnOnce(&mut Sui<'w, 's>) -> R + '_>,
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
}

#[must_use]
pub struct SuiBuilder<'r, 'w, 's> {
    sui: &'r mut Sui<'w, 's>,
    id: SuiId,
    entity: Entity,
    currently_creating: bool,
}
impl<'r, 'w, 's> SuiBuilder<'r, 'w, 's> {
    pub fn at_this_moment_insert_commands<F>(self, f: F) -> Self
    where
        F: FnOnce(&mut EntityCommands),
    {
        let mut entity_commands = self.sui.ctx.commands.entity(self.entity);
        f(&mut entity_commands);
        self
    }

    pub fn on_insert_commands<F>(self, f: F) -> Self
    where
        F: FnOnce(&mut EntityCommands),
    {
        if self.currently_creating {
            self.at_this_moment_insert_commands(f)
        } else {
            self
        }
    }

    pub fn on_insert_add_bundle<F, B>(self, f: F) -> Self
    where
        F: FnOnce() -> B,
        B: Bundle,
    {
        self.on_insert_commands(|commands| {
            commands.insert(f());
        })
    }

    pub fn on_insert_add_observer<E: Event, B: Bundle, M>(
        self,
        observer: impl IntoObserverSystem<E, B, M>,
    ) -> Self {
        self.on_insert_commands(|commands| {
            commands.observe(observer);
        })
    }

    pub fn on_change_add_bundle<F, B>(self, changed: bool, f: F) -> Self
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

    pub fn add<R>(self, f: impl FnOnce(&mut Sui<'w, 's>) -> R) -> SuiReturn<'r, 'w, 's, R> {
        let resp = self.sui.add(self.id, self.entity, f);

        SuiReturn {
            value: resp,
            ui: self.sui,
            entity: self.entity,
            currently_creating: self.currently_creating,
        }
    }

    pub fn add_empty(self) -> SuiReturn<'r, 'w, 's, ()> {
        SuiReturn {
            value: (),
            ui: self.sui,
            entity: self.entity,
            currently_creating: self.currently_creating,
        }
    }
}

pub fn sid<T: std::hash::Hash>(val: T) -> SuiIdBuilder {
    SuiIdBuilder::Hierarchy(SuiId::new(val))
}

pub struct SuiReturn<'r, 'w, 's, Inner> {
    pub value: Inner,

    currently_creating: bool,
    entity: Entity,
    ui: &'r mut Sui<'w, 's>,
}

impl<'r, 'w, 's, Inner> SuiReturn<'r, 'w, 's, Inner> {
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

fn upkeep_ui_system(
    query: Query<(Entity, &SuiMarker)>,
    mut state: ResMut<SimpleUiState>,
    mut commands: Commands,
) {
    for (entity, marker) in query {
        if marker.iteration == state.iteration {
            continue;
        }
        commands.entity(entity).despawn();
    }

    state.iteration += 1;
}
