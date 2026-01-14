#![warn(missing_docs)]
#![doc = include_str!("../README.md")]

//! Logic to provide immediate mode api
//! to attach entity tree to already existing entity
use std::{any::TypeId, marker::PhantomData};

use bevy_ecs::{
    component::Component,
    entity::Entity,
    lifecycle,
    observer::On,
    query::With,
    system::{Commands, In, Query, StaticSystemParam, SystemParam},
};

use bevy_immediate_core::{BevyImmediatePlugin, CapSet, Imm, ImmCtx, ImmId};

/// Implement trait to be able to attach immediate tree in arbitrary place
///
/// Remember to add [`BevyImmediateAttachPlugin`]
pub trait ImmediateAttach<Caps: CapSet>: Component {
    /// Use 'static lifetimes where lifetimes are needed.
    type Params: SystemParam;

    /// Executes construction of immediate mode tree
    ///
    /// Function will be called during update or some time after `Self`
    /// has been added to entity.
    fn construct<'w, 's>(
        imm: &mut Imm<'w, 's, Caps>,
        params: &mut <Self::Params as SystemParam>::Item<'w, 's>,
    );
}

/// Add plugin to execute immediate tree so that it is attached to an existing entity
///
/// Immediate tree will be refreshed during each bevy_app::Update schedule or
/// after `RootComponent` is added to entity to avoid 1 frame delay.
///
/// For `RootComponent` trait [`ImmediateAttachRoot`] must be implemented.
pub struct BevyImmediateAttachPlugin<Caps: CapSet, RootComponent: ImmediateAttach<Caps>> {
    _ph: PhantomData<(Caps, RootComponent)>,
}

impl<Caps: CapSet, RootComponent: ImmediateAttach<Caps>>
    BevyImmediateAttachPlugin<Caps, RootComponent>
{
    /// Construct plugin
    pub fn new() -> Self {
        Self { _ph: PhantomData }
    }
}

impl<Caps: CapSet, RootComponent: ImmediateAttach<Caps>> Default
    for BevyImmediateAttachPlugin<Caps, RootComponent>
{
    fn default() -> Self {
        Self::new()
    }
}

impl<Caps: CapSet, RootComponent: ImmediateAttach<Caps>> bevy_app::Plugin
    for BevyImmediateAttachPlugin<Caps, RootComponent>
{
    fn build(&self, app: &mut bevy_app::App) {
        if !app.is_plugin_added::<BevyImmediatePlugin<Caps>>() {
            app.add_plugins(BevyImmediatePlugin::<Caps>::new());
        }

        app.add_systems(
            bevy_app::Update,
            run_system_each_frame::<Caps, RootComponent>,
        );
        app.add_observer(on_insert::<Caps, RootComponent>);
    }
}

/// Retrieve marker type id. In rare cases multiple immediate mode trees could be built from the
/// same root node. We need to provide unique id for each of them.
fn const_type_id<Caps: 'static, RootComponent: 'static>() -> ImmId {
    let root_type_id = TypeId::of::<RootComponent>();
    let cap_type_id = TypeId::of::<Caps>();
    ImmId::new((root_type_id, cap_type_id))
}

#[derive(SystemParam)]
struct AttachSysParam<'w, 's, Caps: CapSet, RootComponent: ImmediateAttach<Caps>> {
    query: Query<'w, 's, Entity, With<RootComponent>>,
    ctx: ImmCtx<'w, 's, Caps>,
    params: StaticSystemParam<'w, 's, <RootComponent as ImmediateAttach<Caps>>::Params>,
}

fn run_system_each_frame<Caps: CapSet, RootComponent: ImmediateAttach<Caps>>(
    sys: AttachSysParam<Caps, RootComponent>,
) {
    let AttachSysParam {
        query,
        mut ctx,
        params,
    } = sys;

    let id = const_type_id::<Caps, RootComponent>();
    let mut params = params.into_inner();

    for entity in query.iter() {
        let mut imm = ctx.build_immediate_from(id.with(entity), entity);
        RootComponent::construct(&mut imm, &mut params);
        ctx = imm.deconstruct();
    }
}

#[allow(clippy::type_complexity)]
fn run_system_on_insert<Caps: CapSet, RootComponent: ImmediateAttach<Caps>>(
    In(entity): In<Entity>,
    sys: AttachSysParam<Caps, RootComponent>,
) {
    let AttachSysParam { query, ctx, params } = sys;

    let id = const_type_id::<Caps, RootComponent>();
    let mut params = params.into_inner();

    if !query.contains(entity) {
        // Root component doesn't exist anymore for this entity
        return;
    };

    let mut imm = ctx.build_immediate_from(id.with(entity), entity);
    RootComponent::construct(&mut imm, &mut params);
}

fn on_insert<Caps: CapSet, RootComponent: ImmediateAttach<Caps>>(
    trigger: On<lifecycle::Add, RootComponent>,
    mut commands: Commands,
) {
    let entity = trigger.event().entity;
    log::trace!(
        "On insert system scheduled to build immediate tree for {}",
        entity
    );
    commands.run_system_cached_with(run_system_on_insert::<Caps, RootComponent>, entity);
}
