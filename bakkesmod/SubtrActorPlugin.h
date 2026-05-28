#pragma once

#include <array>
#include <chrono>
#include <deque>
#include <filesystem>
#include <initializer_list>
#include <limits>
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
  using DecodedStatsPlayerConfigJsonLen = size_t (*)(const char *);
  using WriteDecodedStatsPlayerConfigJson = size_t (*)(const char *, uint8_t *, size_t);
  using EncodedStatsPlayerConfigLen = size_t (*)(const char *);
  using WriteEncodedStatsPlayerConfig = size_t (*)(const char *, uint8_t *, size_t);
  using DrainEvents = size_t (*)(SaEngine *, SaMechanicEvent *, size_t);
  using DrainTeamEvents = size_t (*)(SaEngine *, SaTeamEvent *, size_t);
  using DrainGoalContextEvents = size_t (*)(SaEngine *, SaGoalContextEvent *, size_t);
  using ReplayAnnotationsCreate = SaReplayAnnotations *(*)(const char *);
  using ReplayAnnotationsDestroy = void (*)(SaReplayAnnotations *);
  using ReplayAnnotationCount = size_t (*)(const SaReplayAnnotations *);
  using ReplayAnnotationPlayerCount = size_t (*)(const SaReplayAnnotations *);
  using WriteReplayAnnotationPlayers =
      size_t (*)(const SaReplayAnnotations *, SaReplayPlayerInfo *, size_t);
  using ReplayAnnotationScoreAtTime =
      int32_t (*)(const SaReplayAnnotations *, float, SaReplayScore *);
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
    std::string config_id;
    UiStatsWindowKind kind = UiStatsWindowKind::Player;
    bool open = true;
    bool pending_focus = false;
    bool picker_open = false;
    uint32_t selected_player_index = 0;
    std::string selected_player_id;
    uint8_t selected_team_is_team_0 = 1;
    std::string module_name;
    int module_view = 0;
    std::string picker_query;
    std::vector<Entry> entries;
    bool has_placement = false;
    bool pending_apply_placement = false;
    float x = 0.0f;
    float y = 0.0f;
    float width = 0.0f;
    float height = 0.0f;
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

  static constexpr uint8_t UI_STAT_SCOPE_PLAYER = static_cast<uint8_t>(1u << 0);
  static constexpr uint8_t UI_STAT_SCOPE_TEAM = static_cast<uint8_t>(1u << 1);
  static constexpr uint8_t UI_STAT_SCOPE_EVENT = static_cast<uint8_t>(1u << 2);

  struct SingletonWindowControl {
    const char *label;
    const char *config_id;
    const char *legacy_open_key;
    const char *legacy_placement_key;
    bool web_config;
    int launcher_order;
    bool *open;
    UiWindowPlacement *placement;
    float x;
    float y;
    float width;
    float height;
  };

  struct StatsWindowKindControl {
    UiStatsWindowKind kind;
    const char *config_id;
    const char *label;
    const char *create_label;
    uint8_t stat_scopes;
    bool scope_selector;
    bool stat_picker;
    bool web_config;
    bool default_window;
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
  DecodedStatsPlayerConfigJsonLen decodedStatsPlayerConfigJsonLen = nullptr;
  WriteDecodedStatsPlayerConfigJson writeDecodedStatsPlayerConfigJson = nullptr;
  EncodedStatsPlayerConfigLen encodedStatsPlayerConfigLen = nullptr;
  WriteEncodedStatsPlayerConfig writeEncodedStatsPlayerConfig = nullptr;
  DrainEvents drainEvents = nullptr;
  DrainTeamEvents drainTeamEvents = nullptr;
  DrainGoalContextEvents drainGoalContextEvents = nullptr;
  ReplayAnnotationsCreate replayAnnotationsCreate = nullptr;
  ReplayAnnotationsDestroy replayAnnotationsDestroy = nullptr;
  ReplayAnnotationCount replayAnnotationCount = nullptr;
  ReplayAnnotationPlayerCount replayAnnotationPlayerCount = nullptr;
  WriteReplayAnnotationPlayers writeReplayAnnotationPlayers = nullptr;
  ReplayAnnotationScoreAtTime replayAnnotationScoreAtTime = nullptr;
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
  std::unordered_map<uint32_t, std::string> playerUniqueIdsByIndex;
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
  bool uiLauncherOpen = false;
  bool uiScoreboardOpen = true;
  bool uiEventsOpen = false;
  bool uiStatusOpen = false;
  bool uiCameraOpen = true;
  bool uiPlaybackControlsOpen = false;
  bool uiRecordingOpen = false;
  bool uiGraphInspectorOpen = false;
  bool uiEventPlaylistOpen = false;
  bool uiMechanicsReviewOpen = false;
  bool uiReplayLoadingOpen = false;
  bool uiModuleControlsOpen = false;
  bool uiTouchControlsOpen = false;
  bool uiBoostPickupControlsOpen = false;
  bool uiLauncherToggleHovered = false;
  bool eventPlaylistMechanicsEnabled = true;
  bool eventPlaylistTeamEventsEnabled = true;
  bool eventPlaylistGoalContextEnabled = true;
  bool eventPlaylistAutoFollow = true;
  std::string eventPlaylistSourceFilter = "default";
  std::string eventPlaylistLastActiveKey;
  bool timelineRangeBoostEnabled = false;
  bool timelineRangePossessionEnabled = false;
  bool timelineRangePressureEnabled = false;
  bool timelineRangeRushEnabled = false;
  bool timelineRangeAbsolutePositioningEnabled = false;
  bool renderEffectCeilingShotEnabled = false;
  bool renderEffectFiftyFiftyEnabled = false;
  bool renderEffectPressureEnabled = false;
  bool renderEffectRelativePositioningEnabled = false;
  bool renderEffectAbsolutePositioningEnabled = false;
  bool renderEffectSpeedFlipEnabled = false;
  bool renderEffectTouchEnabled = false;
  int cameraViewMode = 0;
  int cameraFreePreset = -1;
  uint32_t cameraSelectedPlayerIndex = 0;
  std::string cameraSelectedPlayerId;
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
  float playbackCurrentTime = 0.0f;
  bool playbackPlaying = false;
  float playbackRate = 1.0f;
  bool playbackSkipPostGoalTransitions = true;
  bool playbackSkipKickoffs = false;
  int recordingFps = 60;
  int recordingPlaybackRateIndex = 1;
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
  bool boostPickupAnimationEnabled = false;
  bool boostPickupActivityActive = true;
  bool boostPickupActivityInactive = true;
  bool boostPickupActivityUnknown = true;
  bool boostPickupFieldOwn = true;
  bool boostPickupFieldOpponent = true;
  bool boostPickupFieldUnknown = true;
  bool boostPickupPlayerFilterEnabled = false;
  std::vector<std::string> boostPickupPlayerIds;
  int graphInspectorView = 0;
  int mechanicsReviewIndex = 0;
  float mechanicsReviewClipLeadSeconds = 2.0f;
  float mechanicsReviewClipTrailSeconds = 2.0f;
  bool mechanicsReviewClipActive = false;
  float mechanicsReviewClipStartSeconds = 0.0f;
  float mechanicsReviewClipEndSeconds = 0.0f;
  std::string mechanicsReviewStatus;
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
  mutable uint64_t cachedStatsJsonFrameNumber = std::numeric_limits<uint64_t>::max();
  mutable std::string cachedStatsJson;
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
  void renderLauncherToggleChrome();
  void applyLauncherMenuPlacement();
  void renderLauncherWindow();
  void renderLauncherWorkspaceControls();
  void renderWebWindowToggleControls(
      const char *idSuffix,
      bool closeLauncherOnToggle,
      bool includeState = true,
      bool fullWidth = false);
  void renderStatsWindowCreationControls(
      const char *idSuffix,
      bool closeLauncherOnCreate,
      bool includeHeading = true,
      bool includeManager = true,
      bool fullWidth = false);
  void renderSettingsWindowControls();
  void renderEmptyStateWindow();
  void renderFloatingWindowLayer();
  void renderScoreboardWindow();
  void renderEventsWindow();
  void renderEventSourceControls();
  void renderStatusWindow();
  void renderCameraWindow();
  void applyPlaybackConfigToReplay(std::string_view sourceLabel);
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
  bool renderModuleSummaryToggle(
      const char *label,
      bool active,
      const char *idSuffix,
      float width = 230.0f);
  void renderCvarModuleSummaryToggle(
      const char *label,
      const char *name,
      bool defaultValue,
      const char *idSuffix,
      float width = 230.0f);
  void renderEventFilterModuleSummaryToggle(
      const char *label,
      const char *token,
      const char *idSuffix,
      float width = 230.0f);
  void renderBoolModuleSummaryToggle(
      const char *label,
      bool &active,
      const char *idSuffix,
      float width = 230.0f);
  void renderModuleSummaryControls(
      const char *idSuffix,
      bool collapsibleGroups = true,
      float toggleWidth = 230.0f,
      bool includePluginControls = true);
  void renderModuleSettingsControls(
      const char *idSuffix,
      bool includeOpenButtons,
      bool webCardHeaders = false,
      bool onlyWebActivePanels = false);
  std::array<SingletonWindowControl, 13> singletonWindowControls();
  std::vector<SingletonWindowControl> webSingletonWindowControls();
  std::array<StatsWindowKindControl, 7> statsWindowKindControls() const;
  void renderSingletonWindowManager();
  void renderStatsWindowManager();
  void focusTopLoadedWindow();
  void resetWindowPlacements();
  void resetDefaultStatsWindows();
  void applyWorkspaceWindowVisibility(
      bool launcherOpen,
      std::initializer_list<std::string_view> openWindowIds);
  void applyDefaultUiWorkspace();
  void applyReplayReviewUiWorkspace();
  void applyGraphDebugUiWorkspace();
  void applyRecordingUiWorkspace();
  void createStatsWindow(UiStatsWindowKind kind, bool initializeEntries = false);
  void createStatsModuleWindow(std::string moduleName, int moduleView = 0);
  std::pair<float, float> defaultStatsWindowSize(UiStatsWindowKind kind) const;
  std::pair<float, float> defaultStatsWindowPosition(size_t stackIndex) const;
  void initializeStatsWindowPlacement(UiStatsWindow &window);
  void resetStatsWindowPlacement(UiStatsWindow &window, size_t stackIndex);
  void showStatsWindow(UiStatsWindow &window);
  void focusStatsWindow(UiStatsWindow &window);
  void resetSingletonWindowPlacement(
      UiWindowPlacement &placement,
      float x,
      float y,
      float width,
      float height,
      bool focus = false);
  void resetScoreboardWindowPlacement(bool focus = false);
  void showSingletonWindow(bool &open, UiWindowPlacement &placement);
  void hideSingletonWindow(bool &open);
  void hideStatsWindow(UiStatsWindow &window);
  void focusSingletonWindow(UiWindowPlacement &placement);
  void showLauncherWindow();
  void hideLauncherWindow();
  void applyWindowPlacement(
      UiWindowPlacement &placement,
      float x,
      float y,
      float width,
      float height);
  void applySingletonWindowPlacement(UiWindowPlacement &placement);
  void applyScoreboardWindowPlacement();
  void captureWindowPlacement(UiWindowPlacement &placement);
  bool renderSingletonWindowHeader(const char *label, bool &open);
  void applyStatsWindowPlacement(UiStatsWindow &window);
  void captureStatsWindowPlacement(UiStatsWindow &window);
  void renderStatsWindow(UiStatsWindow &window, size_t stackIndex);
  void renderStatsWindowScopeSelector(UiStatsWindow &window);
  void renderStatsWindowAddControl(UiStatsWindow &window);
  void renderStatsWindowEntries(UiStatsWindow &window);
  bool renderStatsWindowValueRow(
      UiStatsWindow &window,
      size_t entryIndex,
      std::string_view label,
      std::string_view value,
      std::string_view idSuffix = {});
  void renderMissingStatsRows(UiStatsWindow &window);
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
  std::optional<UiStatsWindowKind> parseStatsWindowKind(std::string_view value) const;
  const char *statsWindowKindConfigId(UiStatsWindowKind kind) const;
  const char *statsWindowKindLabel(UiStatsWindowKind kind) const;
  uint8_t statsWindowKindStatScopes(UiStatsWindowKind kind) const;
  bool statsWindowKindHasScopeSelector(UiStatsWindowKind kind) const;
  bool statsWindowKindHasStatPicker(UiStatsWindowKind kind) const;
  std::string statsWindowDisplayLabel(const UiStatsWindow &window) const;
  std::string statsWindowTitle(const UiStatsWindow &window) const;
  const SaPlayerFrame *sampledPlayerByIndex(uint32_t playerIndex) const;
  void initializeStatsWindowEntries(UiStatsWindow &window);
  bool statsWindowSupportsStat(const UiStatsWindow &window, std::string_view statId) const;
  bool statsWindowHasStat(
      const UiStatsWindow &window,
      std::string_view statId,
      std::string_view targetId = {}) const;
  bool statsWindowTargetsEqual(
      std::string_view statId,
      std::string_view lhsTargetId,
      std::string_view rhsTargetId) const;
  const std::vector<std::string> &statsModuleNames();
  std::string playerStatValue(const SaPlayerFrame &player, std::string_view statId) const;
  std::string teamStatValue(uint8_t isTeam0, std::string_view statId) const;
  const std::string &currentStatsJson() const;
  std::optional<std::string> graphPlayerStatValue(
      const SaPlayerFrame &player,
      std::string_view statId) const;
  std::optional<std::string> graphTeamStatValue(uint8_t isTeam0, std::string_view statId) const;
  std::string defaultAdHocTargetId(std::string_view statId) const;
  std::string adHocStatValue(std::string_view statId, std::string_view targetId) const;
  std::string webUiStatIdForWindow(
      const UiStatsWindow &window,
      const UiStatsWindow::Entry &entry) const;
  std::optional<uint32_t> playerIndexForTargetId(std::string_view targetId) const;
  std::string webPlayerIdForIndex(uint32_t playerIndex) const;
  std::optional<std::string> webPlayerIdForIndexIfKnown(uint32_t playerIndex) const;
  std::string webPlayerIdForWindow(const UiStatsWindow &window) const;
  std::optional<std::string> webPlayerIdForWindowConfig(const UiStatsWindow &window) const;
  void resolveStatsWindowPlayerSelection(UiStatsWindow &window);
  std::string webCameraPlayerId() const;
  std::optional<std::string> webCameraPlayerIdConfig() const;
  void resolveCameraPlayerSelection();
  void renderAdHocTargetSelector(
      UiStatsWindow &window,
      UiStatsWindow::Entry &entry,
      std::string_view statId,
      size_t index);
  std::filesystem::path uiConfigPath() const;
  std::string uiConfigJson();
  std::optional<std::string> statsPlayerCfgFromJson(const std::string &json);
  std::optional<std::string> statsPlayerCfgJsonFromClipboard(std::string_view clipboardText);
  void applyUiConfigParams(std::vector<std::string> params);
  void applyUiConfigJson(const std::string &json, std::string_view sourceLabel);
  void loadUiConfig();
  void saveUiConfig();
  void scheduleUiConfigAutosave(
      std::chrono::milliseconds delay = std::chrono::milliseconds(150));
  void maybeAutosaveUiConfig();
  void renderLayoutConfigControls(const char *idSuffix, bool fullWidth = false);
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
  void importReplayAnnotationPlayers();
  void tickMechanicsReviewClipBoundary();
  void resetReplayAnnotations();
  std::optional<std::string> currentReplayPath(ReplayServerWrapper replayServer);
  std::string readJsonBuffer(JsonLen len, WriteJson write) const;
  std::string readNamedJsonBuffer(
      NamedJsonLen len,
      WriteNamedJson write,
      const std::string &name) const;
  void dumpGraphJson(std::vector<std::string> params);
  void dumpStatsModuleJson(std::vector<std::string> params);
  void dumpStatsModuleFrameJson(std::vector<std::string> params);
  void dumpStatsModuleConfigJson(std::vector<std::string> params);
  void dumpGraphOutputJson(std::vector<std::string> params);
  void dumpAnalysisNodeJson(std::vector<std::string> params);
  void verifyGraphRuntime(std::vector<std::string> params);
  void selfTestGraphRuntime(std::vector<std::string> params);
  void pushGoalEventMessage(const SaGoalEvent &event);
  void pushEventMessage(const SaMechanicEvent &event);
  void pushTeamEventMessage(const SaTeamEvent &event);
  void pushGoalContextEventMessage(const SaGoalContextEvent &event);
  void pushPlayerStatEventMessage(const SaPlayerStatEvent &event);
  std::optional<std::pair<int32_t, int32_t>> currentScoreboardScore() const;
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
