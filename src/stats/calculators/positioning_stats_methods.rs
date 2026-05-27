use super::*;

fn safe_ratio(numerator: f32, denominator: f32) -> f32 {
    if denominator == 0.0 {
        0.0
    } else {
        numerator / denominator
    }
}

macro_rules! pct_method {
    ($name:ident, $field:ident) => {
        pub fn $name(&self) -> f32 {
            self.pct(self.$field)
        }
    };
}

impl PositioningStats {
    pub fn average_distance_to_teammates(&self) -> f32 {
        safe_ratio(self.sum_distance_to_teammates, self.tracked_time)
    }

    pub fn average_distance_to_ball(&self) -> f32 {
        safe_ratio(self.sum_distance_to_ball, self.tracked_time)
    }

    pub fn average_distance_to_ball_has_possession(&self) -> f32 {
        safe_ratio(
            self.sum_distance_to_ball_has_possession,
            self.time_has_possession,
        )
    }

    pub fn average_distance_to_ball_no_possession(&self) -> f32 {
        safe_ratio(
            self.sum_distance_to_ball_no_possession,
            self.time_no_possession,
        )
    }

    fn pct(&self, value: f32) -> f32 {
        if self.tracked_time == 0.0 {
            0.0
        } else {
            value * 100.0 / self.tracked_time
        }
    }

    pct_method!(most_back_pct, time_most_back);
    pct_method!(most_forward_pct, time_most_forward);
    pct_method!(mid_role_pct, time_mid_role);
    pct_method!(other_role_pct, time_other_role);
    pct_method!(defensive_third_pct, time_defensive_zone);
    pct_method!(neutral_third_pct, time_neutral_zone);
    pct_method!(offensive_third_pct, time_offensive_zone);

    pub fn defensive_zone_pct(&self) -> f32 {
        self.defensive_third_pct()
    }

    pub fn neutral_zone_pct(&self) -> f32 {
        self.neutral_third_pct()
    }

    pub fn offensive_zone_pct(&self) -> f32 {
        self.offensive_third_pct()
    }

    pct_method!(defensive_half_pct, time_defensive_half);
    pct_method!(offensive_half_pct, time_offensive_half);
    pct_method!(closest_to_ball_pct, time_closest_to_ball);
    pct_method!(farthest_from_ball_pct, time_farthest_from_ball);
    pct_method!(behind_ball_pct, time_behind_ball);
    pct_method!(level_with_ball_pct, time_level_with_ball);
    pct_method!(in_front_of_ball_pct, time_in_front_of_ball);
}
