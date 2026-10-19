//! Per-frame replay telemetry recorded for the whiff-learning workflow.
//!
//! [`WhiffTelemetryNode`] records the exact inputs the whiff calculator
//! consumes (frame clock, live-play flag, ball and player rigid bodies,
//! boost/dodge/jump state, and the oriented-hitbox ball contact gap). Values are
//! rounded once here, so the offline dump, the Python trainer, and
//! [`super::features`] all compute features from identical numbers.

use serde::Serialize;
use serde_json::{Value, json};
use subtr_actor::stats::analysis_graph::{
    AnalysisDependency, AnalysisNode, AnalysisStateContext, collect_analysis_graph_for_replay,
    graph_with_builtin_analysis_nodes,
};
use subtr_actor::{
    BallFrameState, BeatenToBallCalculator, FrameInfo, LivePlayState, PlayerControlState,
    PlayerFrameState, PlayerId, SubtrActorResult, TouchState, WhiffCalculator,
    car_hitbox_ball_contact_gap, quat_to_glam, vec_to_glam,
};

#[derive(Debug, Default, Clone, Serialize)]
pub struct PlayerTrack {
    pub id: String,
    pub is_team_0: bool,
    pub hitbox: Option<Value>,
    /// Per-frame `[x, y, z]` or null.
    pub pos: Vec<Option<[f32; 3]>>,
    pub vel: Vec<Option<[f32; 3]>>,
    pub ang_vel: Vec<Option<[f32; 3]>>,
    /// Per-frame `[x, y, z, w]` rotation quaternion or null.
    pub quat: Vec<Option<[f32; 4]>>,
    pub boost: Vec<Option<f32>>,
    pub boost_active: Vec<bool>,
    pub dodge_active: Vec<bool>,
    pub double_jump_active: Vec<bool>,
    pub powerslide_active: Vec<bool>,
    /// Ball-surface to oriented-hitbox contact gap (uu, clamped at 0).
    pub contact_gap: Vec<Option<f32>>,
    /// Ball center in car-local coordinates.
    pub local_ball: Vec<Option<[f32; 3]>>,
}

#[derive(Debug, Default, Clone, Serialize)]
pub struct FrameTracks {
    pub frame: Vec<usize>,
    pub time: Vec<f32>,
    pub dt: Vec<f32>,
    pub live: Vec<bool>,
    pub seconds_remaining: Vec<Option<i32>>,
    pub ball_pos: Vec<Option<[f32; 3]>>,
    pub ball_vel: Vec<Option<[f32; 3]>>,
    pub ball_ang_vel: Vec<Option<[f32; 3]>>,
}

#[derive(Debug, Default, Clone)]
pub struct WhiffTelemetry {
    pub frames: FrameTracks,
    pub players: Vec<PlayerTrack>,
    pub player_ids: Vec<PlayerId>,
    pub touches: Vec<Value>,
}

/// Records the whiff calculator's per-frame inputs.
#[derive(Default)]
pub struct WhiffTelemetryNode {
    telemetry: WhiffTelemetry,
}

fn round1(value: f32) -> f32 {
    (value * 10.0).round() / 10.0
}

fn round4(value: f32) -> f32 {
    (value * 10_000.0).round() / 10_000.0
}

fn vec3(v: glam::Vec3) -> [f32; 3] {
    [round1(v.x), round1(v.y), round1(v.z)]
}

impl WhiffTelemetry {
    fn player_index(&mut self, player_id: &PlayerId, is_team_0: bool) -> usize {
        if let Some(index) = self.player_ids.iter().position(|id| id == player_id) {
            return index;
        }
        let frames_so_far = self.frames.frame.len().saturating_sub(1);
        let track = PlayerTrack {
            id: player_id_string(player_id),
            is_team_0,
            pos: vec![None; frames_so_far],
            vel: vec![None; frames_so_far],
            ang_vel: vec![None; frames_so_far],
            quat: vec![None; frames_so_far],
            boost: vec![None; frames_so_far],
            boost_active: vec![false; frames_so_far],
            dodge_active: vec![false; frames_so_far],
            double_jump_active: vec![false; frames_so_far],
            powerslide_active: vec![false; frames_so_far],
            contact_gap: vec![None; frames_so_far],
            local_ball: vec![None; frames_so_far],
            ..PlayerTrack::default()
        };
        self.players.push(track);
        self.player_ids.push(player_id.clone());
        self.players.len() - 1
    }
}

impl AnalysisNode for WhiffTelemetryNode {
    type State = WhiffTelemetry;

    fn name(&self) -> &'static str {
        "whiff_telemetry_dump"
    }

    fn dependencies(&self) -> Vec<AnalysisDependency> {
        vec![
            AnalysisDependency::required::<FrameInfo>(),
            AnalysisDependency::required::<BallFrameState>(),
            AnalysisDependency::required::<PlayerFrameState>(),
            AnalysisDependency::required::<PlayerControlState>(),
            AnalysisDependency::required::<TouchState>(),
            AnalysisDependency::required::<LivePlayState>(),
            AnalysisDependency::required::<WhiffCalculator>(),
            AnalysisDependency::required::<BeatenToBallCalculator>(),
        ]
    }

    fn evaluate(&mut self, ctx: &AnalysisStateContext<'_>) -> SubtrActorResult<()> {
        let frame = ctx.get::<FrameInfo>()?;
        let ball = ctx.get::<BallFrameState>()?;
        let players = ctx.get::<PlayerFrameState>()?;
        let controls = ctx.get::<PlayerControlState>()?;
        let touch_state = ctx.get::<TouchState>()?;
        let live = ctx.get::<LivePlayState>()?;
        let telemetry = &mut self.telemetry;

        let tracks = &mut telemetry.frames;
        tracks.frame.push(frame.frame_number);
        tracks.time.push(round4(frame.time));
        tracks.dt.push(round4(frame.dt));
        tracks.live.push(live.is_live_play);
        tracks.seconds_remaining.push(frame.seconds_remaining);
        let ball_sample = ball.sample();
        tracks
            .ball_pos
            .push(ball_sample.map(|ball| vec3(ball.position())));
        tracks
            .ball_vel
            .push(ball_sample.map(|ball| vec3(ball.velocity())));
        tracks.ball_ang_vel.push(ball_sample.and_then(|ball| {
            ball.rigid_body
                .angular_velocity
                .as_ref()
                .map(|v| vec3(vec_to_glam(v)))
        }));
        let ball_position = ball.position();
        let frame_count = tracks.frame.len();

        for player in &players.players {
            let index = telemetry.player_index(&player.player_id, player.is_team_0);
            let control = controls.sample(&player.player_id);
            let track = &mut telemetry.players[index];
            if track.hitbox.is_none() {
                let hitbox = player.hitbox;
                track.hitbox = Some(json!({
                    "family": format!("{:?}", hitbox.family),
                    "length": hitbox.length,
                    "width": hitbox.width,
                    "height": hitbox.height,
                    "angle": hitbox.angle,
                    "offset": hitbox.offset,
                    "elevation": hitbox.elevation,
                }));
            }
            let body = player.rigid_body.as_ref();
            track.pos.push(body.map(|b| vec3(vec_to_glam(&b.location))));
            track.vel.push(
                body.and_then(|b| b.linear_velocity.as_ref())
                    .map(|v| vec3(vec_to_glam(v))),
            );
            track.ang_vel.push(
                body.and_then(|b| b.angular_velocity.as_ref())
                    .map(|v| vec3(vec_to_glam(v))),
            );
            track.quat.push(body.map(|b| {
                let q = quat_to_glam(&b.rotation);
                [round4(q.x), round4(q.y), round4(q.z), round4(q.w)]
            }));
            track.boost.push(player.boost_amount.map(round1));
            track.boost_active.push(player.boost_active);
            track.dodge_active.push(player.dodge_active);
            track.double_jump_active.push(control.double_jump_active);
            track.powerslide_active.push(player.powerslide_active);
            let (gap, local) = match (body, ball_position) {
                (Some(body), Some(ball_position)) => {
                    let gap = car_hitbox_ball_contact_gap(ball_position, body, player.hitbox);
                    let local = quat_to_glam(&body.rotation).inverse()
                        * (ball_position - vec_to_glam(&body.location));
                    (gap.map(round1), Some(vec3(local)))
                }
                _ => (None, None),
            };
            track.contact_gap.push(gap);
            track.local_ball.push(local);
        }
        for track in &mut telemetry.players {
            if track.pos.len() < frame_count {
                track.pos.push(None);
                track.vel.push(None);
                track.ang_vel.push(None);
                track.quat.push(None);
                track.boost.push(None);
                track.boost_active.push(false);
                track.dodge_active.push(false);
                track.double_jump_active.push(false);
                track.powerslide_active.push(false);
                track.contact_gap.push(None);
                track.local_ball.push(None);
            }
        }

        for touch in &touch_state.touch_events {
            let mut value = serde_json::to_value(touch).unwrap_or(Value::Null);
            if let (Value::Object(map), Some(player)) = (&mut value, touch.player.as_ref()) {
                map.insert("player_id".to_owned(), json!(player_id_string(player)));
                map.insert("frame_index".to_owned(), json!(frame_count - 1));
            }
            telemetry.touches.push(value);
        }
        Ok(())
    }

    fn state(&self) -> &Self::State {
        &self.telemetry
    }
}

pub fn player_id_string(player_id: &PlayerId) -> String {
    match serde_json::to_value(player_id) {
        Ok(Value::Object(map)) if map.len() == 1 => {
            let (kind, value) = map.into_iter().next().expect("map has one value");
            let id = match value {
                Value::String(text) => text,
                Value::Object(fields) => fields
                    .get("online_id")
                    .or_else(|| fields.get("id"))
                    .map(|v| v.to_string().trim_matches('"').to_owned())
                    .unwrap_or_else(|| Value::Object(fields).to_string()),
                other => other.to_string(),
            };
            format!("{kind}:{id}")
        }
        Ok(value) => value.to_string(),
        Err(_) => format!("{player_id:?}"),
    }
}

pub fn events_with_player_ids<T: Serialize>(
    events: &[T],
    player: impl Fn(&T) -> &PlayerId,
) -> Vec<Value> {
    events
        .iter()
        .map(|event| {
            let mut value = serde_json::to_value(event).unwrap_or(Value::Null);
            if let Value::Object(map) = &mut value {
                map.insert(
                    "player_id".to_owned(),
                    json!(player_id_string(player(event))),
                );
            }
            value
        })
        .collect()
}

/// Telemetry plus the current detectors' events for one replay.
pub struct ReplayWhiffTelemetry {
    pub telemetry: WhiffTelemetry,
    pub whiff_events: Vec<Value>,
    pub beaten_to_ball_events: Vec<Value>,
}

pub fn collect_replay_telemetry(replay: &boxcars::Replay) -> anyhow::Result<ReplayWhiffTelemetry> {
    let mut graph = graph_with_builtin_analysis_nodes([
        "whiff",
        "beaten_to_ball",
        "touch",
        "player_control_state",
        "live_play",
    ])
    .map_err(|error| anyhow::anyhow!("building graph: {error:?}"))?;
    graph.push_node(WhiffTelemetryNode::default());
    let graph = collect_analysis_graph_for_replay(replay, graph)
        .map_err(|error| anyhow::anyhow!("processing replay: {error:?}"))?;
    let whiff_events = graph
        .state::<WhiffCalculator>()
        .map(|calculator| events_with_player_ids(calculator.events(), |event| &event.player))
        .unwrap_or_default();
    let beaten_to_ball_events = graph
        .state::<BeatenToBallCalculator>()
        .map(|calculator| events_with_player_ids(calculator.events(), |event| &event.player))
        .unwrap_or_default();
    let telemetry = graph
        .state::<WhiffTelemetry>()
        .cloned()
        .ok_or_else(|| anyhow::anyhow!("telemetry node missing after processing"))?;
    Ok(ReplayWhiffTelemetry {
        telemetry,
        whiff_events,
        beaten_to_ball_events,
    })
}
