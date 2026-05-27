use super::*;

impl BallCarryCalculator {
    fn event_from_sequence(
        sequence: CompletedBallControlSequence<BallCarryKind>,
    ) -> BallCarryEvent {
        let air_dribble_origin = (sequence.kind == BallCarryKind::AirDribble)
            .then(|| AirDribblePolicy::origin(sequence.start_position));
        BallCarryEvent {
            player_id: sequence.player_id,
            is_team_0: sequence.is_team_0,
            kind: sequence.kind,
            start_frame: sequence.start_frame,
            end_frame: sequence.end_frame,
            start_time: sequence.start_time,
            end_time: sequence.end_time,
            duration: sequence.duration,
            straight_line_distance: sequence.straight_line_distance,
            path_distance: sequence.path_distance,
            average_horizontal_gap: sequence.average_horizontal_gap,
            average_vertical_gap: sequence.average_vertical_gap,
            average_speed: sequence.average_speed,
            touch_count: sequence.touch_count,
            air_touch_count: sequence.air_touch_count,
            air_dribble_origin,
        }
    }

    fn record_carry_event(&mut self, event: BallCarryEvent) {
        match event.kind {
            BallCarryKind::Carry => self.record_ground_carry_event(&event),
            BallCarryKind::AirDribble => self.record_air_dribble_event(&event),
        }
        self.carry_events.push(event);
    }

    fn record_ground_carry_event(&mut self, event: &BallCarryEvent) {
        let player_stats = self
            .player_stats
            .entry(event.player_id.clone())
            .or_default();
        Self::apply_carry_event(player_stats, event);

        let team_stats = if event.is_team_0 {
            &mut self.team_zero_stats
        } else {
            &mut self.team_one_stats
        };
        Self::apply_carry_event(team_stats, event);
    }

    fn record_air_dribble_event(&mut self, event: &BallCarryEvent) {
        let player_stats = self
            .player_air_dribble_stats
            .entry(event.player_id.clone())
            .or_default();
        AirDribblePolicy::apply_event(player_stats, event);

        let team_stats = if event.is_team_0 {
            &mut self.team_zero_air_dribble_stats
        } else {
            &mut self.team_one_air_dribble_stats
        };
        AirDribblePolicy::apply_event(team_stats, event);
    }

    fn apply_carry_event(stats: &mut BallCarryStats, event: &BallCarryEvent) {
        stats.record_event(event);
        stats.total_carry_time += event.duration;
        stats.total_straight_line_distance += event.straight_line_distance;
        stats.total_path_distance += event.path_distance;
        stats.longest_carry_time = stats.longest_carry_time.max(event.duration);
        stats.furthest_carry_distance = stats
            .furthest_carry_distance
            .max(event.straight_line_distance);
        stats.fastest_carry_speed = stats.fastest_carry_speed.max(event.average_speed);
        stats.carry_speed_sum += event.average_speed;
        stats.average_horizontal_gap_sum += event.average_horizontal_gap;
        stats.average_vertical_gap_sum += event.average_vertical_gap;
    }

    pub fn update(&mut self, control_state: &ContinuousBallControlState) -> SubtrActorResult<()> {
        for sequence in control_state
            .completed_sequences
            .iter()
            .skip(self.processed_control_sequence_count)
            .cloned()
        {
            if AirDribblePolicy::is_valid_sequence(&sequence) {
                self.record_carry_event(Self::event_from_sequence(sequence));
            }
        }
        self.processed_control_sequence_count = control_state.completed_sequences.len();
        Ok(())
    }
}
