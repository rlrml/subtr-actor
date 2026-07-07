export {
  createBoostPadsPlugin,
  capturePlayerImage,
  capturePlayerImageFromParsed,
  capturePlayerImages,
  capturePlayerImagesFromParsed,
  createCameraPlugin,
  createFpsOverlayPlugin,
  createNameTagPlugin,
  createScoredTextPlugin,
  createPlayer,
  createPlayerFromParsed,
  fromReplayPlayerPlugin,
  loadReplay,
  parseReplay,
  SubtrActorPlayer,
  ReplayPlayer,
} from "./player/lib";
export { getPlayerAssetBase, resolvePlayerAssetUrl, setPlayerAssetBase } from "./player/asset-url";
export type {
  CameraPlugin,
  CameraPluginMode,
  CameraPluginOptions,
  FpsOverlayOptions,
  FpsSample,
  RecordedCameraSettings,
  ScoredTextOverlayOptions,
  SubtrActorPlayerOptions,
  PlayerImageBallCamMode,
  PlayerImageCamera,
  PlayerImageCaptureOptions,
  PlayerImageCaptureRequest,
  PlayerImageCaptureResult,
  PlayerCameraViewMode,
  PlayerFreeCameraPreset,
  PlayerOptions,
  ReplayPlayerInfo,
  PlayerPlugin,
  PlayerPluginContext,
  PlayerPluginDefinition,
  PlayerPluginFactory,
  PlayerPluginStateContext,
  PlayerRenderContext,
  PlayerSnapshot,
  PlayerState,
  PlayerStatePatch,
} from "./player/lib";
export {
  BALLCHASING_API_BASE_URL,
  BALLCHASING_BASE_URL,
  createBallchasingReplaySource,
  fetchBallchasingReplayBytes,
  getBallchasingReplayApiFileUrl,
  getBallchasingReplayFileName,
  getBallchasingReplayFileUrl,
  isBallchasingReplayId,
  normalizeBallchasingReplayId,
} from "./ballchasing";
export type { BallchasingReplayDownloadOptions } from "./ballchasing";
export {
  createBallchasingOverlayPlugin,
  DEFAULT_FLOATING_NAMEPLATE_LIFT_UU,
} from "./ballchasing-overlay";
export type { BallchasingOverlayPluginOptions } from "./ballchasing-overlay";
export { BOOST_RAW_MAX, boostAmountToPercent, boostPercentToAmount } from "./boost-units";
export { createBoostPadOverlayPlugin } from "./boost-pad-overlay";
export type { BoostPadOverlayPluginOptions } from "./boost-pad-overlay";
export { createBoostPickupAnimationPlugin } from "./boost-pickup-animation";
export type {
  BoostPickupAnimationFilter,
  BoostPickupAnimationPickup,
  BoostPickupAnimationPluginOptions,
} from "./boost-pickup-animation";
export { createCanvasRecorderPlugin } from "./canvas-recorder";
export type {
  CanvasRecorderPlugin,
  CanvasRecorderPluginOptions,
  CanvasRecorderRangeOptions,
  CanvasRecorderStartOptions,
  CanvasRecorderState,
  CanvasRecorderStatus,
  CanvasRecorderStatusListener,
} from "./canvas-recorder";
export { createTimelineOverlayPlugin, timelineEventSeekTime } from "./timeline-overlay";
export type {
  TimelineOverlayEventSourceOptions,
  TimelineOverlayPlugin,
  TimelineOverlayPluginOptions,
} from "./timeline-overlay";
export {
  loadPlaylistManifestFromFile,
  parsePlaylistManifest,
  resolvePlaylistItemsFromManifest,
} from "./manifest";
export {
  PlaylistLoadCache,
  PlaylistSession,
  ReplayPlaylistPlayer,
  createFullReplayPlaylistItem,
  createReplayBytesSource,
  createReplayFileSource,
  createReplayPathSource,
  createReplaySource,
  createStaticReplaySource,
  frameBound,
  resolvePlaylistItem,
  timeBound,
} from "./playlist";
export type {
  FullReplayPlaylistItemOptions,
  PlaylistSessionOptions,
  PlaylistSessionState,
  ReplayPlaylistPlayerSingleReplayOptions,
} from "./playlist";
export { findFrameIndexAtTime, normalizeReplayData, normalizeReplayDataAsync } from "./replay-data";
export { playerIdToString } from "./replay-data-helpers";
// Pure ReplayModel timeline utilities, exported so other players (e.g.
// @rlrml/player) can offer the same timeline-projection / skip-window
// semantics over a shared ReplayModel.
export {
  computeTimelineSegments,
  getFrameWindow,
  getKickoffCountdownMetadata,
  getReplayPlaybackEndTime,
  inferKickoffGameState,
  inferLiveGameState,
  projectReplayTimeToTimeline,
  projectTimelineTimeToReplay,
} from "./player-internals/timeline";
export {
  getActiveDemoEvent,
  getKickoffSkipTargetTime,
  getPostGoalTransitionSkipTargetTime,
  isPlayerSamplePresent,
} from "./player-helpers";
// Pure render-context math, exported so other players can synthesize a
// `ReplayPlayerRenderContext` with identical interpolation semantics.
export { interpolatePosition } from "./player-internals/spatial";
export { createStaticReplayScene, createStaticReplaySceneFromParsed } from "./static-scene";
export type {
  StaticReplayScene,
  StaticReplayPlayerSceneOptions,
  StaticReplaySceneOptions,
  StaticSceneHeatmapOptions,
  StaticScenePoint,
  StaticScenePointLayerOptions,
  StaticSceneVec3,
} from "./static-scene";
export type {
  NormalizeReplayDataAsyncOptions,
  NormalizeReplayDataOptions,
  NormalizeReplayProgress,
} from "./replay-data";
export {
  DEFAULT_REPLAY_HITBOX_KIND,
  REPLAY_HITBOX_SPECS,
  getReplayHitboxSpec,
  inferReplayHitboxKind,
  inferReplayHitboxKindFromBodyName,
  normalizeReplayHitboxKind,
} from "./hitboxes";
export type { ReplayHitboxKind, ReplayHitboxSpec } from "./hitboxes";
export {
  createReplayLoadOverlay,
  formatReplayLoadProgress,
  formatReplayLoadProgressMeta,
} from "./load-ui";
export { ensureBindingsReady, loadReplayFromBytes, validateReplayBytes } from "./wasm";
export { TrainingPackFile, defaultTrainingPack } from "./training-pack";
export type { TrainingPackBindings, TrainingPackFileOptions } from "./training-pack";
export {
  DEFAULT_TRAINING_SHOT_TIME_LIMIT_SECONDS,
  MIN_CAR_SPAWN_Z,
  appendCapturedRound,
  ballSpawnFromReplayState,
  capturedTrainingPackDefaults,
  carSpawnFromReplayState,
  generateTrainingPackGuid,
  playerCarSpawnFromReplayState,
  quaternionToRotator,
  radiansToRotatorUnits,
  trainingPackFileName,
  trainingPackGuidHex,
  velocityToRotatorAndSpeed,
} from "./training-capture";
export type {
  CapturedBallState,
  CapturedCarState,
  RotatorUnits,
  TrainingCaptureOptions,
} from "./training-capture";
export type { Guid } from "./generated/Guid";
export type { PlayerId } from "./generated/PlayerId";
export type { Round } from "./generated/Round";
export type { TrainingPack } from "./generated/TrainingPack";
export type {
  ReplayPlayerActiveMetadata,
  BallSample,
  BeforeRenderCallback,
  CameraSettings,
  ReplayCameraViewMode,
  ReplayFreeCameraPreset,
  FrameRenderInfo,
  LoadedReplay,
  PlaybackBound,
  PlaybackFrame,
  PlaylistAdvanceMode,
  PlaylistEndMode,
  PlaylistItem,
  PlaylistLoadSource,
  PlaylistManifest,
  PlaylistManifestItem,
  PlaylistManifestPage,
  PlaylistManifestPlaybackOptions,
  PlaylistManifestReplay,
  PlaylistManifestReplayLocator,
  PlaylistPlaybackOptions,
  PlaylistPreloadContext,
  PlaylistPreloadPolicy,
  PlaylistSourceLoadContext,
  PlaylistSourceLoadProgress,
  PlaylistSourceLoadState,
  PlaylistSourceLoadStatus,
  ReplayPreloadContext,
  ReplayPreloadPolicy,
  PlayerSample,
  RawReplayGameType,
  RawReplayGameTypeDetails,
  RawReplayFramesData,
  ReplayLoadResult,
  ReplayLoadOptions,
  ReplayLoadOverlayController,
  ReplayLoadOverlayOptions,
  ReplayLoadProgress,
  ReplayLoadStage,
  ReplayModel,
  ReplayPlaylistPlayerOptions,
  ReplayPlaylistPlayerSnapshot,
  ReplayPlaylistPlayerState,
  ReplayPlayerOptions,
  ReplayPlayerPlugin,
  ReplayPlayerPluginContext,
  ReplayPlayerPluginDefinition,
  ReplayPlayerPluginFactory,
  ReplayBoostPad,
  ReplayBoostPadEvent,
  ReplayBoostPadSize,
  ReplayPlayerTimelineProjection,
  ReplayPlayerTimelineSegment,
  ReplayPlayerRenderContext,
  ReplayPlayerRenderTrackContext,
  ReplayPlayerKickoffCountdownMetadata,
  ReplayPlayerSnapshot,
  ReplayPlayerPluginStateContext,
  ReplayPlayerState,
  ReplayPlayerStatePatch,
  ReplayPlayerTrack,
  ReplaySource,
  ReplayTickMark,
  ReplayTimelineEvent,
  ReplayTimelineEventKind,
  ReplayTimelineEventSource,
  ReplayTimelineRange,
  ReplayTimelineRangeSource,
  ResolvedPlaybackBound,
  ResolvedPlaylistItem,
} from "./types";
export type { ReplayScene } from "./scene";
