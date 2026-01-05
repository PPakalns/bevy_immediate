use std::{marker::PhantomData, ops::DerefMut};

use bevy_ecs::{
    entity::Entity,
    hierarchy::Children,
    resource::Resource,
    schedule::IntoScheduleConfigs,
    system::{Query, ResMut},
};
use bevy_platform::collections::HashMap;

use bevy_immediate_core::{
    CapSet, ImmCapAccessRequests, ImmCapability, ImmEntity, ImmediateSystemSet,
};

/// Base capability for UI that sets up correct order of immediate system execution
pub struct CapabilityUiLayoutOrder;

impl ImmCapability for CapabilityUiLayoutOrder {
    fn build<Cap: CapSet>(app: &mut bevy_app::App, cap_req: &mut ImmCapAccessRequests<Cap>) {
        app.insert_resource(UiOrderTracker::<Cap>::default());
        app.add_systems(
            bevy_app::PostUpdate,
            immediate_mode_ui_children_order_system::<Cap>
                .in_set(ImmediateSystemSet::<Cap>::default()),
        );

        cap_req.request_resource_write::<UiOrderTracker<Cap>>(app.world_mut());
        cap_req.add_on_children_event_listener(Box::new(add_children_to_ui_order_tracker::<Cap>));
    }
}

fn add_children_to_ui_order_tracker<Cap: CapSet>(entity: &mut ImmEntity<Cap>) {
    let Some(parent) = entity.parent_entity() else {
        return;
    };
    let children = entity.entity();
    let mut tracker = entity
        .cap_get_resource_mut::<UiOrderTracker<Cap>>()
        .expect("Capability requested this resource");

    let tracker = tracker.deref_mut();
    let value = tracker.parent_entities.entry(parent).or_default();
    tracker.children_order.insert(children, *value);
    *value += 1;
}

fn immediate_mode_ui_children_order_system<Cap: CapSet>(
    mut tracker: ResMut<UiOrderTracker<Cap>>,

    // We can not add `With<ImmMarker<Cap>>` because
    // in case of widgets parent entity could be
    // entity that was not created in immediate mode
    mut query: Query<&mut Children>,
) {
    let tracker = tracker.deref_mut();

    let mut for_sort = Vec::new();
    let mut idx_to_location = Vec::new();
    let mut location_to_idx = Vec::new();
    let mut next_locations = Vec::new();

    for (&parent_entity, &child_count) in tracker.parent_entities.iter() {
        if child_count <= 1 {
            // Nothing to sort
            continue;
        }

        let Ok(mut children) = query.get_mut(parent_entity) else {
            // Looks like entity was removed. Could happen due to parent UI triggering remove of
            // all children
            continue;
        };

        // Collect all childrens for sorting
        for_sort.reserve(children.len());
        next_locations.reserve(children.len());

        // We need to extract subset of entities
        // that are managed by immediate mode and which needs sorting
        let mut count_order = 0;
        let mut matches = true;
        for (child_idx, child) in children.iter().enumerate() {
            let Some(&child_order) = tracker.children_order.get(child) else {
                continue;
            };
            if child_order != count_order {
                matches = false;
            }

            count_order += 1;
            for_sort.push((child_order, child_idx));
            next_locations.push(child_idx);
        }

        if !matches {
            // Needs sorting

            idx_to_location.reserve(children.len());
            location_to_idx.reserve(children.len());
            for child_idx in 0..children.len() {
                idx_to_location.push(child_idx);
                location_to_idx.push(child_idx);
            }

            let children = children.deref_mut();
            for_sort.sort_unstable();
            next_locations.reverse();

            for &(_, orig_idx) in for_sort.iter() {
                let orig_idx_loc = idx_to_location[orig_idx]; // Retrieve real position
                let nxt_loc = next_locations.pop().unwrap();

                if orig_idx_loc == nxt_loc {
                    continue; // Already at the right place
                }

                // Simulate swaps in lookup tables
                let swapped_away = location_to_idx[nxt_loc];

                children.swap(orig_idx_loc, nxt_loc);
                location_to_idx.swap(orig_idx_loc, nxt_loc);

                idx_to_location[orig_idx] = nxt_loc;
                idx_to_location[swapped_away] = orig_idx_loc;
            }
        }

        for_sort.clear();
        idx_to_location.clear();
        location_to_idx.clear();
        next_locations.clear();
    }

    tracker.parent_entities.clear();
    tracker.children_order.clear();
}

/// Stores immediate mode managed parent entities and children entity order
#[derive(Resource)]
pub struct UiOrderTracker<Cap: CapSet> {
    parent_entities: HashMap<Entity, usize>,
    children_order: HashMap<Entity, usize>,
    _ph: PhantomData<Cap>,
}

impl<Cap: CapSet> UiOrderTracker<Cap> {
    /// Set of parent entities for immediate mode managed entities
    pub fn parent_entities(&self) -> &HashMap<Entity, usize> {
        &self.parent_entities
    }

    /// For each children stores their order in children component
    ///
    /// Children could store additional entities that are not managed by immediate mode
    pub fn children_order(&self) -> &HashMap<Entity, usize> {
        &self.children_order
    }
}

impl<Cap: CapSet> Default for UiOrderTracker<Cap> {
    fn default() -> Self {
        Self {
            parent_entities: Default::default(),
            children_order: Default::default(),
            _ph: PhantomData,
        }
    }
}
