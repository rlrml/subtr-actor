use super::*;

pub(super) fn append_ceiling_shot_events(
    events: &mut Vec<MechanicEvent>,
    ceiling_shot: &CeilingShotCalculator,
) {
    for (index, event) in ceiling_shot.events().iter().enumerate() {
        events.push(span_mechanic_event(
            MECHANIC_CEILING_SHOT,
            index,
            event.ceiling_contact_frame,
            event.frame,
            event.ceiling_contact_time,
            event.time,
            event.player.clone(),
            event.is_team_0,
        ));
    }
}

pub(super) fn append_wall_aerial_events(
    events: &mut Vec<MechanicEvent>,
    wall_aerial: &WallAerialCalculator,
) {
    for (index, event) in wall_aerial.events().iter().enumerate() {
        let mut mechanic_event = span_mechanic_event(
            MECHANIC_WALL_AERIAL,
            index,
            event.wall_contact_frame,
            event.frame,
            event.wall_contact_time,
            event.time,
            event.player.clone(),
            event.is_team_0,
        );
        mechanic_event.properties = vec![mechanic_event_text_property(
            "wall",
            event.wall.as_label_value(),
        )];
        events.push(mechanic_event);
    }
}

pub(super) fn append_wall_aerial_shot_events(
    events: &mut Vec<MechanicEvent>,
    wall_aerial_shot: &WallAerialShotCalculator,
) {
    for (index, event) in wall_aerial_shot.events().iter().enumerate() {
        let mut mechanic_event = span_mechanic_event(
            MECHANIC_WALL_AERIAL_SHOT,
            index,
            event.takeoff_frame,
            event.frame,
            event.takeoff_time,
            event.time,
            event.player.clone(),
            event.is_team_0,
        );
        mechanic_event.properties = vec![mechanic_event_text_property(
            "wall",
            event.wall.as_label_value(),
        )];
        events.push(mechanic_event);
    }
}
