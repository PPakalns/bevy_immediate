use std::{any::TypeId, marker::PhantomData};

use bevy_ecs::{
    component::Component,
    entity::Entity,
    observer::Trigger,
    query::With,
    system::{Commands, In, Query, StaticSystemParam, SystemParam},
    world::OnAdd,
};

use crate::{Imm, ImmCap, ImmCtx, ImmId};

/// Implement trait to be able to attach immediate tree in arbitrary place
///
/// Remember to add [`BevyImmediateAttachPlugin`]
pub trait ImmediateAttachRoot<Cap: ImmCap>: Component {
    /// Use 'static lifetimes where lifetimes are needed.
    type Params: SystemParam;

    /// Executes construction of immediate mode tree
    ///
    /// Function will be cared during update or some time after `Self`
    /// has been added to entity.
    fn execute(
        imm: &mut Imm<'_, '_, Cap>,
        params: &mut <Self::Params as SystemParam>::Item<'_, '_>,
    );
}

/// Add plugin to execute immediate tree so that it is attached to an existing entity
///
/// Immediate tree will be refreshed during each bevy_app::Update schedule or
/// after `RootComponent` is added to entity to avoid 1 frame delay.
///
/// For `RootComponent` trait [`ImmediateAttachRoot`] must be implemented.
pub struct BevyImmediateAttachPlugin<Cap: ImmCap, RootComponent: ImmediateAttachRoot<Cap>> {
    _ph: PhantomData<(Cap, RootComponent)>,
}

impl<Cap: ImmCap, RootComponent: ImmediateAttachRoot<Cap>>
    BevyImmediateAttachPlugin<Cap, RootComponent>
{
    /// Construct plugin
    pub fn new() -> Self {
        Self { _ph: PhantomData }
    }
}

impl<Cap: ImmCap, RootComponent: ImmediateAttachRoot<Cap>> Default
    for BevyImmediateAttachPlugin<Cap, RootComponent>
{
    fn default() -> Self {
        Self::new()
    }
}

impl<Cap: ImmCap, RootComponent: ImmediateAttachRoot<Cap>> bevy_app::Plugin
    for BevyImmediateAttachPlugin<Cap, RootComponent>
{
    fn build(&self, app: &mut bevy_app::App) {
        app.add_systems(
            bevy_app::Update,
            run_system_each_frame::<Cap, RootComponent>,
        );
        app.add_observer(on_insert::<Cap, RootComponent>);
    }
}

/// Retrieve marker type id. In rare cases multiple immediate mode trees could be built from the
/// same root node. We need to provide unique id for each of them.
fn const_type_id<Cap: 'static, RootComponent: 'static>() -> ImmId {
    let root_type_id = TypeId::of::<RootComponent>();
    let cap_type_id = TypeId::of::<Cap>();
    ImmId::new((root_type_id, cap_type_id))
}

fn run_system_each_frame<Cap: ImmCap, RootComponent: ImmediateAttachRoot<Cap>>(
    query: Query<Entity, With<RootComponent>>,
    mut ctx: ImmCtx<Cap>,
    params: StaticSystemParam<RootComponent::Params>,
) {
    let id = const_type_id::<Cap, RootComponent>();
    let mut params = params.into_inner();

    for entity in query.iter() {
        let mut imm = ctx.build_immediate_from(id.with(entity), entity);
        RootComponent::execute(&mut imm, &mut params);
        ctx = imm.deconstruct();
    }
}

#[allow(clippy::type_complexity)]
fn run_system_on_insert<Cap: ImmCap, RootComponent: ImmediateAttachRoot<Cap>>(
    In(entity): In<Entity>,
    query: Query<Option<&RootComponentBuilt<(Cap, RootComponent)>>, With<RootComponent>>,
    ctx: ImmCtx<Cap>,
    params: StaticSystemParam<RootComponent::Params>,
) {
    let id = const_type_id::<Cap, RootComponent>();
    let mut params = params.into_inner();

    let Ok(built) = query.get(entity) else {
        // Root component doesn't exist anymore for this entity
        return;
    };

    if built.is_some() {
        // UI already initialized
        return;
    }

    let mut imm = ctx.build_immediate_from(id.with(entity), entity);
    RootComponent::execute(&mut imm, &mut params);
}

fn on_insert<Cap: ImmCap, RootComponent: ImmediateAttachRoot<Cap>>(
    trigger: Trigger<OnAdd, RootComponent>,
    query: Query<(), With<RootComponentBuilt<(Cap, RootComponent)>>>,
    mut commands: Commands,
) {
    let entity = trigger.target();
    if !query.contains(entity) {
        log::trace!(
            "On insert system scheduled to build immediate tree for {}",
            entity
        );
        commands.run_system_cached_with(run_system_on_insert::<Cap, RootComponent>, entity);
    }
}

#[derive(bevy_ecs::component::Component)]
#[component(storage = "SparseSet")]
struct RootComponentBuilt<T> {
    _ph: PhantomData<T>,
}
