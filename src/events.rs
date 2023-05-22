use crate::{Hover, PickableMesh, Selection};
use bevy::prelude::*;
use bevy_mod_raycast::IntersectionData;

/// An event that triggers when the selection state of a [Selection] enabled [PickableMesh] changes.
#[derive(Debug)]
pub enum SelectionEvent {
    JustSelected(Entity),
    JustDeselected(Entity),
}

/// An event that triggers when the hover state of a [Hover] enabled [PickableMesh] changes.
#[derive(Debug)]
pub enum HoverEvent {
    JustEntered{entity: Entity, intersection: IntersectionData},
    HoverOver{entity: Entity, intersection: IntersectionData},
    JustLeft(Entity),
}

/// An event that wraps selection and hover events
#[derive(Debug)]
pub enum PickingEvent {
    Selection(SelectionEvent),
    Hover(HoverEvent),
    Clicked{entity: Entity, intersection: IntersectionData},
}

/// Looks for changes in selection or hover state, and sends the appropriate events
#[allow(clippy::type_complexity)]
pub fn mesh_events_system(
    mouse_button_input: Res<Input<MouseButton>>,
    touches_input: Res<Touches>,
    mut picking_events: EventWriter<PickingEvent>,
    hover_query: Query<(Entity, Ref<Hover>), (Changed<Hover>, With<PickableMesh>)>,
    selection_query: Query<(Entity, Ref<Selection>), (Changed<Selection>, With<PickableMesh>)>,
    click_query: Query<(Entity, &Hover)>,
) {
    for (entity, hover) in hover_query.iter() {
        if hover.is_added() {
            continue; // Avoid a false change detection when a component is added.
        }
        if let Some(intersection) = hover.intersection.clone() {
            if hover.last_intersection.is_none() {
                picking_events.send(PickingEvent::Hover(HoverEvent::JustEntered{entity, intersection}));
            } else {
                picking_events.send(PickingEvent::Hover(HoverEvent::HoverOver{entity, intersection}));
            }
        } else {
            picking_events.send(PickingEvent::Hover(HoverEvent::JustLeft(entity)));
        }
    }
    for (entity, selection) in selection_query.iter() {
        if selection.is_added() {
            continue; // Avoid a false change detection when a component is added.
        }
        if selection.selected() {
            picking_events.send(PickingEvent::Selection(SelectionEvent::JustSelected(
                entity,
            )));
        } else {
            picking_events.send(PickingEvent::Selection(SelectionEvent::JustDeselected(
                entity,
            )));
        }
    }
    if mouse_button_input.just_pressed(MouseButton::Left)
        || touches_input.iter_just_pressed().next().is_some()
    {
        for (entity, hover) in click_query.iter() {
            if let Some(intersection) = &hover.intersection {
                picking_events.send(PickingEvent::Clicked{entity, intersection: intersection.clone() });
            }
        }
    }
}

/// Listens for [HoverEvent] and [SelectionEvent] events and prints them
pub fn event_debug_system(mut events: EventReader<PickingEvent>) {
    for event in events.iter() {
        info!("{:?}", event);
    }
}
