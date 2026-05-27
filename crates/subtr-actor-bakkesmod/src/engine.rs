use super::*;

pub struct SaEngine {
    pub(super) graph: AnalysisGraph,
    pub(super) live_events: SaLiveEventGenerator,
    pub(super) live_event_history: SaLiveEventHistory,
    pub(super) live_replay_meta_initialized: bool,
    pub(super) live_replay_meta: Option<ReplayMeta>,
    pub(super) live_replay_meta_signature: Vec<(RemoteId, bool, Option<String>)>,
    pub(super) emitted_mechanic_ids: HashSet<String>,
    pub(super) emitted_team_event_ids: HashSet<String>,
    pub(super) emitted_goal_context_ids: HashSet<String>,
    pub(super) graph_info_json: Vec<u8>,
    pub(super) timeline_frames: Vec<ReplayStatsFrame>,
    pub(super) pending_events: Vec<SaMechanicEvent>,
    pub(super) pending_team_events: Vec<SaTeamEvent>,
    pub(super) pending_goal_context_events: Vec<SaGoalContextEvent>,
}

pub struct SaReplayAnnotations {
    pub(super) events: Vec<SaMechanicEvent>,
    pub(super) cursor: usize,
    pub(super) last_poll_time: f32,
    pub(super) initialized: bool,
}

impl Default for SaEngine {
    fn default() -> Self {
        let mut graph = live_analysis_graph();
        let graph_info_json = serialize_graph_info(&mut graph);
        Self {
            graph,
            live_events: SaLiveEventGenerator::default(),
            live_event_history: SaLiveEventHistory::default(),
            live_replay_meta_initialized: false,
            live_replay_meta: None,
            live_replay_meta_signature: Vec::new(),
            emitted_mechanic_ids: HashSet::new(),
            emitted_team_event_ids: HashSet::new(),
            emitted_goal_context_ids: HashSet::new(),
            graph_info_json,
            timeline_frames: Vec::new(),
            pending_events: Vec::new(),
            pending_team_events: Vec::new(),
            pending_goal_context_events: Vec::new(),
        }
    }
}
