use super::*;

const TOUCH_KIND_LABEL_VALUES: [&str; 3] = ["control", "medium_hit", "hard_hit"];
const TOUCH_SURFACE_LABEL_VALUES: [&str; 3] = ["ground", "air", "wall"];
const TOUCH_DODGE_STATE_LABEL_VALUES: [&str; 2] = ["no_dodge", "dodge"];
const TOUCH_INTENTION_LABEL_VALUES: [&str; 7] = [
    "control",
    "shot",
    "save",
    "challenge",
    "clear",
    "pass",
    "neutral",
];
const TOUCH_RECEPTION_LABEL_VALUES: [&str; 2] = ["first_touch", "continuation"];

fn touch_kind_label(value: &str) -> StatLabel {
    match value {
        "medium_hit" => StatLabel::new("kind", "medium_hit"),
        "hard_hit" => StatLabel::new("kind", "hard_hit"),
        _ => StatLabel::new("kind", "control"),
    }
}

fn touch_height_band_label(value: &str) -> StatLabel {
    match value {
        "low_air" => StatLabel::new("height_band", "low_air"),
        "high_air" => StatLabel::new("height_band", "high_air"),
        _ => StatLabel::new("height_band", "ground"),
    }
}

fn touch_surface_label(value: &str) -> StatLabel {
    match value {
        "air" => StatLabel::new("surface", "air"),
        "wall" => StatLabel::new("surface", "wall"),
        _ => StatLabel::new("surface", "ground"),
    }
}

fn touch_dodge_state_label(value: &str) -> StatLabel {
    match value {
        "dodge" => StatLabel::new("dodge_state", "dodge"),
        _ => StatLabel::new("dodge_state", "no_dodge"),
    }
}

fn touch_intention_label(value: &str) -> StatLabel {
    match value {
        "control" => StatLabel::new("intention", "control"),
        "shot" => StatLabel::new("intention", "shot"),
        "save" => StatLabel::new("intention", "save"),
        "challenge" => StatLabel::new("intention", "challenge"),
        "clear" => StatLabel::new("intention", "clear"),
        "pass" => StatLabel::new("intention", "pass"),
        _ => StatLabel::new("intention", "neutral"),
    }
}

fn touch_reception_label(first_touch: bool) -> StatLabel {
    StatLabel::new(
        "reception",
        if first_touch {
            "first_touch"
        } else {
            "continuation"
        },
    )
}

/// Accumulated touch stats: counts by control, hardness, and aerial context.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize, ts_rs::TS)]
#[ts(export)]
pub struct TouchStats {
    pub touch_count: u32,
    pub control_touch_count: u32,
    pub medium_hit_count: u32,
    pub hard_hit_count: u32,
    pub aerial_touch_count: u32,
    pub high_aerial_touch_count: u32,
    #[serde(default)]
    pub wall_touch_count: u32,
    #[serde(default)]
    pub first_touch_count: u32,
    pub is_last_touch: bool,
    pub last_touch_time: Option<f32>,
    pub last_touch_frame: Option<usize>,
    pub time_since_last_touch: Option<f32>,
    pub frames_since_last_touch: Option<usize>,
    pub last_ball_speed_change: Option<f32>,
    pub max_ball_speed_change: f32,
    pub cumulative_ball_speed_change: f32,
    #[serde(default)]
    pub total_ball_travel_distance: f32,
    #[serde(default)]
    pub total_ball_advance_distance: f32,
    #[serde(default)]
    pub total_ball_retreat_distance: f32,
    #[serde(default, skip_serializing_if = "LabeledCounts::is_empty")]
    pub labeled_touch_counts: LabeledCounts,
    #[serde(default, skip_serializing_if = "LabeledCounts::is_empty")]
    pub labeled_intention_counts: LabeledCounts,
    #[serde(default, skip_serializing_if = "LabeledCounts::is_empty")]
    pub touch_counts_by_role: LabeledCounts,
    #[serde(default, skip_serializing_if = "LabeledCounts::is_empty")]
    pub touch_counts_by_play_depth: LabeledCounts,
}

impl TouchStats {
    pub fn average_ball_speed_change(&self) -> f32 {
        if self.touch_count == 0 {
            0.0
        } else {
            self.cumulative_ball_speed_change / self.touch_count as f32
        }
    }

    pub fn touch_count_with_labels(&self, labels: &[StatLabel]) -> u32 {
        self.labeled_touch_counts.count_matching(labels)
    }

    pub fn dodge_touch_count(&self) -> u32 {
        self.touch_count_with_labels(&[StatLabel::new("dodge_state", "dodge")])
    }

    pub fn dodge_hit_count(&self) -> u32 {
        self.touch_count_with_labels(&[
            StatLabel::new("dodge_state", "dodge"),
            StatLabel::new("kind", "medium_hit"),
        ]) + self.touch_count_with_labels(&[
            StatLabel::new("dodge_state", "dodge"),
            StatLabel::new("kind", "hard_hit"),
        ])
    }

    pub fn intention_count(&self, intention: &str) -> u32 {
        self.labeled_intention_counts
            .count_matching(&[touch_intention_label(intention)])
    }

    pub fn first_touch_intention_count(&self, intention: &str) -> u32 {
        self.labeled_intention_counts.count_matching(&[
            touch_intention_label(intention),
            touch_reception_label(true),
        ])
    }

    pub fn complete_labeled_intention_counts(&self) -> LabeledCounts {
        let mut entries: Vec<_> = TOUCH_INTENTION_LABEL_VALUES
            .into_iter()
            .flat_map(|intention| {
                TOUCH_RECEPTION_LABEL_VALUES
                    .into_iter()
                    .map(move |reception| {
                        let mut labels = vec![
                            StatLabel::new("intention", intention),
                            StatLabel::new("reception", reception),
                        ];
                        labels.sort();
                        LabeledCountEntry {
                            count: self.labeled_intention_counts.count_exact(&labels),
                            labels,
                        }
                    })
            })
            .collect();

        entries.sort_by(|left, right| left.labels.cmp(&right.labels));

        LabeledCounts { entries }
    }

    pub fn touch_count_with_role(&self, role: RoleState) -> u32 {
        self.touch_counts_by_role.count_exact(&[role.as_label()])
    }

    pub fn touch_count_with_play_depth(&self, play_depth: PlayDepthState) -> u32 {
        self.touch_counts_by_play_depth
            .count_exact(&[play_depth.as_label()])
    }

    pub fn touches_as_first_man(&self) -> u32 {
        self.touch_count_with_role(RoleState::FirstMan)
    }

    pub fn touches_as_second_man(&self) -> u32 {
        self.touch_count_with_role(RoleState::SecondMan)
    }

    pub fn touches_as_third_man(&self) -> u32 {
        self.touch_count_with_role(RoleState::ThirdMan)
    }

    pub fn touches_behind_play(&self) -> u32 {
        self.touch_count_with_play_depth(PlayDepthState::BehindPlay)
    }

    pub fn touches_ahead_of_play(&self) -> u32 {
        self.touch_count_with_play_depth(PlayDepthState::AheadOfPlay)
    }

    pub fn complete_touch_counts_by_role(&self) -> LabeledCounts {
        let mut entries: Vec<_> = ALL_ROLE_STATES
            .into_iter()
            .map(|role| {
                let labels = vec![role.as_label()];
                LabeledCountEntry {
                    count: self.touch_counts_by_role.count_exact(&labels),
                    labels,
                }
            })
            .collect();
        entries.sort_by(|left, right| left.labels.cmp(&right.labels));
        LabeledCounts { entries }
    }

    pub fn complete_touch_counts_by_play_depth(&self) -> LabeledCounts {
        let mut entries: Vec<_> = ALL_PLAY_DEPTH_STATES
            .into_iter()
            .map(|play_depth| {
                let labels = vec![play_depth.as_label()];
                LabeledCountEntry {
                    count: self.touch_counts_by_play_depth.count_exact(&labels),
                    labels,
                }
            })
            .collect();
        entries.sort_by(|left, right| left.labels.cmp(&right.labels));
        LabeledCounts { entries }
    }

    pub fn complete_labeled_touch_counts(&self) -> LabeledCounts {
        let mut entries: Vec<_> = ALL_PLAYER_VERTICAL_BANDS
            .into_iter()
            .flat_map(|height_band| {
                TOUCH_SURFACE_LABEL_VALUES
                    .into_iter()
                    .flat_map(move |surface| {
                        TOUCH_DODGE_STATE_LABEL_VALUES
                            .into_iter()
                            .flat_map(move |dodge_state| {
                                TOUCH_KIND_LABEL_VALUES.into_iter().map(move |kind| {
                                    let mut labels = vec![
                                        StatLabel::new("kind", kind),
                                        height_band.as_label(),
                                        StatLabel::new("surface", surface),
                                        StatLabel::new("dodge_state", dodge_state),
                                    ];
                                    labels.sort();
                                    LabeledCountEntry {
                                        count: self.labeled_touch_counts.count_exact(&labels),
                                        labels,
                                    }
                                })
                            })
                    })
            })
            .collect();

        entries.sort_by(|left, right| left.labels.cmp(&right.labels));

        LabeledCounts { entries }
    }

    pub fn with_complete_labeled_touch_counts(mut self) -> Self {
        self.labeled_touch_counts = self.complete_labeled_touch_counts();
        self.labeled_intention_counts = self.complete_labeled_intention_counts();
        self
    }
}

/// Accumulates touch stats over the replay from touch events.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct TouchStatsAccumulator {
    player_stats: HashMap<PlayerId, TouchStats>,
    current_last_touch_player: Option<PlayerId>,
}

impl TouchStatsAccumulator {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn player_stats(&self) -> &HashMap<PlayerId, TouchStats> {
        &self.player_stats
    }

    pub fn begin_sample(&mut self, frame: &FrameInfo) {
        for stats in self.player_stats.values_mut() {
            stats.is_last_touch = false;
            stats.time_since_last_touch = stats
                .last_touch_time
                .map(|time| (frame.time - time).max(0.0));
            stats.frames_since_last_touch = stats
                .last_touch_frame
                .map(|last_frame| frame.frame_number.saturating_sub(last_frame));
        }
    }

    pub fn apply_touch_event(&mut self, event: &TouchClassificationEvent, frame: &FrameInfo) {
        let stats = self.player_stats.entry(event.player.clone()).or_default();
        stats.touch_count += 1;
        match event.height_band.as_str() {
            "low_air" => stats.aerial_touch_count += 1,
            "high_air" => {
                stats.aerial_touch_count += 1;
                stats.high_aerial_touch_count += 1;
            }
            _ => {}
        }
        match event.kind.as_str() {
            "control" => stats.control_touch_count += 1,
            "medium_hit" => stats.medium_hit_count += 1,
            "hard_hit" => stats.hard_hit_count += 1,
            _ => {}
        }
        if event.surface == "wall" {
            stats.wall_touch_count += 1;
        }
        stats.labeled_touch_counts.increment([
            touch_kind_label(&event.kind),
            touch_height_band_label(&event.height_band),
            touch_surface_label(&event.surface),
            touch_dodge_state_label(&event.dodge_state),
        ]);
        if event.first_touch {
            stats.first_touch_count += 1;
        }
        stats.labeled_intention_counts.increment([
            touch_intention_label(&event.intention),
            touch_reception_label(event.first_touch),
        ]);
        stats
            .touch_counts_by_role
            .increment([event.role.as_label()]);
        stats
            .touch_counts_by_play_depth
            .increment([event.play_depth.as_label()]);
        stats.last_touch_time = Some(event.time);
        stats.last_touch_frame = Some(event.frame);
        stats.time_since_last_touch = Some((frame.time - event.time).max(0.0));
        stats.frames_since_last_touch = Some(frame.frame_number.saturating_sub(event.frame));
        stats.last_ball_speed_change = Some(event.ball_speed_change);
        stats.max_ball_speed_change = stats.max_ball_speed_change.max(event.ball_speed_change);
        stats.cumulative_ball_speed_change += event.ball_speed_change;
        if let Some(movement) = event.ball_movement.as_ref() {
            stats.total_ball_travel_distance += movement.travel_distance;
            stats.total_ball_advance_distance += movement.advance_distance;
            stats.total_ball_retreat_distance += movement.retreat_distance;
        }
        self.current_last_touch_player = Some(event.player.clone());
    }

    pub fn set_current_last_touch_player(&mut self, player: Option<PlayerId>) {
        self.current_last_touch_player = player;
    }

    pub fn restore_current_last_touch_marker(&mut self) {
        if let Some(player_id) = self.current_last_touch_player.as_ref() {
            if let Some(stats) = self.player_stats.get_mut(player_id) {
                stats.is_last_touch = true;
            }
        }
    }
}
