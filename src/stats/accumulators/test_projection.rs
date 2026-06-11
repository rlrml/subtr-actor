#![allow(dead_code)]

use super::*;
use std::collections::HashMap;

fn stats_test_frame(time: f32, frame_number: usize) -> FrameInfo {
    FrameInfo {
        frame_number,
        time,
        dt: 0.0,
        seconds_remaining: None,
    }
}

fn leak_test_stats<T: 'static>(value: T) -> &'static T {
    Box::leak(Box::new(value))
}

pub(crate) trait BallCarryTestProjection {
    fn player_stats(&self) -> &HashMap<PlayerId, BallCarryStats>;
    fn player_air_dribble_stats(&self) -> &HashMap<PlayerId, AirDribbleStats>;
    fn team_zero_stats(&self) -> &BallCarryStats;
    fn team_one_stats(&self) -> &BallCarryStats;
    fn team_zero_air_dribble_stats(&self) -> &AirDribbleStats;
    fn team_one_air_dribble_stats(&self) -> &AirDribbleStats;
}

impl BallCarryTestProjection for BallCarryCalculator {
    fn player_stats(&self) -> &HashMap<PlayerId, BallCarryStats> {
        let mut stats = BallCarryStatsAccumulator::default();
        for event in self.carry_events() {
            stats.apply_event(event);
        }
        leak_test_stats(stats.player_stats().clone())
    }

    fn player_air_dribble_stats(&self) -> &HashMap<PlayerId, AirDribbleStats> {
        let mut stats = BallCarryStatsAccumulator::default();
        for event in self.carry_events() {
            stats.apply_event(event);
        }
        leak_test_stats(stats.player_air_dribble_stats().clone())
    }

    fn team_zero_stats(&self) -> &BallCarryStats {
        let mut stats = BallCarryStatsAccumulator::default();
        for event in self.carry_events() {
            stats.apply_event(event);
        }
        leak_test_stats(stats.team_zero_stats().clone())
    }

    fn team_one_stats(&self) -> &BallCarryStats {
        let mut stats = BallCarryStatsAccumulator::default();
        for event in self.carry_events() {
            stats.apply_event(event);
        }
        leak_test_stats(stats.team_one_stats().clone())
    }

    fn team_zero_air_dribble_stats(&self) -> &AirDribbleStats {
        let mut stats = BallCarryStatsAccumulator::default();
        for event in self.carry_events() {
            stats.apply_event(event);
        }
        leak_test_stats(stats.team_zero_air_dribble_stats().clone())
    }

    fn team_one_air_dribble_stats(&self) -> &AirDribbleStats {
        let mut stats = BallCarryStatsAccumulator::default();
        for event in self.carry_events() {
            stats.apply_event(event);
        }
        leak_test_stats(stats.team_one_air_dribble_stats().clone())
    }
}

pub(crate) trait BoostTestProjection {
    fn player_stats(&self) -> &HashMap<PlayerId, BoostStats>;
    fn team_zero_stats(&self) -> &BoostStats;
    fn team_one_stats(&self) -> &BoostStats;
}

impl BoostTestProjection for BoostCalculator {
    fn player_stats(&self) -> &HashMap<PlayerId, BoostStats> {
        leak_test_stats(self.boost_stats().player_stats().clone())
    }

    fn team_zero_stats(&self) -> &BoostStats {
        leak_test_stats(self.boost_stats().team_zero_stats().clone())
    }

    fn team_one_stats(&self) -> &BoostStats {
        leak_test_stats(self.boost_stats().team_one_stats().clone())
    }
}

pub(crate) trait BumpTestProjection {
    fn player_stats(&self) -> &HashMap<PlayerId, BumpPlayerStats>;
    fn team_zero_stats(&self) -> &BumpTeamStats;
    fn team_one_stats(&self) -> &BumpTeamStats;
    fn stats(&self) -> &BumpStatsAccumulator;
}

impl BumpTestProjection for BumpCalculator {
    fn player_stats(&self) -> &HashMap<PlayerId, BumpPlayerStats> {
        let stats = BumpStatsAccumulator::from_events(self.events());
        leak_test_stats(stats.player_stats().clone())
    }

    fn team_zero_stats(&self) -> &BumpTeamStats {
        let stats = BumpStatsAccumulator::from_events(self.events());
        leak_test_stats(stats.team_zero_stats().clone())
    }

    fn team_one_stats(&self) -> &BumpTeamStats {
        let stats = BumpStatsAccumulator::from_events(self.events());
        leak_test_stats(stats.team_one_stats().clone())
    }

    fn stats(&self) -> &BumpStatsAccumulator {
        leak_test_stats(BumpStatsAccumulator::from_events(self.events()))
    }
}

pub(crate) trait CenterTestProjection {
    fn player_stats(&self) -> &HashMap<PlayerId, CenterPlayerStats>;
    fn team_zero_stats(&self) -> &CenterTeamStats;
    fn team_one_stats(&self) -> &CenterTeamStats;
}

impl CenterTestProjection for CenterCalculator {
    fn player_stats(&self) -> &HashMap<PlayerId, CenterPlayerStats> {
        let mut stats = CenterStatsAccumulator::default();
        for event in self.events() {
            let frame = stats_test_frame(event.time, event.frame);
            stats.apply_event(&frame, event);
        }
        leak_test_stats(stats.player_stats().clone())
    }

    fn team_zero_stats(&self) -> &CenterTeamStats {
        let mut stats = CenterStatsAccumulator::default();
        for event in self.events() {
            let frame = stats_test_frame(event.time, event.frame);
            stats.apply_event(&frame, event);
        }
        leak_test_stats(stats.team_zero_stats().clone())
    }

    fn team_one_stats(&self) -> &CenterTeamStats {
        let mut stats = CenterStatsAccumulator::default();
        for event in self.events() {
            let frame = stats_test_frame(event.time, event.frame);
            stats.apply_event(&frame, event);
        }
        leak_test_stats(stats.team_one_stats().clone())
    }
}

pub(crate) trait DemoTestProjection {
    fn player_stats(&self) -> &HashMap<PlayerId, DemoPlayerStats>;
    fn team_zero_stats(&self) -> &DemoTeamStats;
    fn team_one_stats(&self) -> &DemoTeamStats;
}

impl DemoTestProjection for DemoCalculator {
    fn player_stats(&self) -> &HashMap<PlayerId, DemoPlayerStats> {
        let mut stats = DemoStatsAccumulator::default();
        for event in self.timeline() {
            stats.apply_timeline_event(event);
        }
        leak_test_stats(stats.player_stats().clone())
    }

    fn team_zero_stats(&self) -> &DemoTeamStats {
        let mut stats = DemoStatsAccumulator::default();
        for event in self.timeline() {
            stats.apply_timeline_event(event);
        }
        leak_test_stats(stats.team_zero_stats().clone())
    }

    fn team_one_stats(&self) -> &DemoTeamStats {
        let mut stats = DemoStatsAccumulator::default();
        for event in self.timeline() {
            stats.apply_timeline_event(event);
        }
        leak_test_stats(stats.team_one_stats().clone())
    }
}

pub(crate) trait DodgeResetTestProjection {
    fn player_stats(&self) -> &HashMap<PlayerId, DodgeResetStats>;
}

impl DodgeResetTestProjection for DodgeResetCalculator {
    fn player_stats(&self) -> &HashMap<PlayerId, DodgeResetStats> {
        let mut stats = DodgeResetStatsAccumulator::default();
        for event in self.events() {
            stats.apply_event(event);
        }
        for event in self.flip_reset_outcome_events() {
            stats.apply_flip_reset_outcome_event(event);
        }
        leak_test_stats(stats.player_stats().clone())
    }
}

pub(crate) trait DoubleTapTestProjection {
    fn player_stats(&self) -> &HashMap<PlayerId, DoubleTapPlayerStats>;
    fn team_zero_stats(&self) -> &DoubleTapTeamStats;
    fn team_one_stats(&self) -> &DoubleTapTeamStats;
}

impl DoubleTapTestProjection for DoubleTapCalculator {
    fn player_stats(&self) -> &HashMap<PlayerId, DoubleTapPlayerStats> {
        let mut stats = DoubleTapStatsAccumulator::default();
        for event in self.events() {
            let frame = stats_test_frame(event.time, event.frame);
            stats.apply_event(&frame, event);
        }
        leak_test_stats(stats.player_stats().clone())
    }

    fn team_zero_stats(&self) -> &DoubleTapTeamStats {
        let mut stats = DoubleTapStatsAccumulator::default();
        for event in self.events() {
            let frame = stats_test_frame(event.time, event.frame);
            stats.apply_event(&frame, event);
        }
        leak_test_stats(stats.team_zero_stats().clone())
    }

    fn team_one_stats(&self) -> &DoubleTapTeamStats {
        let mut stats = DoubleTapStatsAccumulator::default();
        for event in self.events() {
            let frame = stats_test_frame(event.time, event.frame);
            stats.apply_event(&frame, event);
        }
        leak_test_stats(stats.team_one_stats().clone())
    }
}

pub(crate) trait FiftyFiftyTestProjection {
    fn stats(&self) -> &FiftyFiftyStats;
    fn player_stats(&self) -> &HashMap<PlayerId, FiftyFiftyPlayerStats>;
}

impl FiftyFiftyTestProjection for FiftyFiftyCalculator {
    fn stats(&self) -> &FiftyFiftyStats {
        let mut stats = FiftyFiftyStatsAccumulator::default();
        for event in self.events() {
            stats.apply_event(event);
        }
        leak_test_stats(stats.stats().clone())
    }

    fn player_stats(&self) -> &HashMap<PlayerId, FiftyFiftyPlayerStats> {
        let mut stats = FiftyFiftyStatsAccumulator::default();
        for event in self.events() {
            stats.apply_event(event);
        }
        leak_test_stats(stats.player_stats().clone())
    }
}

pub(crate) trait FlickTestProjection {
    fn player_stats(&self) -> &HashMap<PlayerId, FlickStats>;
}

impl FlickTestProjection for FlickCalculator {
    fn player_stats(&self) -> &HashMap<PlayerId, FlickStats> {
        let mut stats = FlickStatsAccumulator::default();
        for event in self.events() {
            let frame = stats_test_frame(event.sample_time, event.sample_frame);
            stats.apply_event(event, &frame);
        }
        leak_test_stats(stats.player_stats().clone())
    }
}

pub(crate) trait HalfFlipTestProjection {
    fn player_stats(&self) -> &HashMap<PlayerId, HalfFlipStats>;
}

impl HalfFlipTestProjection for HalfFlipCalculator {
    fn player_stats(&self) -> &HashMap<PlayerId, HalfFlipStats> {
        let mut stats = HalfFlipStatsAccumulator::default();
        for event in self.events() {
            stats.apply_event(event);
        }
        leak_test_stats(stats.player_stats().clone())
    }
}

pub(crate) trait HalfVolleyTestProjection {
    fn player_stats(&self) -> &HashMap<PlayerId, HalfVolleyPlayerStats>;
    fn team_zero_stats(&self) -> &HalfVolleyTeamStats;
    fn team_one_stats(&self) -> &HalfVolleyTeamStats;
}

impl HalfVolleyTestProjection for HalfVolleyCalculator {
    fn player_stats(&self) -> &HashMap<PlayerId, HalfVolleyPlayerStats> {
        let mut stats = HalfVolleyStatsAccumulator::default();
        for event in self.events() {
            let frame = stats_test_frame(event.sample_time, event.sample_frame);
            stats.apply_event(event, &frame);
        }
        leak_test_stats(stats.player_stats().clone())
    }

    fn team_zero_stats(&self) -> &HalfVolleyTeamStats {
        let mut stats = HalfVolleyStatsAccumulator::default();
        for event in self.events() {
            let frame = stats_test_frame(event.sample_time, event.sample_frame);
            stats.apply_event(event, &frame);
        }
        leak_test_stats(stats.team_zero_stats().clone())
    }

    fn team_one_stats(&self) -> &HalfVolleyTeamStats {
        let mut stats = HalfVolleyStatsAccumulator::default();
        for event in self.events() {
            let frame = stats_test_frame(event.sample_time, event.sample_frame);
            stats.apply_event(event, &frame);
        }
        leak_test_stats(stats.team_one_stats().clone())
    }
}

pub(crate) trait MatchStatsTestProjection {
    fn player_stats(&self) -> &HashMap<PlayerId, CorePlayerStats>;
    fn team_zero_stats(&self) -> CoreTeamStats;
    fn team_one_stats(&self) -> CoreTeamStats;
}

impl MatchStatsTestProjection for MatchStatsCalculator {
    fn player_stats(&self) -> &HashMap<PlayerId, CorePlayerStats> {
        let mut stats = CoreStatsAccumulator::default();
        for event in self.core_player_events() {
            stats.apply_scoreboard_event(event);
        }
        for event in self.core_player_goal_context_events() {
            stats.apply_goal_context_event(event);
        }
        leak_test_stats(stats.player_stats().clone())
    }

    fn team_zero_stats(&self) -> CoreTeamStats {
        let mut stats = CoreStatsAccumulator::default();
        for event in self.core_player_events() {
            stats.apply_scoreboard_event(event);
        }
        for event in self.core_player_goal_context_events() {
            stats.apply_goal_context_event(event);
        }
        stats.team_zero_stats()
    }

    fn team_one_stats(&self) -> CoreTeamStats {
        let mut stats = CoreStatsAccumulator::default();
        for event in self.core_player_events() {
            stats.apply_scoreboard_event(event);
        }
        for event in self.core_player_goal_context_events() {
            stats.apply_goal_context_event(event);
        }
        stats.team_one_stats()
    }
}

pub(crate) trait OneTimerTestProjection {
    fn player_stats(&self) -> &HashMap<PlayerId, OneTimerPlayerStats>;
    fn team_zero_stats(&self) -> &OneTimerTeamStats;
    fn team_one_stats(&self) -> &OneTimerTeamStats;
}

impl OneTimerTestProjection for OneTimerCalculator {
    fn player_stats(&self) -> &HashMap<PlayerId, OneTimerPlayerStats> {
        let mut stats = OneTimerStatsAccumulator::default();
        for event in self.events() {
            let frame = stats_test_frame(event.time, event.frame);
            stats.apply_event(&frame, event);
        }
        leak_test_stats(stats.player_stats().clone())
    }

    fn team_zero_stats(&self) -> &OneTimerTeamStats {
        let mut stats = OneTimerStatsAccumulator::default();
        for event in self.events() {
            let frame = stats_test_frame(event.time, event.frame);
            stats.apply_event(&frame, event);
        }
        leak_test_stats(stats.team_zero_stats().clone())
    }

    fn team_one_stats(&self) -> &OneTimerTeamStats {
        let mut stats = OneTimerStatsAccumulator::default();
        for event in self.events() {
            let frame = stats_test_frame(event.time, event.frame);
            stats.apply_event(&frame, event);
        }
        leak_test_stats(stats.team_one_stats().clone())
    }
}

pub(crate) trait PassTestProjection {
    fn player_stats(&self) -> &HashMap<PlayerId, PassPlayerStats>;
    fn team_zero_stats(&self) -> &PassTeamStats;
    fn team_one_stats(&self) -> &PassTeamStats;
}

impl PassTestProjection for PassCalculator {
    fn player_stats(&self) -> &HashMap<PlayerId, PassPlayerStats> {
        let mut stats = PassStatsAccumulator::default();
        for event in self.events() {
            let frame = stats_test_frame(event.sample_time, event.sample_frame);
            stats.apply_event(&frame, event);
        }
        leak_test_stats(stats.player_stats().clone())
    }

    fn team_zero_stats(&self) -> &PassTeamStats {
        let mut stats = PassStatsAccumulator::default();
        for event in self.events() {
            let frame = stats_test_frame(event.sample_time, event.sample_frame);
            stats.apply_event(&frame, event);
        }
        leak_test_stats(stats.team_zero_stats().clone())
    }

    fn team_one_stats(&self) -> &PassTeamStats {
        let mut stats = PassStatsAccumulator::default();
        for event in self.events() {
            let frame = stats_test_frame(event.sample_time, event.sample_frame);
            stats.apply_event(&frame, event);
        }
        leak_test_stats(stats.team_one_stats().clone())
    }
}

pub(crate) trait PositioningTestProjection {
    fn player_stats(&self) -> &HashMap<PlayerId, PositioningStats>;
}

impl PositioningTestProjection for PositioningCalculator {
    fn player_stats(&self) -> &HashMap<PlayerId, PositioningStats> {
        let mut stats = PositioningStatsAccumulator::default();
        for event in self.activity_events().iter() {
            stats.apply_activity_event(event);
        }
        for event in self.field_third_events().iter() {
            stats.apply_field_third_event(event);
        }
        for event in self.field_half_events().iter() {
            stats.apply_field_half_event(event);
        }
        for event in self.ball_depth_events().iter() {
            stats.apply_ball_depth_event(event);
        }
        for event in self.depth_role_events().iter() {
            stats.apply_depth_role_event(event);
        }
        for event in self.ball_proximity_events().iter() {
            stats.apply_ball_proximity_event(event);
        }
        for (player, signal) in self.signals() {
            stats.apply_signal(player, signal);
        }
        leak_test_stats(stats.player_stats().clone())
    }
}

pub(crate) trait RotationTestProjection {
    fn player_stats(&self) -> &HashMap<PlayerId, RotationPlayerStats>;
    fn team_zero_stats(&self) -> &RotationTeamStats;
    fn team_one_stats(&self) -> &RotationTeamStats;
}

impl RotationTestProjection for RotationCalculator {
    fn player_stats(&self) -> &HashMap<PlayerId, RotationPlayerStats> {
        leak_test_stats(rotation_projection(self).player_stats().clone())
    }

    fn team_zero_stats(&self) -> &RotationTeamStats {
        leak_test_stats(rotation_projection(self).team_zero_stats().clone())
    }

    fn team_one_stats(&self) -> &RotationTeamStats {
        leak_test_stats(rotation_projection(self).team_one_stats().clone())
    }
}

fn rotation_projection(calculator: &RotationCalculator) -> RotationStatsAccumulator {
    let mut stats = RotationStatsAccumulator::with_first_man_stint_end_grace_seconds(
        calculator.config().first_man_stint_end_grace_seconds,
    );
    for event in calculator.role_events().iter() {
        stats.apply_role_event(event);
    }
    for event in calculator.first_man_change_events() {
        stats.apply_first_man_change_event(event);
    }
    stats
}

pub(crate) trait RushTestProjection {
    fn stats(&self) -> &RushStats;
}

impl RushTestProjection for RushCalculator {
    fn stats(&self) -> &RushStats {
        let mut stats = RushStatsAccumulator::default();
        for event in self.events() {
            stats.apply_event(event);
        }
        leak_test_stats(stats.stats().clone())
    }
}

pub(crate) trait SpeedFlipTestProjection {
    fn player_stats(&self) -> &HashMap<PlayerId, SpeedFlipStats>;
}

impl SpeedFlipTestProjection for SpeedFlipCalculator {
    fn player_stats(&self) -> &HashMap<PlayerId, SpeedFlipStats> {
        let mut stats = SpeedFlipStatsAccumulator::default();
        for event in self.events() {
            stats.apply_event(event);
        }
        leak_test_stats(stats.player_stats().clone())
    }
}

pub(crate) trait TerritorialPressureTestProjection {
    fn stats(&self) -> &TerritorialPressureStats;
}

impl TerritorialPressureTestProjection for TerritorialPressureCalculator {
    fn stats(&self) -> &TerritorialPressureStats {
        let mut stats = TerritorialPressureStatsAccumulator::default();
        for event in self.projected_events() {
            stats.apply_event(&event);
        }
        leak_test_stats(stats.stats().clone())
    }
}

pub(crate) trait TouchTestProjection {
    fn player_stats(&self) -> &HashMap<PlayerId, TouchStats>;
}

impl TouchTestProjection for TouchCalculator {
    fn player_stats(&self) -> &HashMap<PlayerId, TouchStats> {
        let mut stats = TouchStatsAccumulator::default();
        for event in self.events() {
            let frame = stats_test_frame(event.sample_time, event.sample_frame);
            stats.apply_touch_event(event, &frame);
        }
        leak_test_stats(stats.player_stats().clone())
    }
}

pub(crate) trait WallAerialTestProjection {
    fn player_stats(&self) -> &HashMap<PlayerId, WallAerialStats>;
}

impl WallAerialTestProjection for WallAerialCalculator {
    fn player_stats(&self) -> &HashMap<PlayerId, WallAerialStats> {
        let mut stats = WallAerialStatsAccumulator::default();
        for event in self.events() {
            let frame = stats_test_frame(event.sample_time, event.sample_frame);
            stats.apply_event(event, &frame);
        }
        leak_test_stats(stats.player_stats().clone())
    }
}

pub(crate) trait WallAerialShotTestProjection {
    fn player_stats(&self) -> &HashMap<PlayerId, WallAerialShotStats>;
}

impl WallAerialShotTestProjection for WallAerialShotCalculator {
    fn player_stats(&self) -> &HashMap<PlayerId, WallAerialShotStats> {
        let mut stats = WallAerialShotStatsAccumulator::default();
        for event in self.events() {
            let frame = stats_test_frame(event.time, event.frame);
            stats.apply_event(event, &frame);
        }
        leak_test_stats(stats.player_stats().clone())
    }
}

pub(crate) trait WavedashTestProjection {
    fn player_stats(&self) -> &HashMap<PlayerId, WavedashStats>;
}

impl WavedashTestProjection for WavedashCalculator {
    fn player_stats(&self) -> &HashMap<PlayerId, WavedashStats> {
        let mut stats = WavedashStatsAccumulator::default();
        for event in self.events() {
            stats.apply_event(event);
        }
        leak_test_stats(stats.player_stats().clone())
    }
}

pub(crate) trait WhiffTestProjection {
    fn player_stats(&self) -> &HashMap<PlayerId, WhiffStats>;
}

impl WhiffTestProjection for WhiffCalculator {
    fn player_stats(&self) -> &HashMap<PlayerId, WhiffStats> {
        let mut stats = WhiffStatsAccumulator::default();
        for event in self.events() {
            let frame = stats_test_frame(event.time, event.frame);
            stats.apply_event(event, &frame);
        }
        leak_test_stats(stats.player_stats().clone())
    }
}
