pub(crate) const BALLCHASING_API_BASE_URL: &str = "https://ballchasing.com/api";
pub(crate) const DEFAULT_PLAYLIST: &str = "ranked-duels";
pub(crate) const DEFAULT_COUNT: usize = 10;
pub(crate) const DEFAULT_MIN_CONFIDENCE: f32 = 0.55;
pub(crate) const DEFAULT_BEFORE_SECONDS: f32 = 10.0;
pub(crate) const DEFAULT_AFTER_SECONDS: f32 = 3.5;
pub(crate) const DEFAULT_GOAL_LOOKAHEAD_SECONDS: f32 = 10.0;
pub(crate) const DEFAULT_GOAL_TAIL_SECONDS: f32 = 3.0;
pub(crate) const DEFAULT_MIN_CLIP_SECONDS: f32 = 8.0;
pub(crate) const DEFAULT_DOWNLOAD_DELAY_MS: u64 = 1100;
pub(crate) const DEFAULT_MECHANICS: &[&str] = &[
    "flick",
    "musty_flick",
    "one_timer",
    "air_dribble",
    "flip_reset",
    "ceiling_shot",
    "double_tap",
];
pub(crate) const ALL_MECHANICS: &[&str] = &[
    "flick",
    "musty_flick",
    "one_timer",
    "air_dribble",
    "flip_reset",
    "ceiling_shot",
    "double_tap",
    "speed_flip",
    "half_flip",
    "wavedash",
];
