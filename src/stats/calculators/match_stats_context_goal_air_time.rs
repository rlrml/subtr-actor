use super::*;

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize, ts_rs::TS)]
#[ts(export)]
pub struct GoalBallAirTimeStats {
    pub goal_ball_air_time_sample_count: u32,
    pub cumulative_goal_ball_air_time: f32,
    pub last_goal_ball_air_time: Option<f32>,
    #[serde(default, skip_serializing)]
    pub(in crate::stats::calculators::match_stats) goal_ball_air_times: Vec<f32>,
}

impl GoalBallAirTimeStats {
    pub fn goal_ball_air_times(&self) -> &[f32] {
        &self.goal_ball_air_times
    }

    pub fn record_goal(&mut self, ball_air_time: f32) {
        let clamped_time = ball_air_time.max(0.0);
        self.goal_ball_air_time_sample_count += 1;
        self.cumulative_goal_ball_air_time += clamped_time;
        self.last_goal_ball_air_time = Some(clamped_time);
        self.goal_ball_air_times.push(clamped_time);
        self.goal_ball_air_times
            .sort_by(|left, right| left.total_cmp(right));
    }

    pub fn average_goal_ball_air_time(&self) -> f32 {
        if self.goal_ball_air_time_sample_count == 0 {
            0.0
        } else {
            self.cumulative_goal_ball_air_time / self.goal_ball_air_time_sample_count as f32
        }
    }

    pub fn median_goal_ball_air_time(&self) -> f32 {
        if self.goal_ball_air_times.is_empty() {
            return 0.0;
        }

        let mut sorted_times = self.goal_ball_air_times.clone();
        sorted_times.sort_by(|a, b| a.total_cmp(b));
        let midpoint = sorted_times.len() / 2;
        if sorted_times.len().is_multiple_of(2) {
            (sorted_times[midpoint - 1] + sorted_times[midpoint]) * 0.5
        } else {
            sorted_times[midpoint]
        }
    }

    pub(in crate::stats::calculators::match_stats) fn merge(&mut self, other: &Self) {
        self.goal_ball_air_time_sample_count += other.goal_ball_air_time_sample_count;
        self.cumulative_goal_ball_air_time += other.cumulative_goal_ball_air_time;
        self.last_goal_ball_air_time = other
            .last_goal_ball_air_time
            .or(self.last_goal_ball_air_time);
        self.goal_ball_air_times
            .extend(other.goal_ball_air_times.iter().copied());
    }
}
