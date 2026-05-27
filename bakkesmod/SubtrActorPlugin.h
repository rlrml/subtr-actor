#pragma once

#include <chrono>
#include <deque>
#include <filesystem>
#include <memory>
#include <optional>
#include <string>
#include <string_view>
#include <unordered_map>
#include <utility>
#include <vector>
#include <windows.h>

#pragma comment(lib, "pluginsdk.lib")

#include "bakkesmod/plugin/bakkesmodplugin.h"
#include "bakkesmod/plugin/PluginSettingsWindow.h"
#include "bakkesmod/plugin/pluginwindow.h"
#include "bakkesmod/wrappers/Engine/UnrealStringWrapper.h"
#include "bakkesmod/wrappers/arraywrapper.h"
#include "bakkesmod/wrappers/Engine/ActorWrapper.h"
#include "bakkesmod/wrappers/GameObject/BallWrapper.h"
#include "bakkesmod/wrappers/GameObject/CarWrapper.h"
#include "bakkesmod/wrappers/GameObject/CarComponent/BoostWrapper.h"
#include "bakkesmod/wrappers/GameObject/GoalWrapper.h"
#include "bakkesmod/wrappers/GameObject/PriWrapper.h"
#include "bakkesmod/wrappers/GameObject/TeamWrapper.h"
#include "bakkesmod/wrappers/GameEvent/ServerWrapper.h"
#include "bakkesmod/wrappers/GameEvent/ReplayWrapper.h"
#include "bakkesmod/wrappers/ReplayServerWrapper.h"
#include "bakkesmod/wrappers/canvaswrapper.h"
#include "subtr_actor_bakkesmod.h"

class SubtrActorPlugin : public BakkesMod::Plugin::BakkesModPlugin,
                         public BakkesMod::Plugin::PluginWindow,
                         public BakkesMod::Plugin::PluginSettingsWindow {
public:
  void onLoad() override;
  void onUnload() override;
  void Render() override;
  std::string GetMenuName() override;
  std::string GetMenuTitle() override;
  void SetImGuiContext(uintptr_t ctx) override;
  bool ShouldBlockInput() override;
  bool IsActiveOverlay() override;
  void OnOpen() override;
  void OnClose() override;
  void RenderSettings() override;
  std::string GetPluginName() override;

private:
  using JsonLen = size_t (*)(const SaEngine *);
  using WriteJson = size_t (*)(const SaEngine *, uint8_t *, size_t);
  using NamedJsonLen = size_t (*)(const SaEngine *, const char *);
  using WriteNamedJson = size_t (*)(const SaEngine *, const char *, uint8_t *, size_t);
  using EngineCreate = SaEngine *(*)();
  using EngineDestroy = void (*)(SaEngine *);
  using EngineReset = void (*)(SaEngine *);
  using EngineFinish = int32_t (*)(SaEngine *);
  using ProcessFrame = int32_t (*)(SaEngine *, const SaLiveFrame *);
  using EventsJsonLen = JsonLen;
  using WriteEventsJson = WriteJson;
  using FrameJsonLen = JsonLen;
  using WriteFrameJson = WriteJson;
  using TimelineJsonLen = JsonLen;
  using WriteTimelineJson = WriteJson;
  using StatsJsonLen = JsonLen;
  using WriteStatsJson = WriteJson;
  using StatsModuleJsonLen = NamedJsonLen;
  using WriteStatsModuleJson = WriteNamedJson;
  using StatsModuleFrameJsonLen = NamedJsonLen;
  using WriteStatsModuleFrameJson = WriteNamedJson;
  using StatsModuleConfigJsonLen = NamedJsonLen;
  using WriteStatsModuleConfigJson = WriteNamedJson;
  using GraphOutputJsonLen = NamedJsonLen;
  using WriteGraphOutputJson = WriteNamedJson;
  using AnalysisNodeJsonLen = NamedJsonLen;
  using WriteAnalysisNodeJson = WriteNamedJson;
  using AnalysisNodeNamesJsonLen = JsonLen;
  using WriteAnalysisNodeNamesJson = WriteJson;
  using GraphInfoJsonLen = JsonLen;
  using WriteGraphInfoJson = WriteJson;
  using DrainEvents = size_t (*)(SaEngine *, SaMechanicEvent *, size_t);
  using DrainTeamEvents = size_t (*)(SaEngine *, SaTeamEvent *, size_t);
  using DrainGoalContextEvents = size_t (*)(SaEngine *, SaGoalContextEvent *, size_t);
  using ReplayAnnotationsCreate = SaReplayAnnotations *(*)(const char *);
  using ReplayAnnotationsDestroy = void (*)(SaReplayAnnotations *);
  using ReplayAnnotationCount = size_t (*)(const SaReplayAnnotations *);
  using PollReplayAnnotations =
      size_t (*)(SaReplayAnnotations *, float, SaMechanicEvent *, size_t);

  struct OverlayMessage {
    std::string text;
    LinearColor color;
    std::chrono::steady_clock::time_point expires_at;
  };

  struct UiEventRecord {
    std::string category;
    std::string type;
    std::string actor;
    std::string label;
    std::string details;
    LinearColor color;
    uint64_t frame_number = 0;
    float time = 0.0f;
  };

  enum class UiStatsWindowKind {
    Player,
    Team,
    AllPlayers,
    AllTeams,
    GoalsOverview,
    AdHoc,
    StatsModule,
  };

  struct UiStatsWindow {
    struct Entry {
      std::string stat_id;
      std::string target_id;
    };

    uint32_t id = 0;
    UiStatsWindowKind kind = UiStatsWindowKind::Player;
    bool open = true;
    bool pending_focus = false;
    bool picker_open = false;
    uint32_t selected_player_index = 0;
    uint8_t selected_team_is_team_0 = 1;
    std::string module_name;
    int module_view = 0;
    std::string picker_query;
    std::vector<Entry> entries;
    bool has_placement = false;
    bool pending_apply_placement = false;
    float x = 0.0f;
    float y = 0.0f;
    float width = 540.0f;
    float height = 330.0f;
    float viewport_width = 0.0f;
    float viewport_height = 0.0f;
    int z_index = 0;
  };

  struct UiWindowPlacement {
    bool has_placement = false;
    bool pending_apply_placement = false;
    bool pending_focus = false;
    float x = 0.0f;
    float y = 0.0f;
    float width = 0.0f;
    float height = 0.0f;
    float viewport_width = 0.0f;
    float viewport_height = 0.0f;
    int z_index = 0;
  };

  struct PlayerStatSnapshot {
    int shots = 0;
    int saves = 0;
    int assists = 0;
    int demolishes = 0;
  };

  struct TouchAttribution {
    uint32_t player_index = 0;
    uint8_t is_team_0 = 1;
  };

  HMODULE rustLibrary = nullptr;
  SaEngine *engine = nullptr;
  EngineCreate engineCreate = nullptr;
  EngineDestroy engineDestroy = nullptr;
  EngineReset engineReset = nullptr;
  EngineFinish engineFinish = nullptr;
  ProcessFrame processFrame = nullptr;
  EventsJsonLen eventsJsonLen = nullptr;
  WriteEventsJson writeEventsJson = nullptr;
  FrameJsonLen frameJsonLen = nullptr;
  WriteFrameJson writeFrameJson = nullptr;
  TimelineJsonLen timelineJsonLen = nullptr;
  WriteTimelineJson writeTimelineJson = nullptr;
  StatsJsonLen statsJsonLen = nullptr;
  WriteStatsJson writeStatsJson = nullptr;
  StatsModuleJsonLen statsModuleJsonLen = nullptr;
  WriteStatsModuleJson writeStatsModuleJson = nullptr;
  StatsModuleFrameJsonLen statsModuleFrameJsonLen = nullptr;
  WriteStatsModuleFrameJson writeStatsModuleFrameJson = nullptr;
  StatsModuleConfigJsonLen statsModuleConfigJsonLen = nullptr;
  WriteStatsModuleConfigJson writeStatsModuleConfigJson = nullptr;
  GraphOutputJsonLen graphOutputJsonLen = nullptr;
  WriteGraphOutputJson writeGraphOutputJson = nullptr;
  AnalysisNodeJsonLen analysisNodeJsonLen = nullptr;
  WriteAnalysisNodeJson writeAnalysisNodeJson = nullptr;
  AnalysisNodeNamesJsonLen analysisNodeNamesJsonLen = nullptr;
  WriteAnalysisNodeNamesJson writeAnalysisNodeNamesJson = nullptr;
  GraphInfoJsonLen graphInfoJsonLen = nullptr;
  WriteGraphInfoJson writeGraphInfoJson = nullptr;
  DrainEvents drainEvents = nullptr;
  DrainTeamEvents drainTeamEvents = nullptr;
  DrainGoalContextEvents drainGoalContextEvents = nullptr;
  ReplayAnnotationsCreate replayAnnotationsCreate = nullptr;
  ReplayAnnotationsDestroy replayAnnotationsDestroy = nullptr;
  ReplayAnnotationCount replayAnnotationCount = nullptr;
  PollReplayAnnotations pollReplayAnnotations = nullptr;

  uint64_t frameNumber = 0;
  uint64_t inputTickNumber = 0;
  float lastTime = 0.0f;
  std::optional<float> lastProcessedGameTime;
  uint64_t profileSampleCount = 0;
  double profileSamplingMs = 0.0;
  double profileProcessingMs = 0.0;
  double profileDrainMs = 0.0;
  std::shared_ptr<bool> liveTickCancelled = std::make_shared<bool>(false);
  bool loaded = false;
  bool wasInGame = false;
  std::vector<SaPlayerFrame> sampledPlayers;
  std::vector<std::string> sampledPlayerNames;
  std::vector<SaTouchEvent> pendingTouches;
  std::vector<SaDodgeRefreshedEvent> pendingDodgeRefreshes;
  std::vector<SaBoostPadEvent> pendingBoostPadEvents;
  std::vector<SaGoalEvent> pendingGoals;
  std::vector<SaPlayerStatEvent> pendingPlayerStatEvents;
  std::vector<SaDemolishEvent> pendingDemolishes;
  std::unordered_map<uint32_t, float> lastBoostAmounts;
  std::unordered_map<uintptr_t, uint32_t> carPlayerIndices;
  std::unordered_map<uintptr_t, uint32_t> priPlayerIndices;
  std::unordered_map<std::string, uint32_t> uniqueIdPlayerIndices;
  std::unordered_map<uintptr_t, uint32_t> stablePriPlayerIndices;
  std::unordered_map<uint32_t, std::string> playerNamesByIndex;
  std::unordered_map<uint32_t, uint8_t> playerTeamsByIndex;
  std::unordered_map<uintptr_t, PlayerStatSnapshot> lastPlayerStats;
  std::unordered_map<uintptr_t, PlayerStatSnapshot> suppressedPlayerStatDeltas;
  std::unordered_map<uint32_t, bool> lastDoubleJumped;
  std::unordered_map<uint32_t, bool> lastCanJump;
  std::unordered_map<uint32_t, uint64_t> lastBallTouchFrames;
  std::unordered_map<uint32_t, int32_t> dodgeRefreshCounters;
  std::unordered_map<uintptr_t, uint32_t> boostPadIds;
  std::unordered_map<uintptr_t, uint8_t> boostPadSequences;
  std::optional<std::pair<int, int>> lastTeamScores;
  std::optional<SaGoalEvent> lastGoalEvent;
  std::optional<TouchAttribution> lastTouch;
  uint32_t nextPlayerIndex = 0;
  uint32_t nextBoostPadId = 1;
  std::deque<OverlayMessage> messages;
  SaReplayAnnotations *replayAnnotations = nullptr;
  std::string replayAnnotationPath;
  bool replayAnnotationLoadFailed = false;
  uintptr_t imguiContext = 0;
  bool uiWindowOpen = false;
  bool uiLauncherOpen = true;
  bool uiScoreboardOpen = true;
  bool uiEventsOpen = true;
  bool uiStatusOpen = true;
  bool uiCameraOpen = false;
  bool uiPlaybackControlsOpen = false;
  bool uiRecordingOpen = false;
  bool uiGraphInspectorOpen = false;
  bool uiEventPlaylistOpen = false;
  bool uiMechanicsReviewOpen = false;
  bool uiReplayLoadingOpen = false;
  bool uiModuleControlsOpen = false;
  bool uiTouchControlsOpen = false;
  bool uiBoostPickupControlsOpen = false;
  bool eventPlaylistMechanicsEnabled = true;
  bool eventPlaylistTeamEventsEnabled = true;
  bool eventPlaylistGoalContextEnabled = true;
  bool eventPlaylistAutoFollow = true;
  int cameraViewMode = 0;
  int cameraFreePreset = 0;
  uint32_t cameraSelectedPlayerIndex = 0;
  float cameraDistanceScale = 1.0f;
  bool cameraCustomSettingsEnabled = false;
  bool cameraBallCamEnabled = false;
  float cameraCustomFov = 110.0f;
  float cameraCustomHeight = 100.0f;
  float cameraCustomPitch = -4.0f;
  float cameraCustomDistance = 270.0f;
  float cameraCustomStiffness = 0.0f;
  float cameraCustomSwivelSpeed = 1.0f;
  float cameraCustomTransitionSpeed = 1.0f;
  int recordingFps = 60;
  int recordingPlaybackRateIndex = 1;
  bool recordingFinishBeforeDump = false;
  bool recordingActive = false;
  int recordingSnapshotCount = 0;
  size_t recordingLastBytes = 0;
  std::string recordingStatus = "Idle";
  std::chrono::steady_clock::time_point recordingStartedAt = std::chrono::steady_clock::time_point{};
  int touchControlsMode = 1;
  float touchMarkerDecaySeconds = 5.0f;
  bool touchBreakdownKind = false;
  bool touchBreakdownHeight = false;
  bool touchBreakdownSurface = false;
  bool touchBreakdownDodge = false;
  bool movementBreakdownSpeed = false;
  bool movementBreakdownHeight = false;
  bool possessionBreakdownState = false;
  bool possessionBreakdownThird = false;
  bool boostPickupPadBig = true;
  bool boostPickupPadSmall = true;
  bool boostPickupPadAmbiguous = true;
  bool boostPickupActivityActive = true;
  bool boostPickupActivityInactive = true;
  bool boostPickupActivityUnknown = true;
  bool boostPickupFieldOwn = true;
  bool boostPickupFieldOpponent = true;
  bool boostPickupFieldUnknown = true;
  int graphInspectorView = 0;
  int mechanicsReviewIndex = 0;
  std::string selectedGraphOutput;
  std::string selectedAnalysisNode;
  std::string graphInspectorNodeQuery;
  uint32_t nextUiStatsWindowId = 1;
  int nextUiWindowZIndex = 1;
  std::deque<UiEventRecord> recentUiEvents;
  std::unordered_map<std::string, int> mechanicsReviewDecisions;
  std::vector<UiStatsWindow> uiStatsWindows;
  UiWindowPlacement launcherPlacement;
  UiWindowPlacement scoreboardPlacement;
  UiWindowPlacement eventsPlacement;
  UiWindowPlacement statusPlacement;
  UiWindowPlacement cameraPlacement;
  UiWindowPlacement playbackControlsPlacement;
  UiWindowPlacement recordingPlacement;
  UiWindowPlacement graphInspectorPlacement;
  UiWindowPlacement eventPlaylistPlacement;
  UiWindowPlacement mechanicsReviewPlacement;
  UiWindowPlacement replayLoadingPlacement;
  UiWindowPlacement moduleControlsPlacement;
  UiWindowPlacement touchControlsPlacement;
  UiWindowPlacement boostPickupControlsPlacement;
  std::vector<std::string> cachedStatsModuleNames;
  std::chrono::steady_clock::time_point nextStatsModuleNamesRefresh =
      std::chrono::steady_clock::time_point{};
  std::chrono::steady_clock::time_point nextUiConfigAutosave =
      std::chrono::steady_clock::time_point{};
  std::string lastSavedUiConfigJson;

  bool loadRustLibrary();
  void unloadRustLibrary();
  void tick(std::string eventName);
  void scheduleLiveTick(float delaySeconds = 0.25f);
  bool liveProcessingEnabled();
  bool replayAnnotationsEnabled();
  float sampleIntervalSeconds();
  int overlayX();
  int overlayY();
  float overlayScale();
  float overlayMessageSeconds();
  int overlayMaxMessages();
  bool profileTimingEnabled();
  uint64_t profileLogEvery();
  void recordProfileTiming(double samplingMs, double processingMs, double drainMs);
  void resetProfileTiming();
  void render(CanvasWrapper canvas);
  void renderLauncherWindow();
  void renderScoreboardWindow();
  void renderEventsWindow();
  void renderEventSourceControls();
  void renderStatusWindow();
  void renderCameraWindow();
  void renderPlaybackControlsWindow();
  void renderRecordingWindow();
  void renderGraphInspectorWindow();
  void renderEventPlaylistWindow();
  void renderMechanicsReviewWindow();
  bool renderEventFilterCombo(const char *label);
  void renderReplayLoadingWindow();
  void renderModuleControlsWindow();
  void renderTouchControlsWindow();
  void renderBoostPickupControlsWindow();
  void renderSingletonWindowManager();
  void renderStatsWindowManager();
  void resetWindowPlacements();
  void resetDefaultStatsWindows();
  void applyDefaultUiWorkspace();
  void applyReplayReviewUiWorkspace();
  void applyGraphDebugUiWorkspace();
  void applyRecordingUiWorkspace();
  void createStatsWindow(UiStatsWindowKind kind, bool initializeEntries = false);
  void createStatsModuleWindow(std::string moduleName, int moduleView = 0);
  void initializeStatsWindowPlacement(UiStatsWindow &window);
  void applyWindowPlacement(
      UiWindowPlacement &placement,
      float x,
      float y,
      float width,
      float height);
  void captureWindowPlacement(UiWindowPlacement &placement);
  bool renderSingletonWindowHeader(const char *label, bool &open);
  void applyStatsWindowPlacement(UiStatsWindow &window);
  void captureStatsWindowPlacement(UiStatsWindow &window);
  void renderStatsWindows();
  void renderStatsWindow(UiStatsWindow &window);
  void renderStatsWindowScopeSelector(UiStatsWindow &window);
  void renderStatsWindowAddControl(UiStatsWindow &window);
  void renderStatsWindowEntries(UiStatsWindow &window);
  void renderPlayerStatsTable(UiStatsWindow &window, const SaPlayerFrame &player);
  void renderTeamStatsTable(UiStatsWindow &window, uint8_t isTeam0);
  void renderAllPlayersStatsTable(UiStatsWindow &window);
  void renderAllTeamsStatsTable(UiStatsWindow &window);
  void renderGoalsOverviewStats(UiStatsWindow &window);
  void renderAdHocStatsWindow(UiStatsWindow &window);
  void renderStatsModuleWindow(UiStatsWindow &window);
  void renderJsonSummary(const std::string &json);
  void renderJsonInspectorPayload(const char *id, const std::string &label, const std::string &json);
  std::vector<std::string> graphOutputNames();
  std::vector<std::string> analysisNodeNames();
  const char *statsWindowKindLabel(UiStatsWindowKind kind) const;
  std::string statsWindowDisplayLabel(const UiStatsWindow &window) const;
  std::string statsWindowTitle(const UiStatsWindow &window) const;
  const SaPlayerFrame *sampledPlayerByIndex(uint32_t playerIndex) const;
  void initializeStatsWindowEntries(UiStatsWindow &window);
  bool statsWindowSupportsStat(const UiStatsWindow &window, std::string_view statId) const;
  bool statsWindowHasStat(
      const UiStatsWindow &window,
      std::string_view statId,
      std::string_view targetId = {}) const;
  const std::vector<std::string> &statsModuleNames();
  std::string playerStatValue(const SaPlayerFrame &player, std::string_view statId) const;
  std::string teamStatValue(uint8_t isTeam0, std::string_view statId) const;
  std::string defaultAdHocTargetId(std::string_view statId) const;
  std::string adHocStatValue(std::string_view statId, std::string_view targetId) const;
  std::string webUiStatIdForWindow(
      const UiStatsWindow &window,
      const UiStatsWindow::Entry &entry) const;
  void renderAdHocTargetSelector(
      UiStatsWindow &window,
      UiStatsWindow::Entry &entry,
      std::string_view statId,
      size_t index);
  std::filesystem::path uiConfigPath() const;
  std::string uiConfigJson() const;
  void applyUiConfigJson(const std::string &json, std::string_view sourceLabel);
  void loadUiConfig();
  void saveUiConfig();
  void maybeAutosaveUiConfig();
  int recentEventCountForActor(std::string_view actor) const;
  int recentEventCountForTeam(uint8_t isTeam0) const;
  int recentEventCountForType(std::string_view type) const;
  void renderSharedSettingsControls();
  bool uiEnabled();
  bool cvarBool(const char *name, bool defaultValue) const;
  void setCvarBool(const char *name, bool value);
  std::string cvarString(const char *name, std::string_view defaultValue) const;
  void setCvarString(const char *name, std::string_view value);
  void appendUiEvent(UiEventRecord event);
  bool uiEventVisible(const UiEventRecord &event);
  bool eventPlaylistSourceEnabled(const UiEventRecord &event) const;
  std::string mechanicsReviewKey(const UiEventRecord &event) const;
  const char *mechanicsReviewDecisionLabel(const UiEventRecord &event) const;
  void tickReplayAnnotations();
  void resetReplayAnnotations();
  std::optional<std::string> currentReplayPath(ReplayServerWrapper replayServer);
  std::string readJsonBuffer(JsonLen len, WriteJson write);
  std::string readNamedJsonBuffer(
      NamedJsonLen len,
      WriteNamedJson write,
      const std::string &name);
  void dumpGraphJson(std::vector<std::string> params);
  void dumpStatsModuleJson(std::vector<std::string> params);
  void dumpStatsModuleFrameJson(std::vector<std::string> params);
  void dumpStatsModuleConfigJson(std::vector<std::string> params);
  void dumpGraphOutputJson(std::vector<std::string> params);
  void dumpAnalysisNodeJson(std::vector<std::string> params);
  void verifyGraphRuntime(std::vector<std::string> params);
  void selfTestGraphRuntime(std::vector<std::string> params);
  void pushEventMessage(const SaMechanicEvent &event);
  void pushTeamEventMessage(const SaTeamEvent &event);
  void pushGoalContextEventMessage(const SaGoalContextEvent &event);
  bool overlayCategoryEnabled(std::string_view category);
  bool overlayMechanicEnabled(SaMechanicKind kind);
  std::string playerLabel(uint32_t playerIndex, uint8_t isTeam0) const;
  std::string teamLabel(uint8_t isTeam0) const;
  bool finishAndDrainPendingEvents(std::string_view context);
  void drainPendingEvents();
  SaLiveFrame sampleFrame();
  void samplePlayers(ServerWrapper server, CarWrapper localCar);
  SaRigidBody sampleRigidBody(ActorWrapper actor);
  SaPlayerFrame samplePlayer(CarWrapper car, uint32_t playerIndex);
  SaPlayerFrame samplePlayer(PriWrapper pri, uint32_t playerIndex);
  void populatePlayerFromPri(SaPlayerFrame &player, PriWrapper pri, uint32_t fallbackIndex);
  void hookGameEvents();
  void unhookGameEvents();
  void resetLiveState();
  void clearPendingFrameEvents();
  void commitPendingFrameEvents();
  void attachPendingFrameEvents(SaLiveFrame &frame);
  SaEventTiming currentEventTiming();
  void recordTouch(CarWrapper car);
  void recordDodgeRefreshFromJumpState(CarWrapper car, uint32_t playerIndex, uint8_t isTeam0);
  void recordBoostPadEvent(ActorWrapper pickup, SaBoostPadEventKind kind);
  void recordGoal(ServerWrapper server, GoalWrapper goal, int scoreIndex, int assistIndex);
  void recordDemolish(CarWrapper victim, ActorWrapper demolisher);
  void recordPlayerStatDeltas(PriWrapper pri, uint32_t playerIndex, uint8_t isTeam0);
  void recordExplicitPlayerStat(PriWrapper pri, SaPlayerStatEventKind kind);
  std::optional<uint32_t> playerIndexForCar(CarWrapper car);
  std::optional<uint32_t> playerIndexForPri(PriWrapper pri);
  PriWrapper priForScoreIndex(ServerWrapper server, int scoreIndex);
  std::optional<uint32_t> playerIndexForScoreIndex(ServerWrapper server, int scoreIndex);
  std::optional<uint32_t> playerIndexForNearestCar(ActorWrapper actor, float maxDistance);
  uint32_t stablePlayerIndexForPri(PriWrapper pri, uint32_t fallbackIndex);
  uint32_t boostPadId(ActorWrapper pickup);
  void sampleTeamScores(ServerWrapper server, SaLiveFrame &frame);
  void sampleTeamScores(ServerWrapper server, SaGoalEvent &goal);
  std::optional<bool> scoringTeamFromScoreDelta(const SaGoalEvent &goal) const;
  void rememberTeamScores(const SaLiveFrame &frame);
  void rememberTeamScores(const SaGoalEvent &goal);
  bool goalEventIsDuplicate(const SaGoalEvent &goal) const;
};
