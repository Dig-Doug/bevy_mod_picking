use crate::{PausedForBlockers, PickableMesh, PickingCamera, PickingRaycastSet};
use bevy::{prelude::*, ui::FocusPolicy};
use bevy_mod_raycast::{IntersectionData};

/// Tracks the current hover state to be used with change tracking in the events system.
///
/// # Requirements
///
/// An entity with the `Hover` component must also have an [Interaction] component.
#[derive(Component, Debug, Default, Clone, Reflect, PartialEq)]
#[reflect(Component, Default)]
pub struct Hover {
    #[reflect(ignore)]
    pub(crate) intersection: Option<IntersectionData>,
    #[reflect(ignore)]
    pub(crate) last_intersection: Option<IntersectionData>,
}

impl Hover {
    pub fn hovered(&self) -> bool {
        self.intersection.is_some()
    }
}

/// Marker component for entities that, whenever their [Interaction] component is anything other
/// than `None`, will suspend highlighting and selecting [PickableMesh]s. Bevy UI [Node]s have this
/// behavior by default.
#[derive(Component, Reflect, Default)]
#[reflect(Component)]
pub struct PickingBlocker;

#[allow(clippy::type_complexity)]
pub fn pause_for_picking_blockers(
    mut paused: ResMut<PausedForBlockers>,
    mut interactions: ParamSet<(
        Query<
            (
                &mut Interaction,
                Option<&mut Hover>,
                Option<&FocusPolicy>,
                Entity,
            ),
            With<PickableMesh>,
        >,
        // UI nodes are picking blockers by default.
        Query<&Interaction, Or<(With<Node>, With<PickingBlocker>)>>,
    )>,
) {
    paused.0 = false;
    for ui_interaction in interactions.p1().iter() {
        if *ui_interaction != Interaction::None {
            for (mut interaction, hover, _, _) in &mut interactions.p0().iter_mut() {
                if *interaction != Interaction::None {
                    *interaction = Interaction::None;
                }
                if let Some(mut hover) = hover {
                    if hover.hovered() {
                        hover.intersection = None;
                    }
                }
            }
            paused.0 = true;
            return;
        }
    }
}

#[allow(clippy::type_complexity)]
pub fn mesh_focus(
    paused: Option<Res<PausedForBlockers>>,
    mouse_button_input: Res<Input<MouseButton>>,
    touches_input: Res<Touches>,
    pick_source_query: Query<&PickingCamera>,
    mut interactions: Query<
        (
            &mut Interaction,
            Option<&mut Hover>,
            Option<&FocusPolicy>,
            Entity,
        ),
        With<PickableMesh>,
    >,
) {
    if let Some(paused) = paused {
        if paused.0 {
            return;
        }
    }

    let mut hovered_entity = None;
    let mut hovered_intersection = None;

    if mouse_button_input.just_released(MouseButton::Left)
        || touches_input.iter_just_released().next().is_some()
    {
        for (mut interaction, _, _, _) in &mut interactions.iter_mut() {
            if *interaction == Interaction::Clicked {
                *interaction = Interaction::None;
            }
        }
    }

    let mouse_clicked = mouse_button_input.just_pressed(MouseButton::Left)
        || touches_input.iter_just_pressed().next().is_some();
    for pick_source in pick_source_query.iter() {
        for (topmost_entity, _intersection) in pick_source.intersections().iter() {
            if let Ok((mut interaction, _hover, focus_policy, _entity)) =
                interactions.get_mut(*topmost_entity)
            {
                if mouse_clicked {
                    if *interaction != Interaction::Clicked {
                        *interaction = Interaction::Clicked;
                    }
                } else if *interaction == Interaction::None {
                    *interaction = Interaction::Hovered;
                }

                hovered_entity = Some(*topmost_entity);
                hovered_intersection = Some(_intersection);

                match focus_policy.cloned().unwrap_or(FocusPolicy::Block) {
                    FocusPolicy::Block => {
                        break;
                    }
                    FocusPolicy::Pass => { /* allow the next node to be hovered/clicked */ }
                }
            }
        }

        for (mut interaction, hover, _, entity) in &mut interactions.iter_mut() {
            let is_hovered_entity = Some(entity) == hovered_entity;
            if !is_hovered_entity && *interaction == Interaction::Hovered {
                *interaction = Interaction::None;
            }
          
            if let Some(mut hover) = hover {
                let new_state = Hover {
                    last_intersection:  if is_hovered_entity {
                        hover.intersection.clone()
                    } else {
                        None
                    },
                    intersection: if is_hovered_entity {
                        hovered_intersection.cloned()
                    } else {
                        None
                    }
                };
                if new_state != *hover {
                    *hover = new_state;
                }
            }
        }
    }
}
