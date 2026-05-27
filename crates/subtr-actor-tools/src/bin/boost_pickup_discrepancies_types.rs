use clap::Parser;
use serde::Serialize;
use subtr_actor::{
    BoostPickupActivity, BoostPickupComparison, BoostPickupFieldHalf, BoostPickupPadType, PlayerId,
};

#[derive(Default, Serialize)]
pub(crate) struct PickupCountBreakdown {
    pub(crate) total: usize,
    pub(crate) both: usize,
    pub(crate) ghost: usize,
    pub(crate) missed: usize,
    pub(crate) big: usize,
    pub(crate) small: usize,
    pub(crate) ambiguous: usize,
    pub(crate) active: usize,
    pub(crate) inactive: usize,
    pub(crate) unknown_activity: usize,
}

#[derive(Serialize)]
pub(crate) struct SummaryRecord<'a> {
    pub(crate) record_type: &'static str,
    pub(crate) replay: &'a str,
    pub(crate) emitted: &'static str,
    pub(crate) all_events: PickupCountBreakdown,
    pub(crate) emitted_events: PickupCountBreakdown,
}

#[derive(Serialize)]
pub(crate) struct PickupRecord<'a> {
    pub(crate) record_type: &'static str,
    pub(crate) replay: &'a str,
    pub(crate) comparison: BoostPickupComparison,
    pub(crate) frame: usize,
    pub(crate) time: f32,
    pub(crate) player_id: &'a PlayerId,
    pub(crate) player: String,
    pub(crate) team: &'static str,
    pub(crate) pad_type: BoostPickupPadType,
    pub(crate) field_half: BoostPickupFieldHalf,
    pub(crate) activity: BoostPickupActivity,
    pub(crate) reported_frame: Option<usize>,
    pub(crate) reported_time: Option<f32>,
    pub(crate) inferred_frame: Option<usize>,
    pub(crate) inferred_time: Option<f32>,
    pub(crate) boost_before: Option<f32>,
    pub(crate) boost_after: Option<f32>,
}

#[derive(Debug, Parser)]
#[command(about = "Print boost-pickup discrepancy events as JSONL.")]
pub(crate) struct Args {
    /// Include all pickup comparison events, not just discrepancies.
    #[arg(long = "all")]
    pub(crate) include_all: bool,

    /// Replay path or fixture name to inspect.
    #[arg(value_name = "replay-path-or-fixture-name", num_args = 1..)]
    pub(crate) replay_args: Vec<String>,
}
