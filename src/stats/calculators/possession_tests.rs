use super::*;

fn frame_info(frame_number: usize, time: f32) -> FrameInfo {
    FrameInfo {
        frame_number,
        time,
        dt: 1.0 / 30.0,
        seconds_remaining: None,
    }
}

fn touch(frame: usize, time: f32, player: PlayerId, team_is_team_0: bool) -> TouchEvent {
    TouchEvent {
        touch_id: None,
        time,
        frame,
        team_is_team_0,
        player: Some(player),
        player_position: None,
        closest_approach_distance: Some(0.0),
        contact_local_ball_position: None,
        contact_local_hitbox_point: None,
        contact_world_hitbox_point: None,
        dodge_contact: false,
    }
}

/// Drives the backdating resolver through a sequence of frames and collects
/// every finalized segment (live resolutions plus the end-of-stretch flush).
struct ResolverSim {
    tracker: PossessionTracker,
    resolved: Vec<ResolvedPossession>,
}

impl ResolverSim {
    fn new() -> Self {
        let mut tracker = PossessionTracker::default();
        tracker.begin_resolver(&frame_info(0, 0.0));
        Self {
            tracker,
            resolved: Vec::new(),
        }
    }

    fn step(&mut self, frame: usize, time: f32, touches: &[TouchEvent]) {
        let state = self.tracker.update(&frame_info(frame, time), touches);
        self.resolved.extend(state.newly_resolved);
    }

    fn finish(&mut self, frame: usize, time: f32) {
        self.resolved
            .extend(self.tracker.flush_resolver(&frame_info(frame, time)));
    }

    /// The credited (non-neutral) segments as `(team_is_team_0, start, end)`.
    fn team_segments(&self) -> Vec<(bool, f32, f32)> {
        self.resolved
            .iter()
            .filter_map(|segment| match segment.label {
                PossessionLabel::TeamZero => Some((true, segment.start_time, segment.end_time)),
                PossessionLabel::TeamOne => Some((false, segment.start_time, segment.end_time)),
                PossessionLabel::Neutral => None,
            })
            .collect()
    }
}

fn p(id: u64) -> PlayerId {
    PlayerId::Steam(id)
}

#[test]
fn single_kickoff_touch_then_lost_credits_no_possession_to_first_toucher() {
    // Blue (team 0) gets one kickoff touch and hits it away; orange (team 1)
    // recovers and controls. Blue's lone touch must grant nothing.
    let mut sim = ResolverSim::new();
    sim.step(10, 1.0, &[touch(10, 1.0, p(1), true)]); // blue, single → unconfirmed
    sim.step(20, 2.0, &[touch(20, 2.0, p(2), false)]); // orange takes the loose ball
    sim.step(25, 2.5, &[touch(25, 2.5, p(2), false)]); // orange confirms control
    sim.finish(40, 4.0);

    // Only orange is credited, and only from their first touch.
    assert_eq!(sim.team_segments(), vec![(false, 2.0, 2.5)]);
}

#[test]
fn brief_control_with_no_follow_up_is_backdated_to_last_touch() {
    // Blue controls briefly (two touches) then loses it with no follow-up. The
    // loose tail after the last touch is neutral, not blue's.
    let mut sim = ResolverSim::new();
    sim.step(10, 1.0, &[touch(10, 1.0, p(1), true)]);
    sim.step(13, 1.3, &[touch(13, 1.3, p(1), true)]); // confirm
    sim.step(90, 3.0, &[]); // 1.7s with no touch → loss, backdated to 1.3
    sim.finish(120, 4.0);

    assert_eq!(sim.team_segments(), vec![(true, 1.0, 1.3)]);
}

#[test]
fn sustained_dribble_is_one_continuous_segment() {
    let mut sim = ResolverSim::new();
    for (frame, time) in [
        (10, 1.0),
        (13, 1.3),
        (16, 1.6),
        (19, 1.9),
        (22, 2.2),
        (25, 2.5),
    ] {
        sim.step(frame, time, &[touch(frame, time, p(1), true)]);
    }
    sim.finish(40, 4.0);

    // One uninterrupted blue possession from the first touch to the last.
    assert_eq!(sim.team_segments(), vec![(true, 1.0, 2.5)]);
}

#[test]
fn unconfirmed_opponent_clear_grants_nothing_and_keeps_holder_continuous() {
    // Blue holds; orange gets a single clearing touch; blue recovers before
    // orange confirms. Orange's lone clear grants nothing and does not break
    // blue's possession — the hold stays continuous through the repelled poke.
    let mut sim = ResolverSim::new();
    sim.step(10, 1.0, &[touch(10, 1.0, p(1), true)]);
    sim.step(13, 1.3, &[touch(13, 1.3, p(1), true)]); // blue confirms
    sim.step(18, 1.8, &[touch(18, 1.8, p(1), true)]); // blue extends
    sim.step(20, 2.0, &[touch(20, 2.0, p(2), false)]); // orange single clear
    sim.step(23, 2.3, &[touch(23, 2.3, p(1), true)]); // blue repels
    sim.finish(40, 4.0);

    // Orange credited nothing; blue's possession is one continuous span.
    assert_eq!(sim.team_segments(), vec![(true, 1.0, 2.3)]);
}

#[test]
fn confirmed_turnover_credits_winner_from_first_touch_with_neutral_gap() {
    // Blue holds, orange wins it with two touches. Blue ends at its last touch,
    // the loose gap is neutral, orange is credited from its first touch.
    let mut sim = ResolverSim::new();
    sim.step(10, 1.0, &[touch(10, 1.0, p(1), true)]);
    sim.step(13, 1.3, &[touch(13, 1.3, p(1), true)]); // blue confirms, last touch 1.3
    sim.step(20, 2.0, &[touch(20, 2.0, p(2), false)]); // orange challenges
    sim.step(24, 2.4, &[touch(24, 2.4, p(2), false)]); // orange confirms turnover
    sim.finish(40, 4.0);

    assert_eq!(
        sim.resolved
            .iter()
            .map(|s| (s.label, s.start_time, s.end_time))
            .collect::<Vec<_>>(),
        vec![
            (PossessionLabel::Neutral, 0.0, 1.0),
            (PossessionLabel::TeamZero, 1.0, 1.3),
            (PossessionLabel::Neutral, 1.3, 2.0),
            (PossessionLabel::TeamOne, 2.0, 2.4),
            (PossessionLabel::Neutral, 2.4, 4.0),
        ]
    );
}

#[test]
fn tracker_uses_latest_touch_player_for_team_independent_of_slice_order() {
    let earlier_player = PlayerId::Steam(1);
    let later_player = PlayerId::Steam(2);
    let mut tracker = PossessionTracker::default();
    let later_touch = touch(20, 2.0, later_player.clone(), true);
    let earlier_touch = touch(10, 1.0, earlier_player, true);

    let state = tracker.update(&frame_info(20, 2.0), &[later_touch, earlier_touch]);

    assert_eq!(state.current_team_is_team_0, Some(true));
    assert_eq!(state.current_player, Some(later_player));
}
