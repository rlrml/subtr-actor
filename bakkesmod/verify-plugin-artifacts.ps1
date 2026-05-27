param(
    [string]$BuildDir = "bakkesmod/build",
    [string]$Configuration = "Release",
    [switch]$SkipRuntimeCheck
)

$ErrorActionPreference = "Stop"

$ScriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
$RepoRoot = Resolve-Path (Join-Path $ScriptDir "..")

function Resolve-RepoPath([string]$Path) {
    if ([System.IO.Path]::IsPathRooted($Path)) {
        return $Path
    }
    return Join-Path $RepoRoot $Path
}

function Invoke-CheckedPython([string[]]$Arguments) {
    python @Arguments
    if ($LASTEXITCODE -ne 0) {
        throw "python $($Arguments -join ' ') failed with exit code $LASTEXITCODE"
    }
}

function Assert-FileExists([string]$Path) {
    if (-not (Test-Path $Path)) {
        throw "Expected artifact not found: $Path"
    }
}

function Assert-SameFileHash([string]$Expected, [string]$Actual, [string]$Label) {
    if ((Get-FileHash $Expected).Hash -ne (Get-FileHash $Actual).Hash) {
        throw "$Label differs from primary artifact"
    }
}

function Read-ArtifactText([string]$Path) {
    $bytes = [System.IO.File]::ReadAllBytes((Resolve-Path $Path))
    return [System.Text.Encoding]::ASCII.GetString($bytes)
}

function Assert-Contains([string]$Text, [string]$Needle, [string]$ArtifactLabel) {
    if (-not $Text.Contains($Needle)) {
        throw "Missing expected $ArtifactLabel text in built artifact: $Needle"
    }
}

function Assert-AllContains([string]$Text, [string[]]$Needles, [string]$ArtifactLabel) {
    foreach ($Needle in $Needles) {
        Assert-Contains $Text $Needle $ArtifactLabel
    }
}

$BuildRoot = Resolve-RepoPath $BuildDir
$ArtifactDir = Join-Path $BuildRoot $Configuration
$PluginDll = Join-Path $ArtifactDir "SubtrActorPlugin.dll"
$RustDll = Join-Path $ArtifactDir "subtr_actor_bakkesmod.dll"
$InstallPluginDll = Join-Path $ArtifactDir "bakkesmod-install/plugins/SubtrActorPlugin.dll"
$InstallRustDll = Join-Path $ArtifactDir "bakkesmod-install/data/subtr-actor/subtr_actor_bakkesmod.dll"

foreach ($Path in @($PluginDll, $RustDll, $InstallPluginDll, $InstallRustDll)) {
    Assert-FileExists $Path
}

Assert-SameFileHash $PluginDll $InstallPluginDll "Install layout plugin DLL"
Assert-SameFileHash $RustDll $InstallRustDll "Install layout Rust DLL"

Push-Location $RepoRoot
try {
    Invoke-CheckedPython @("bakkesmod/verify-rust-dll-exports.py", $RustDll, $InstallRustDll)
    Invoke-CheckedPython @("bakkesmod/verify-plugin-dll-exports.py", $PluginDll, $InstallPluginDll)
    if (-not $SkipRuntimeCheck) {
        Invoke-CheckedPython @("bakkesmod/verify-rust-dll-runtime.py", $RustDll, $InstallRustDll)
    }
}
finally {
    Pop-Location
}

$PluginText = Read-ArtifactText $PluginDll
$PluginNeedles = @(
    "subtr_actor_verify_graph",
    "subtr_actor_self_test_graph",
    "subtr_actor_replay_annotations_enabled",
    "subtr-actor REPLAY",
    "loaded {} replay annotations from normal replay processor",
    "graph self-test fed every required event family",
    "graph self-test derived active_demos from demolish event",
    "graph self-test writing synthetic graph dump",
    "require_event_history",
    "require_graph_events",
    "every callable analysis node",
    "Function TAGame.Ball_TA.OnCarTouch",
    "Function TAGame.VehiclePickup_TA.EventPickedUp",
    "Function TAGame.VehiclePickup_TA.EventSpawned",
    "Function TAGame.GameEvent_Soccar_TA.EventGoalScored",
    "Function TAGame.Car_TA.Demolish",
    "verified {} graph outputs by name",
    "graph_info declares all {} required graph outputs",
    "graph_info missing required graph output",
    "graph_info declares {} graph event fields and {} required graph event fields",
    "graph_info declares all {} known graph event fields",
    "graph_info missing required graph event field",
    "graph_info declares all {} strict graph event fields",
    "graph_info missing strict graph event field",
    "could not read graph event field names from graph_info",
    "could not read required graph event field names from graph_info",
    "required graph event field '{}' is not declared",
    "graph_info declares {} event_history fields and {} required cumulative event fields",
    "graph_info declares all {} known event_history fields",
    "graph_info missing required event_history field",
    "graph_info declares all {} strict cumulative event_history fields",
    "graph_info missing strict cumulative event_history field",
    "could not read event_history field names from graph_info",
    "could not read required event_history field names from graph_info",
    "required event_history field '{}' is not declared",
    "matches fixed ABI",
    "differs from named output",
    "verified {} builtin stats modules by name",
    "callable analysis node registry matches graph_info",
    "resolved analysis graph nodes are callable by name",
    "analysis_nodes output contains {} callable analysis nodes exactly",
    "analysis_nodes output has unexpected node",
    "frame_events_state exposes {} live event fields",
    "frame_events_state missing event field",
    "frame_events_state event field '{}' has {} entries",
    "frame_events_state event field '{}' is not an array",
    "events output field '{}' has {} entries",
    "events output field '{}' is not an array",
    "events output exposes {} graph event fields",
    "events required graph event field '{}' has no entries",
    "events required graph event fields are nonzero",
    "event_history event field '{}' has {} cumulative entries",
    "event_history event field '{}' is not an array",
    "event_history exposes {} cumulative live event fields",
    "required event field '{}' has no cumulative entries",
    "required cumulative event fields are nonzero",
    "touch_events",
    "dodge_refreshed_events",
    "boost_pad_events",
    "player_stat_events",
    "goal_events",
    "demo_events",
    "active_demos",
    "live frame processing failed",
    "live graph finalization failed during",
    "plugin unload",
    "subtr_actor_dump_graph_output <events|frame|timeline|stats|analysis_nodes|event_history|graph_info> [finish]",
    "graph-analysis-nodes.json",
    "graph-event-history.json",
    "analysis_nodes",
    "event_history",
    "subtr_actor_bakkesmod_analysis_node_names_json_len"
)
Assert-AllContains $PluginText $PluginNeedles "plugin"

$RustText = Read-ArtifactText $RustDll
$RustNeedles = @(
    "subtr_actor_bakkesmod_graph_output_json_len",
    "subtr_actor_bakkesmod_write_graph_output_json",
    "subtr_actor_bakkesmod_analysis_node_json_len",
    "subtr_actor_bakkesmod_write_analysis_node_json",
    "subtr_actor_bakkesmod_analysis_node_names_json_len",
    "subtr_actor_bakkesmod_write_analysis_node_names_json",
    "subtr_actor_bakkesmod_decoded_stats_player_config_json_len",
    "subtr_actor_bakkesmod_write_decoded_stats_player_config_json",
    "subtr_actor_bakkesmod_replay_annotations_create",
    "subtr_actor_bakkesmod_replay_annotation_player_count",
    "subtr_actor_bakkesmod_write_replay_annotation_players",
    "subtr_actor_bakkesmod_poll_replay_annotations",
    "callable_analysis_node_names",
    "builtin_analysis_node_names",
    "analysis_nodes",
    "event_history",
    "graph_event_field_names",
    "required_graph_event_field_names",
    "event_history_field_names",
    "required_event_history_field_names",
    "stats_timeline_events"
)
Assert-AllContains $RustText $RustNeedles "Rust ABI"

Write-Host "BakkesMod plugin artifacts passed verification"
