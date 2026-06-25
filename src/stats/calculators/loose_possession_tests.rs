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

fn p(id: u64) -> PlayerId {
    PlayerId::Steam(id)
}

/// Drives the loose resolver through a sequence of frames and collects every
/// finalized segment (live resolutions plus the end-of-stretch flush).
struct LooseSim {
    resolver: LoosePossessionResolver,
    resolved: Vec<ResolvedPossession>,
}

impl LooseSim {
    fn new() -> Self {
        let mut resolver = LoosePossessionResolver::default();
        resolver.begin(&frame_info(0, 0.0));
        Self {
            resolver,
            resolved: Vec::new(),
        }
    }

    fn step(&mut self, frame: usize, time: f32, touches: &[TouchEvent]) {
        let touched_team_zero = touches.iter().any(|t| t.team_is_team_0);
        let touched_team_one = touches.iter().any(|t| !t.team_is_team_0);
        let team_zero_player = touches
            .iter()
            .rfind(|t| t.team_is_team_0)
            .and_then(|t| t.player.clone());
        let team_one_player = touches
            .iter()
            .rfind(|t| !t.team_is_team_0)
            .and_then(|t| t.player.clone());
        self.resolver.update(
            &frame_info(frame, time),
            &team_zero_player,
            &team_one_player,
            touched_team_zero,
            touched_team_one,
        );
        self.resolved
            .extend(std::mem::take(&mut self.resolver.newly_resolved));
    }

    fn finish(&mut self, frame: usize, time: f32) {
        self.resolver.flush(time, frame);
        self.resolved
            .extend(std::mem::take(&mut self.resolver.newly_resolved));
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

#[test]
fn single_touch_grants_loose_possession_until_opponent_takes_over() {
    // The spec timeline: blue touches (t0, t1), ball goes loose, orange takes it
    // (t3) and confirms (t4). Blue owns the loose tail t1->t3, the turnover is
    // backdated to orange's first touch (t3), and there is no neutral gap.
    let mut sim = LooseSim::new();
    sim.step(10, 1.0, &[touch(10, 1.0, p(1), true)]); // blue first touch
    sim.step(13, 1.3, &[touch(13, 1.3, p(1), true)]); // blue last touch
    sim.step(20, 2.0, &[touch(20, 2.0, p(2), false)]); // orange challenges
    sim.step(24, 2.4, &[touch(24, 2.4, p(2), false)]); // orange confirms
    sim.finish(40, 4.0);

    assert_eq!(
        sim.team_segments(),
        vec![(true, 1.0, 2.0), (false, 2.0, 4.0)]
    );
}

#[test]
fn lone_touch_then_loose_stays_with_last_toucher() {
    // Blue gets a single touch off a neutral ball and nobody follows up. Under
    // the loose definition blue is the last to touch, so the ball stays blue's
    // (sticky) all the way to the end of the stretch.
    let mut sim = LooseSim::new();
    sim.step(10, 1.0, &[touch(10, 1.0, p(1), true)]);
    sim.step(90, 3.0, &[]); // long loose stretch, no follow-up
    sim.finish(120, 4.0);

    assert_eq!(sim.team_segments(), vec![(true, 1.0, 4.0)]);
}

#[test]
fn repelled_poke_keeps_holder_continuous() {
    // Blue holds; orange gets a single clearing touch; blue recovers before the
    // window elapses. The challenge is repelled, so blue's possession stays one
    // continuous span and orange is credited nothing.
    let mut sim = LooseSim::new();
    sim.step(10, 1.0, &[touch(10, 1.0, p(1), true)]);
    sim.step(13, 1.3, &[touch(13, 1.3, p(1), true)]);
    sim.step(20, 2.0, &[touch(20, 2.0, p(2), false)]); // orange poke
    sim.step(23, 2.3, &[touch(23, 2.3, p(1), true)]); // blue repels
    sim.finish(40, 4.0);

    assert_eq!(sim.team_segments(), vec![(true, 1.0, 4.0)]);
}

#[test]
fn unfollowed_challenge_turns_over_to_challenger() {
    // Blue holds; orange pokes once and nobody follows up within the window.
    // Orange was the last to touch, so the ball becomes orange's from its touch.
    let mut sim = LooseSim::new();
    sim.step(10, 1.0, &[touch(10, 1.0, p(1), true)]);
    sim.step(20, 2.0, &[touch(20, 2.0, p(2), false)]); // orange poke
    sim.step(80, 3.7, &[]); // >1.5s with no follow-up
    sim.finish(120, 4.0);

    assert_eq!(
        sim.team_segments(),
        vec![(true, 1.0, 2.0), (false, 2.0, 4.0)]
    );
}

#[test]
fn loose_possession_covers_the_whole_live_stretch_except_the_lead_in() {
    // A normal exchange: every non-neutral segment is contiguous, and the only
    // neutral time is the lead-in before the first touch.
    let mut sim = LooseSim::new();
    sim.step(10, 1.0, &[touch(10, 1.0, p(1), true)]);
    sim.step(20, 2.0, &[touch(20, 2.0, p(2), false)]);
    sim.step(24, 2.4, &[touch(24, 2.4, p(2), false)]);
    sim.finish(40, 4.0);

    let neutral: f32 = sim
        .resolved
        .iter()
        .filter(|s| s.label == PossessionLabel::Neutral)
        .map(|s| s.end_time - s.start_time)
        .sum();
    let credited: f32 = sim
        .team_segments()
        .iter()
        .map(|(_, start, end)| end - start)
        .sum();
    assert!(
        (neutral - 1.0).abs() < 1e-6,
        "neutral lead-in should be 1.0s"
    );
    assert!(
        (credited - 3.0).abs() < 1e-6,
        "rest of the stretch is credited"
    );
}
