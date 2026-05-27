#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub(crate) struct FirstManStintState {
    pub active: bool,
    pub current_first_man_time: f32,
    pub non_first_man_seconds: f32,
}
