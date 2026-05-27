use super::*;

pub fn combined_goal_tag_events(calculators: &[&[GoalTagEvent]]) -> Vec<GoalTagEvent> {
    let mut events: Vec<_> = calculators
        .iter()
        .flat_map(|events| events.iter().cloned())
        .collect();
    events.sort_by(|left, right| {
        left.time
            .total_cmp(&right.time)
            .then_with(|| left.frame.cmp(&right.frame))
            .then_with(|| left.goal_index.cmp(&right.goal_index))
            .then_with(|| format!("{:?}", left.kind).cmp(&format!("{:?}", right.kind)))
    });
    events
}
