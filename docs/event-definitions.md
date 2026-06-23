# Stat Definitions

Generated from static Rust metadata. Do not edit by hand.

## Events

### Backboard Bounce (`backboard_bounce`)

- Category: `mechanic`
- Confidence:
  - Approach: `unknown`
  - True positive evidence: `not_evaluated`
  - False positive evidence: `not_evaluated`
  - False negative evidence: `not_evaluated`
  - Testing: `untested`
- Producers:
  - `backboard_bounce_state` via `BackboardBounceStateNode` / `BackboardBounceCalculator`

**Summary**

A ball rebound off the opponent backboard attributed to the player who sent the ball there.

**Approach**

- Track the last touch during live play and attribute a later backboard rebound to that touch when it occurs within the configured attribution window.
- Require the ball to be high, near the backboard face, moving toward the backboard before the rebound, and moving away after the rebound.
- Ignore frames with a simultaneous touch so the rebound is not confused with a player-ball contact.

**Limitations**

_None documented._

**Known Issues**

_None documented._

### Ball Carry (`ball_carry`)

- Category: `mechanic`
- Confidence:
  - Approach: `unknown`
  - True positive evidence: `not_evaluated`
  - False positive evidence: `not_evaluated`
  - False negative evidence: `not_evaluated`
  - Testing: `untested`
- Producers:
  - `ball_carry` via `BallCarryNode` / `BallCarryCalculator`

**Summary**

A sustained player-ball control sequence, covering grounded carries and air dribbles.

**Approach**

- Use continuous ball-control tracking to build player-owned sequences while live play is active.
- Sample grounded carries from close horizontal/vertical ball gaps over the car, excluding wall contact.
- Sample air dribbles with the air-dribble policy, then emit completed sequences that meet the duration and validity rules for their carry kind.

**Limitations**

_None documented._

**Known Issues**

_None documented._

### Ball Depth (`ball_depth`)

- Category: `positioning`
- Confidence:
  - Approach: `unknown`
  - True positive evidence: `not_evaluated`
  - False positive evidence: `not_evaluated`
  - False negative evidence: `not_evaluated`
  - Testing: `untested`
- Producers:
  - `positioning` via `PositioningNode` / `PositioningCalculator`

**Summary**

Definition pending.

**Approach**

_None documented._

**Limitations**

_None documented._

**Known Issues**

_None documented._

### Ball Half (`ball_half`)

- Category: `other`
- Confidence:
  - Approach: `unknown`
  - True positive evidence: `not_evaluated`
  - False positive evidence: `not_evaluated`
  - False negative evidence: `not_evaluated`
  - Testing: `untested`
- Producers:
  - `ball_half` via `BallHalfNode` / `BallHalfCalculator`

**Summary**

Definition pending.

**Approach**

_None documented._

**Limitations**

_None documented._

**Known Issues**

_None documented._

### Ball Proximity (`ball_proximity`)

- Category: `positioning`
- Confidence:
  - Approach: `unknown`
  - True positive evidence: `not_evaluated`
  - False positive evidence: `not_evaluated`
  - False negative evidence: `not_evaluated`
  - Testing: `untested`
- Producers:
  - `positioning` via `PositioningNode` / `PositioningCalculator`

**Summary**

Definition pending.

**Approach**

_None documented._

**Limitations**

_None documented._

**Known Issues**

_None documented._

### Ball Third (`ball_third`)

- Category: `positioning`
- Confidence:
  - Approach: `unknown`
  - True positive evidence: `not_evaluated`
  - False positive evidence: `not_evaluated`
  - False negative evidence: `not_evaluated`
  - Testing: `untested`
- Producers:
  - `ball_third` via `BallThirdNode` / `BallThirdCalculator`

**Summary**

Definition pending.

**Approach**

_None documented._

**Limitations**

_None documented._

**Known Issues**

_None documented._

### Boost Pickup (`boost_pickups`)

- Category: `other`
- Confidence:
  - Approach: `unknown`
  - True positive evidence: `not_evaluated`
  - False positive evidence: `not_evaluated`
  - False negative evidence: `not_evaluated`
  - Testing: `untested`
- Producers:
  - `boost` via `BoostNode` / `BoostCalculator`

**Summary**

Definition pending.

**Approach**

_None documented._

**Limitations**

_None documented._

**Known Issues**

_None documented._

### Respawn (`boost_respawn`)

- Category: `other`
- Confidence:
  - Approach: `unknown`
  - True positive evidence: `not_evaluated`
  - False positive evidence: `not_evaluated`
  - False negative evidence: `not_evaluated`
  - Testing: `untested`
- Producers:
  - `boost` via `BoostNode` / `BoostCalculator`

**Summary**

Definition pending.

**Approach**

_None documented._

**Limitations**

_None documented._

**Known Issues**

_None documented._

### Bump (`bump`)

- Category: `other`
- Confidence:
  - Approach: `unknown`
  - True positive evidence: `not_evaluated`
  - False positive evidence: `not_evaluated`
  - False negative evidence: `not_evaluated`
  - Testing: `untested`
- Producers:
  - `bump` via `BumpNode` / `BumpCalculator`

**Summary**

Definition pending.

**Approach**

_None documented._

**Limitations**

_None documented._

**Known Issues**

_None documented._

### Ceiling Shot (`ceiling_shot`)

- Category: `mechanic`
- Confidence:
  - Approach: `unknown`
  - True positive evidence: `not_evaluated`
  - False positive evidence: `not_evaluated`
  - False negative evidence: `not_evaluated`
  - Testing: `untested`
- Producers:
  - `ceiling_shot` via `CeilingShotNode` / `CeilingShotCalculator`

**Summary**

A shot touch shortly after the player contacts the ceiling and drops back toward the ball.

**Approach**

- Record recent ceiling contacts when the car is near the ceiling and oriented roof-first against it.
- Match a later touch by the same player within the ceiling-contact window after the player has separated from the ceiling.
- Score the candidate from contact timing, height, separation, forward alignment, approach speed, ball impulse, and ceiling-contact alignment.

**Limitations**

_None documented._

**Known Issues**

_None documented._

### Center (`center`)

- Category: `mechanic`
- Confidence:
  - Approach: `unknown`
  - True positive evidence: `not_evaluated`
  - False positive evidence: `not_evaluated`
  - False negative evidence: `not_evaluated`
  - Testing: `untested`
- Producers:
  - `center` via `CenterNode` / `CenterCalculator`

**Summary**

A touch that moves the ball from a wide attacking position toward the central attacking area.

**Approach**

- Start a pending center from a live-play touch, unless that player immediately has a shot or goal event.
- Watch the ball for a short window after the touch and require meaningful travel from a wide x-position toward a more central x-position in the attacking half.
- Clear the candidate when it ages out, loses attribution, or becomes a shot/goal by the same player instead of a center.

**Limitations**

_None documented._

**Known Issues**

_None documented._

### Controlled Play (`controlled_play`)

- Category: `mechanic`
- Confidence:
  - Approach: `unknown`
  - True positive evidence: `not_evaluated`
  - False positive evidence: `not_evaluated`
  - False negative evidence: `not_evaluated`
  - Testing: `untested`
- Producers:
  - `controlled_play` via `ControlledPlayNode` / `ControlledPlayCalculator`

**Summary**

A same-player possession episode with multiple touches and sustained close-ball time.

**Approach**

- Start a player-owned candidate from an attributed touch during live play.
- Require at least two distinct touches by the same player with at least one second between the first and last touch.
- Require sustained proximity to the ball and finish the candidate when another player touches, live play ends, or the touch chain times out.

**Limitations**

_None documented._

**Known Issues**

_None documented._

### Core Player Scoreboard (`core_player_scoreboard`)

- Category: `core`
- Confidence:
  - Approach: `unknown`
  - True positive evidence: `not_evaluated`
  - False positive evidence: `not_evaluated`
  - False negative evidence: `not_evaluated`
  - Testing: `untested`
- Producers:
  - `match_stats` via `MatchStatsNode` / `MatchStatsCalculator`

**Summary**

Definition pending.

**Approach**

_None documented._

**Limitations**

_None documented._

**Known Issues**

_None documented._

### Demolition (`demolition`)

- Category: `basic`
- Confidence:
  - Approach: `unknown`
  - True positive evidence: `not_evaluated`
  - False positive evidence: `not_evaluated`
  - False negative evidence: `not_evaluated`
  - Testing: `untested`
- Producers:
  - `demo` via `DemoNode` / `DemoCalculator`

**Summary**

Definition pending.

**Approach**

_None documented._

**Limitations**

_None documented._

**Known Issues**

_None documented._

### Depth Role (`depth_role`)

- Category: `positioning`
- Confidence:
  - Approach: `unknown`
  - True positive evidence: `not_evaluated`
  - False positive evidence: `not_evaluated`
  - False negative evidence: `not_evaluated`
  - Testing: `untested`
- Producers:
  - `positioning` via `PositioningNode` / `PositioningCalculator`

**Summary**

Definition pending.

**Approach**

_None documented._

**Limitations**

_None documented._

**Known Issues**

_None documented._

### Dodge (`dodge`)

- Category: `basic`
- Confidence:
  - Approach: `unknown`
  - True positive evidence: `not_evaluated`
  - False positive evidence: `not_evaluated`
  - False negative evidence: `not_evaluated`
  - Testing: `untested`
- Producers:
  - `dodge` via `FlipImpulseNode` / `FlipImpulseCalculator`

**Summary**

A dodge-start event, optionally carrying a rough estimated dodge impulse when the velocity change is measurable.

**Approach**

- Start on the replay's dodge-active rising edge for each player.
- Sample the player's velocity change over the early dodge window and subtract an approximate forward boost contribution when boost is active.
- Store the impulse estimate as dodge_impulse, including car-local direction classification plus raw and compensated world-space vectors for visualization and downstream mechanic detectors.

**Limitations**

_None documented._

**Known Issues**

_None documented._

### Dodge Reset (`dodge_reset`)

- Category: `basic`
- Confidence:
  - Approach: `unknown`
  - True positive evidence: `not_evaluated`
  - False positive evidence: `not_evaluated`
  - False negative evidence: `not_evaluated`
  - Testing: `untested`
- Producers:
  - `dodge_reset` via `DodgeResetNode` / `DodgeResetCalculator`

**Summary**

A frame-level dodge refresh observed from replay state, marked as occurring on the ball (a flip reset) and as used when later converted by a dodge-powered touch.

**Approach**

- Consume dodge-refreshed replay events and preserve the player, team, frame, time, and counter value.
- Classify the refresh as on-ball (a flip reset) when the player and ball are both airborne enough, close together, and the ball is positioned under the car in local space.
- Keep on-ball resets pending in an in-flight ledger; if the player dodges into the ball within the reset-to-touch window, mark the originating reset event `used` with its reset-to-use latency.
- Resolve every pending reset into an outcome: used, landed, superseded by a newer reset, expired, or cut off by a goal, live play ending, or the replay ending.

**Limitations**

_None documented._

**Known Issues**

_None documented._

### Double Tap (`double_tap`)

- Category: `mechanic`
- Confidence:
  - Approach: `unknown`
  - True positive evidence: `not_evaluated`
  - False positive evidence: `not_evaluated`
  - False negative evidence: `not_evaluated`
  - Testing: `untested`
- Producers:
  - `double_tap` via `DoubleTapNode` / `DoubleTapCalculator`

**Summary**

A same-player follow-up touch after an attributed backboard bounce that creates a shot-like trajectory.

**Approach**

- Arm a pending double tap from a backboard-bounce event attributed to the player who sent the ball to the backboard.
- Require the same player and team to touch the ball again during live play within the follow-up window.
- Accept the follow-up only when the post-touch straight-line ball trajectory projects into or close to the opponent goal mouth.

**Limitations**

_None documented._

**Known Issues**

_None documented._

### Event (`event`)

- Category: `mechanic`
- Confidence:
  - Approach: `unknown`
  - True positive evidence: `not_evaluated`
  - False positive evidence: `not_evaluated`
  - False negative evidence: `not_evaluated`
  - Testing: `untested`
- Producers:
  - `stats_timeline_events` via `StatsTimelineEventsNode` / `StatsTimelineEventsState`

**Summary**

A shared event envelope with common metadata and a typed event payload.

**Approach**

- Collect completed events from the analysis graph at finish time.
- Wrap each typed event payload with common timing, participant, team, position, confidence, and stream metadata.
- Serialize timeline events as a single heterogeneous event list for playback and analysis consumers.

**Limitations**

_None documented._

**Known Issues**

_None documented._

### Field Half (`field_half`)

- Category: `positioning`
- Confidence:
  - Approach: `unknown`
  - True positive evidence: `not_evaluated`
  - False positive evidence: `not_evaluated`
  - False negative evidence: `not_evaluated`
  - Testing: `untested`
- Producers:
  - `positioning` via `PositioningNode` / `PositioningCalculator`

**Summary**

Definition pending.

**Approach**

_None documented._

**Limitations**

_None documented._

**Known Issues**

_None documented._

### Field Third (`field_third`)

- Category: `positioning`
- Confidence:
  - Approach: `unknown`
  - True positive evidence: `not_evaluated`
  - False positive evidence: `not_evaluated`
  - False negative evidence: `not_evaluated`
  - Testing: `untested`
- Producers:
  - `positioning` via `PositioningNode` / `PositioningCalculator`

**Summary**

Definition pending.

**Approach**

_None documented._

**Limitations**

_None documented._

**Known Issues**

_None documented._

### 50/50 (`fifty_fifty`)

- Category: `other`
- Confidence:
  - Approach: `unknown`
  - True positive evidence: `not_evaluated`
  - False positive evidence: `not_evaluated`
  - False negative evidence: `not_evaluated`
  - Testing: `untested`
- Producers:
  - `fifty_fifty` via `FiftyFiftyNode` / `FiftyFiftyCalculator`

**Summary**

A contested ball interaction involving touches or pressure from both teams in a short window.

**Approach**

- Start an active 50/50 when a frame contains touches from both teams, including kickoff-specific tracking.
- Continue the contest for short follow-up touch windows while either involved team remains in contact.
- Resolve after a delay once ball movement, possession state, or max duration gives a winner, possession outcome, or neutral result.

**Limitations**

_None documented._

**Known Issues**

_None documented._

### First-Man Change (`first_man_change`)

- Category: `positioning`
- Confidence:
  - Approach: `unknown`
  - True positive evidence: `not_evaluated`
  - False positive evidence: `not_evaluated`
  - False negative evidence: `not_evaluated`
  - Testing: `untested`
- Producers:
  - `rotation` via `RotationNode` / `RotationCalculator`

**Summary**

Definition pending.

**Approach**

_None documented._

**Limitations**

_None documented._

**Known Issues**

_None documented._

### Flick (`flick`)

- Category: `mechanic`
- Confidence:
  - Approach: `unknown`
  - True positive evidence: `not_evaluated`
  - False positive evidence: `not_evaluated`
  - False negative evidence: `not_evaluated`
  - Testing: `untested`
- Producers:
  - `flick` via `FlickNode` / `FlickCalculator`

**Summary**

A dodge-powered touch following a short controlled carry setup.

**Approach**

- Track controlled setup windows where the current controlling player keeps the ball close above the car within local-position and gap thresholds.
- Measure signed horizontal setup rotation so reverse flicks can be labeled as left or right based on the direction the car rotated before the flick.
- Record dodge starts that happen immediately after, or during, a qualifying setup.
- Emit on a same-player touch shortly after the dodge when the ball impulse is large and directed away from the player, with confidence from setup duration, timing, impulse, and separation.

**Limitations**

_None documented._

**Known Issues**

_None documented._

### Flip Reset (`flip_reset`)

- Category: `mechanic`
- Confidence:
  - Approach: `unknown`
  - True positive evidence: `not_evaluated`
  - False positive evidence: `not_evaluated`
  - False negative evidence: `not_evaluated`
  - Testing: `untested`
- Producers:
  - `flip_reset` via `DodgeResetNode` / `DodgeResetCalculator`

**Summary**

An on-ball dodge refresh that is confirmed when the player uses the gained dodge and touches the ball again before landing.

**Approach**

- Consume on-ball dodge refreshes detected from replay state as pending flip-reset candidates.
- Require a later dodge start by the same player while the reset is still pending.
- Confirm only when that player touches the ball while dodge-active before landing and within the reset-to-touch window.

**Limitations**

_None documented._

**Known Issues**

_None documented._

### Half Flip (`half_flip`)

- Category: `mechanic`
- Confidence:
  - Approach: `unknown`
  - True positive evidence: `not_evaluated`
  - False positive evidence: `not_evaluated`
  - False negative evidence: `not_evaluated`
  - Testing: `untested`
- Producers:
  - `half_flip` via `HalfFlipNode` / `HalfFlipCalculator`

**Summary**

A dodge sequence that cancels a flip into an opposite facing direction.

**Approach**

- Start candidates on low grounded or low-air dodge rising edges.
- Track the car's forward vector through the evaluation window, including vertical flip evidence and final horizontal facing direction.
- Emit when the candidate has pitched through a flip, reaches and retains roughly opposite facing instead of rotating through a full end-over-end flip, and finishes with a meaningful horizontal facing direction.

**Limitations**

_None documented._

**Known Issues**

_None documented._

### Half Volley (`half_volley`)

- Category: `mechanic`
- Confidence:
  - Approach: `unknown`
  - True positive evidence: `not_evaluated`
  - False positive evidence: `not_evaluated`
  - False negative evidence: `not_evaluated`
  - Testing: `untested`
- Producers:
  - `half_volley` via `HalfVolleyNode` / `HalfVolleyCalculator`

**Summary**

A fast touch shortly after the ball bounces off the floor, paired with a recent player dodge.

**Approach**

- Detect floor bounces from ball height and vertical velocity reversal when no touch occurs on the bounce frame.
- Track each player's recent ground contact and dodge start.
- Emit on a same-player touch shortly after the floor bounce and dodge when the post-touch ball speed clears the configured threshold.

**Limitations**

_None documented._

**Known Issues**

_None documented._

### Movement (`movement`)

- Category: `other`
- Confidence:
  - Approach: `unknown`
  - True positive evidence: `not_evaluated`
  - False positive evidence: `not_evaluated`
  - False negative evidence: `not_evaluated`
  - Testing: `untested`
- Producers:
  - `movement` via `MovementNode` / `MovementCalculator`

**Summary**

Definition pending.

**Approach**

_None documented._

**Limitations**

_None documented._

**Known Issues**

_None documented._

### One Timer (`one_timer`)

- Category: `mechanic`
- Confidence:
  - Approach: `unknown`
  - True positive evidence: `not_evaluated`
  - False positive evidence: `not_evaluated`
  - False negative evidence: `not_evaluated`
  - Testing: `untested`
- Producers:
  - `one_timer` via `OneTimerNode` / `OneTimerCalculator`

**Summary**

A fast receiver touch from a completed pass that is immediately directed toward goal.

**Approach**

- Consume newly completed pass events on the frame they are recorded.
- Require the current ball speed after the receiver's touch to exceed the one-timer speed threshold.
- Require the post-touch ball velocity to align with the opponent goal center direction.

**Limitations**

_None documented._

**Known Issues**

_None documented._

### Pass (`pass`)

- Category: `mechanic`
- Confidence:
  - Approach: `unknown`
  - True positive evidence: `not_evaluated`
  - False positive evidence: `not_evaluated`
  - False negative evidence: `not_evaluated`
  - Testing: `untested`
- Producers:
  - `pass` via `PassNode` / `PassCalculator`

**Summary**

A same-team touch sequence where one player sends the ball to a different teammate.

**Approach**

- Track the last attributed touch in live play and compare it to each new touch.
- Emit when a different teammate touches the ball within the pass window after the ball has traveled far enough.
- Classify the pass as direct, backboard, fifty-fifty, or fifty-fifty backboard using intervening backboard-bounce and fifty-fifty state.

**Limitations**

_None documented._

**Known Issues**

_None documented._

### Player Activity (`player_activity`)

- Category: `positioning`
- Confidence:
  - Approach: `unknown`
  - True positive evidence: `not_evaluated`
  - False positive evidence: `not_evaluated`
  - False negative evidence: `not_evaluated`
  - Testing: `untested`
- Producers:
  - `positioning` via `PositioningNode` / `PositioningCalculator`

**Summary**

Definition pending.

**Approach**

_None documented._

**Limitations**

_None documented._

**Known Issues**

_None documented._

### Player Possession (`player_possession`)

- Category: `other`
- Confidence:
  - Approach: `unknown`
  - True positive evidence: `not_evaluated`
  - False positive evidence: `not_evaluated`
  - False negative evidence: `not_evaluated`
  - Testing: `untested`
- Producers:
  - `player_possession` via `PlayerPossessionNode` / `PlayerPossessionCalculator`

**Summary**

A contiguous single-player possession span enriched with touch, ball-progress, and sustained-control activity.

**Approach**

- Follow the shared possession tracker's controlling player and open a span when a player establishes control.
- Bridge contested or pending-turnover interruptions shorter than the merge gap when the same player re-establishes control, excluding the gap from possessed duration.
- Accumulate distinct touches (with aerial/wall classification), signed ball travel toward the opponent goal, and per-frame carry/air-dribble samples while the span is active.

**Limitations**

_None documented._

**Known Issues**

_None documented._

### Possession (`possession`)

- Category: `other`
- Confidence:
  - Approach: `unknown`
  - True positive evidence: `not_evaluated`
  - False positive evidence: `not_evaluated`
  - False negative evidence: `not_evaluated`
  - Testing: `untested`
- Producers:
  - `possession` via `PossessionNode` / `PossessionCalculator`

**Summary**

Definition pending.

**Approach**

_None documented._

**Limitations**

_None documented._

**Known Issues**

_None documented._

### Powerslide (`powerslide`)

- Category: `basic`
- Confidence:
  - Approach: `unknown`
  - True positive evidence: `not_evaluated`
  - False positive evidence: `not_evaluated`
  - False negative evidence: `not_evaluated`
  - Testing: `untested`
- Producers:
  - `powerslide` via `PowerslideNode` / `PowerslideCalculator`

**Summary**

A state-change event for effective grounded powerslide use.

**Approach**

- Read each player's powerslide-active input/state on every frame.
- Treat powerslide as effective only while the player is close enough to the ground.
- Emit when a player's effective powerslide state changes between active and inactive.

**Limitations**

_None documented._

**Known Issues**

_None documented._

### Rotation Role (`rotation_role`)

- Category: `positioning`
- Confidence:
  - Approach: `unknown`
  - True positive evidence: `not_evaluated`
  - False positive evidence: `not_evaluated`
  - False negative evidence: `not_evaluated`
  - Testing: `untested`
- Producers:
  - `rotation` via `RotationNode` / `RotationCalculator`

**Summary**

Definition pending.

**Approach**

_None documented._

**Limitations**

_None documented._

**Known Issues**

_None documented._

### Rush (`rush`)

- Category: `other`
- Confidence:
  - Approach: `unknown`
  - True positive evidence: `not_evaluated`
  - False positive evidence: `not_evaluated`
  - False negative evidence: `not_evaluated`
  - Testing: `untested`
- Producers:
  - `rush` via `RushNode` / `RushCalculator`

**Summary**

A quick possession transition where the attacking team has numbers moving out of its defensive half.

**Approach**

- Start from a possession change when the ball is still in the new attacking team's defensive half.
- Count non-demoed attackers near or ahead of the ball and defenders between the ball and their own goal.
- Emit once the new attacking team retains possession long enough with at least two attackers and at least one defender in the rush shape.

**Limitations**

_None documented._

**Known Issues**

_None documented._

### Speed Flip (`speed_flip`)

- Category: `mechanic`
- Confidence:
  - Approach: `unknown`
  - True positive evidence: `not_evaluated`
  - False positive evidence: `not_evaluated`
  - False negative evidence: `not_evaluated`
  - Testing: `untested`
- Producers:
  - `speed_flip` via `SpeedFlipNode` / `SpeedFlipCalculator`

**Summary**

A ground-started diagonal dodge/cancel acceleration pattern, primarily intended for kickoff speed flips.

**Approach**

- Start candidates on dodge rising edges while the player is grounded, moving in the car's forward direction, and, for kickoff cases, within the kickoff-start window.
- Track speed, forward alignment, boost alignment, diagonal angular-velocity balance, and early forward acceleration during a short evaluation window.
- Emit when the combined diagonal, cancel, speed, and alignment confidence score clears the speed-flip threshold before the candidate expires.

**Limitations**

_None documented._

**Known Issues**

_None documented._

### Territorial Pressure (`territorial_pressure`)

- Category: `other`
- Confidence:
  - Approach: `unknown`
  - True positive evidence: `not_evaluated`
  - False positive evidence: `not_evaluated`
  - False negative evidence: `not_evaluated`
  - Testing: `untested`
- Producers:
  - `territorial_pressure` via `TerritorialPressureNode` / `TerritorialPressureCalculator`

**Summary**

Definition pending.

**Approach**

_None documented._

**Limitations**

_None documented._

**Known Issues**

_None documented._

### Replay Timeline Event (`timeline`)

- Category: `core`
- Confidence:
  - Approach: `unknown`
  - True positive evidence: `not_evaluated`
  - False positive evidence: `not_evaluated`
  - False negative evidence: `not_evaluated`
  - Testing: `untested`
- Producers:
  - `match_stats` via `MatchStatsNode` / `MatchStatsCalculator`

**Summary**

Definition pending.

**Approach**

_None documented._

**Limitations**

_None documented._

**Known Issues**

_None documented._

### Touch (`touch`)

- Category: `basic`
- Confidence:
  - Approach: `unknown`
  - True positive evidence: `not_evaluated`
  - False positive evidence: `not_evaluated`
  - False negative evidence: `not_evaluated`
  - Testing: `untested`
- Producers:
  - `touch` via `TouchNode` / `TouchCalculator`

**Summary**

A classified ball touch carrying a set of independent tags: strength, surface/height context, action, and an outcome-based possession tag.

**Approach**

- Carry classification as a set of (group, value) tags rather than rivalrous fields, so independent reads coexist: a boom that the hitter recovers is tagged both action=boom and possession=advance.
- Tag strength kind (control, medium hit, hard hit) from the ball speed change, plus surface, height band, and dodge context for the touching player at contact time.
- Resolve a single mutually-exclusive action by precedence: replay-confirmed saves and shots first, then geometric save/shot trajectory projections, then clears out of the defensive third, then passes led toward a teammate, then booms hit hard downfield into space. A touch matching none of these has no action tag at all, rather than a catch-all value.
- Retroactively raise the action to shot/save by outcome when stronger evidence arrives after the touch: a scored goal (the scorer's touch), a replay shot/save stat event that lands after the touch, or a settled post-touch trajectory that crosses the goal mouth. Upgrades only ever raise the action, never downgrade it.
- Tag a touch contested independently of its action (a contested shot stays a shot and is also flagged contested), rather than collapsing contests into the action.
- Retroactively add a possession tag by outcome on pass/clear/boom and action-less touches: control when the toucher stays close to the ball while matching its velocity for most of a short follow window or wins the follow-up touch with the ball kept near, advance when they win the follow-up touch after playing the ball clear into space.
- Tag a touch's reception as a first touch when it starts a new reception: the previous global touch was by a different player or far enough in the past.

**Limitations**

_None documented._

**Known Issues**

_None documented._

### Wall Aerial (`wall_aerial`)

- Category: `mechanic`
- Confidence:
  - Approach: `unknown`
  - True positive evidence: `not_evaluated`
  - False positive evidence: `not_evaluated`
  - False negative evidence: `not_evaluated`
  - Testing: `untested`
- Producers:
  - `wall_aerial` via `WallAerialNode` / `WallAerialCalculator`

**Summary**

An aerial play that starts from controlled ball movement on a side or back wall.

**Approach**

- Track wall-control sequences where the last toucher keeps the ball close while positioned on a side or back wall.
- Arm a wall-aerial candidate when the player leaves the wall soon after a qualifying wall-control setup.
- Emit on a later aerial touch by the same player when the player and ball are high enough, the setup/takeoff windows hold, and the confidence score clears the threshold.

**Limitations**

_None documented._

**Known Issues**

_None documented._

### Wall Aerial Shot (`wall_aerial_shot`)

- Category: `mechanic`
- Confidence:
  - Approach: `unknown`
  - True positive evidence: `not_evaluated`
  - False positive evidence: `not_evaluated`
  - False negative evidence: `not_evaluated`
  - Testing: `untested`
- Producers:
  - `wall_aerial_shot` via `WallAerialShotNode` / `WallAerialShotCalculator`

**Summary**

A shot credited to a player shortly after taking off from a wall.

**Approach**

- Track recent wall contact for each player and arm a candidate when the player leaves the wall while still above the ground threshold.
- Match a subsequent shot stat event by that player within the takeoff-to-shot window.
- Require the shot touch to occur off the wall with sufficient player and ball height, then score confidence from timing, height, goal alignment, and ball speed.

**Limitations**

_None documented._

**Known Issues**

_None documented._

### Wavedash (`wavedash`)

- Category: `mechanic`
- Confidence:
  - Approach: `unknown`
  - True positive evidence: `not_evaluated`
  - False positive evidence: `not_evaluated`
  - False negative evidence: `not_evaluated`
  - Testing: `untested`
- Producers:
  - `wavedash` via `WavedashNode` / `WavedashCalculator`

**Summary**

A low airborne dodge that lands quickly and converts the dodge into ground speed.

**Approach**

- Start candidates on dodge rising edges from a low but airborne height.
- Watch for a landing within the wavedash window while the car is sufficiently upright.
- Score confidence from dodge-to-landing timing, starting height, speed gain or landing speed, and landing uprightness.

**Limitations**

_None documented._

**Known Issues**

_None documented._

### Whiff (`whiff`)

- Category: `other`
- Confidence:
  - Approach: `unknown`
  - True positive evidence: `not_evaluated`
  - False positive evidence: `not_evaluated`
  - False negative evidence: `not_evaluated`
  - Testing: `untested`
- Producers:
  - `whiff` via `WhiffNode` / `WhiffCalculator`

**Summary**

A committed attempt near the ball that does not result in that player touching it.

**Approach**

- Start candidates when a player gets within hitbox distance of the ball while moving or dodging toward it with sufficient alignment and closing speed.
- Track the closest approach while the candidate remains near the ball.
- Resolve as a whiff when the player exits the candidate window without touching, or as beaten-to-ball when an opponent touches first.

**Limitations**

_None documented._

**Known Issues**

_None documented._

## Goal Tags

### Aerial Goal (`aerial_goal`)


**Summary**

A goal whose scorer last touched the ball while it was high in the air.

**Approach**

- Inspect each goal context and its scorer-last-touch evidence.
- Require the last-touch ball height to meet the aerial-goal threshold.
- Attach goal-context and last-touch evidence to the goal tag metadata.

### High Aerial Goal (`high_aerial_goal`)


**Summary**

A goal whose scoring possession includes a touch taken from a high ball height, even when the finishing touch itself was lower.

**Approach**

- Scan the scoring team's touches within the possession that led to the goal (back to the last turnover or neutral loose ball).
- Require at least one such touch to meet the high-aerial ball-height threshold.
- Tag the goal from the highest qualifying touch, attaching it as leadup-touch evidence.

### Long-Distance Goal (`long_distance_goal`)


**Summary**

A goal where the scorer's last touch started from deep enough in the attacking team's half-space.

**Approach**

- Use the scorer-last-touch ball position from goal context.
- Normalize field direction by scoring team and compare the touch y-position to the long-distance threshold.
- Attach goal-context and last-touch evidence to the goal tag metadata.

### Own-Half Goal (`own_half_goal`)


**Summary**

A long-distance goal where the scorer's last touch came from their own half and close enough in time to the goal.

**Approach**

- Use the scorer-last-touch ball position and time from goal context.
- Require the touch to be in the scoring team's own half and within the own-half touch-to-goal window.
- Allow the long-distance goal tag to also apply when both distance thresholds are met.

### Empty Net Goal (`empty_net_goal`)


**Summary**

A goal where defenders are judged too far or too poorly positioned to cover the net.

**Approach**

- Inspect defending-player positions in the goal context.
- Compare defender depth and distance against the empty-net thresholds.
- Avoid tagging very deep attacking touches as empty nets when the touch position is outside the configured range.

### Counter-Attack Goal (`counter_attack_goal`)


**Summary**

A goal whose buildup was classified as a counterattack.

**Approach**

- Use the goal-buildup classification computed in goal context.
- Tag goals whose buildup kind is counterattack.
- Attach goal-buildup evidence to the goal tag metadata.

### Sustained Pressure Goal (`sustained_pressure_goal`)


**Summary**

A goal whose buildup was classified as sustained offensive pressure.

**Approach**

- Use the goal-buildup classification computed in goal context.
- Tag goals whose buildup kind is sustained pressure.
- Attach goal-buildup evidence to the goal tag metadata.

### Flick Goal (`flick_goal`)


**Summary**

A goal linked to a recent flick event.

**Approach**

- Compare recent flick events against each goal's scorer-last-touch context.
- Require the flick to fall within the configured event-to-goal window.
- Prefer by-scorer evidence when the flick player matches the scorer's last touch.

### Ceiling-Shot Goal (`ceiling_shot_goal`)


**Summary**

A goal linked to a recent ceiling-shot event.

**Approach**

- Compare recent ceiling-shot events against each goal's scorer-last-touch context.
- Require the ceiling shot to fall within the configured event-to-goal window.
- Attach a related ceiling-shot event reference and ceiling-shot evidence to the goal tag metadata.

### Double-Tap Goal (`double_tap_goal`)


**Summary**

A goal linked to a recent double-tap event.

**Approach**

- Compare recent double-tap events against each goal's scorer-last-touch context.
- Require the double tap to fall within the configured event-to-goal window.
- Attach a related-event reference and mechanic evidence to the goal tag metadata.

### One-Timer Goal (`one_timer_goal`)


**Summary**

A goal linked to a recent one-timer event.

**Approach**

- Compare recent one-timer events against each goal's scorer-last-touch context.
- Require the one timer to fall within the configured event-to-goal window.
- Prefer by-scorer evidence when the one-timer receiver matches the scorer's last touch.

### Passing Goal (`passing_goal`)


**Summary**

A goal where a completed pass is linked to the scoring touch.

**Approach**

- Compare pass events against each goal's scorer-last-touch context.
- Require the pass receiver to match the scorer's last touch within the pass-to-goal window.
- Attach a related pass-event reference and pass evidence to the goal tag metadata.

### Air-Dribble Goal (`air_dribble_goal`)


**Summary**

A goal linked to an air-dribble ball-carry sequence that reaches the scoring touch.

**Approach**

- Inspect completed ball-carry events whose kind is air dribble.
- Match air-dribble sequences to goals by timing and scorer-last-touch context.
- Attach a related ball-carry event reference and air-dribble evidence to the goal tag metadata.

### Flip-Reset Goal (`flip_reset_goal`)


**Summary**

A goal linked to a recent on-ball dodge reset or flip-reset event.

**Approach**

- Compare reset-related mechanic events against each goal's scorer-last-touch context.
- Require the reset evidence to fall within the configured event-to-goal window.
- Prefer by-scorer evidence when the reset player matches the scorer's last touch.

### Flip-Into-Ball Goal (`flip_into_ball_goal`)


**Summary**

A goal where the scorer flipped (dodged) into the ball on the scoring touch.

**Approach**

- Match the scorer's last touch to its touch-classification event by touch id (player and frame for data predating touch ids).
- Require the scoring touch's dodge state to be active and the touch to fall within the touch-to-goal window.
- Limitation: the dodge state covers any active dodge overlapping the touch, so incidental flips that happen to contact the ball can also qualify; dodge direction toward the ball is not yet verified.

### Bump Goal (`bump_goal`)


**Summary**

A goal linked to a recent scoring-team bump on an opponent.

**Approach**

- Compare non-team bump events against each goal's timing and scoring team.
- Require the bump initiator to be on the scoring team and within the configured event-to-goal window.
- Attach a related bump-event reference and bump evidence, even when the initiator is not the scorer.

### Demo Goal (`demo_goal`)


**Summary**

A goal linked to a recent scoring-team demolition.

**Approach**

- Compare demolition kill events against each goal's timing and scoring team.
- Require the demo attacker to be on the scoring team and within the configured event-to-goal window.
- Attach a related demo-event reference and demo evidence, even when the attacker is not the scorer.

### Half-Volley Goal (`half_volley_goal`)


**Summary**

A goal where the scorer's last touch matches a recent half-volley candidate.

**Approach**

- Compare half-volley events against each goal's scorer-last-touch context.
- Require the half-volley touch to be close enough to the goal and sufficiently aligned toward goal.
- Attach a related half-volley event reference and half-volley evidence to the goal tag metadata.

### Kickoff Goal (`kickoff_goal`)


**Summary**

A goal flowing directly from the kickoff exchange.

**Approach**

- Use the kickoff calculator's goal attribution as the source of truth.
- Require the goal to land within the kickoff-goal timing window of the first touch.
- Reject goals where the conceding team settled possession or the play reset through the scoring team's own half.
- Attach goal-context evidence so the tag appears with the goal label.

