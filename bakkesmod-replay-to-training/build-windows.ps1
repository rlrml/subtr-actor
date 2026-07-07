param(
    [string]$BuildDir = "",
    [string]$Configuration = "Release",
    [string]$BakkesModSdkDir = "",
    [switch]$Install,
    [switch]$EnableAutoload,
    [string]$BakkesModRoot = ""
)

$ErrorActionPreference = "Stop"

$ScriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
$RepoRoot = Resolve-Path (Join-Path $ScriptDir "..")

if ($BuildDir -eq "") {
    $BuildDir = Join-Path $ScriptDir "build"
}

if ($BakkesModRoot -eq "") {
    $BakkesModRoot = Join-Path $env:APPDATA "bakkesmod\bakkesmod"
}

Push-Location $RepoRoot
try {
    # The crate's [lib] name is "replay_to_training", so the built cdylib is
    # target/release/replay_to_training.dll (the short name the plugin loads).
    # Build identification (REPLAY_TO_TRAINING_GIT_HASH / _GIT_DIRTY /
    # _COMMIT_DATE) needs no plumbing here: build.rs and the CMake configure
    # both derive it from git when the env vars are unset, and this script
    # always runs from a checkout. Set the env vars to override.
    cargo build -p subtr-actor-replay-to-training --release
    if ($LASTEXITCODE -ne 0) {
        throw "cargo build failed with exit code $LASTEXITCODE"
    }

    $cmakeArgs = @(
        "-S", $ScriptDir,
        "-B", $BuildDir,
        "-G", "Visual Studio 17 2022",
        "-A", "x64"
    )

    if ($BakkesModSdkDir -ne "") {
        $cmakeArgs += "-DBAKKESMOD_SDK_DIR=$BakkesModSdkDir"
    }

    cmake @cmakeArgs
    if ($LASTEXITCODE -ne 0) {
        throw "cmake configure failed with exit code $LASTEXITCODE"
    }

    cmake --build $BuildDir --config $Configuration
    if ($LASTEXITCODE -ne 0) {
        throw "cmake build failed with exit code $LASTEXITCODE"
    }

    $PluginOutDir = Join-Path $BuildDir $Configuration
    New-Item -ItemType Directory -Force -Path $PluginOutDir | Out-Null
    Copy-Item `
        -Force `
        -Path (Join-Path $RepoRoot "target/release/replay_to_training.dll") `
        -Destination (Join-Path $PluginOutDir "replay_to_training.dll")

    $InstallLayoutDir = Join-Path $PluginOutDir "bakkesmod-install"
    $InstallLayoutPluginsDir = Join-Path $InstallLayoutDir "plugins"
    $InstallLayoutDataDir = Join-Path $InstallLayoutDir "data\replay-to-training"
    New-Item -ItemType Directory -Force -Path $InstallLayoutPluginsDir | Out-Null
    New-Item -ItemType Directory -Force -Path $InstallLayoutDataDir | Out-Null
    Copy-Item `
        -Force `
        -Path (Join-Path $PluginOutDir "ReplayToTrainingPlugin.dll") `
        -Destination (Join-Path $InstallLayoutPluginsDir "ReplayToTrainingPlugin.dll")
    Copy-Item `
        -Force `
        -Path (Join-Path $PluginOutDir "replay_to_training.dll") `
        -Destination (Join-Path $InstallLayoutDataDir "replay_to_training.dll")

    Write-Host "Built plugin artifacts in $PluginOutDir"
    Write-Host "Prepared install layout in $InstallLayoutDir"

    if ($Install) {
        $BakkesModPluginsDir = Join-Path $BakkesModRoot "plugins"
        $BakkesModDataDir = Join-Path $BakkesModRoot "data\replay-to-training"
        $BakkesModPluginsCfg = Join-Path $BakkesModRoot "cfg\plugins.cfg"
        if (-not (Test-Path $BakkesModPluginsDir)) {
            throw "BakkesMod plugins directory not found: $BakkesModPluginsDir"
        }

        New-Item -ItemType Directory -Force -Path $BakkesModDataDir | Out-Null
        Copy-Item `
            -Force `
            -Path (Join-Path $PluginOutDir "ReplayToTrainingPlugin.dll") `
            -Destination (Join-Path $BakkesModPluginsDir "ReplayToTrainingPlugin.dll")
        Copy-Item `
            -Force `
            -Path (Join-Path $PluginOutDir "replay_to_training.dll") `
            -Destination (Join-Path $BakkesModDataDir "replay_to_training.dll")

        Write-Host "Installed ReplayToTrainingPlugin.dll to $BakkesModPluginsDir"
        Write-Host "Installed replay_to_training.dll to $BakkesModDataDir"

        if ($EnableAutoload) {
            if (-not (Test-Path $BakkesModPluginsCfg)) {
                New-Item -ItemType File -Force -Path $BakkesModPluginsCfg | Out-Null
            }

            $AutoloadCommand = "plugin load ReplayToTrainingPlugin"
            $ExistingAutoload = Get-Content $BakkesModPluginsCfg | Where-Object {
                $_.Trim().ToLowerInvariant() -eq $AutoloadCommand.ToLowerInvariant()
            }
            if (-not $ExistingAutoload) {
                Add-Content -Path $BakkesModPluginsCfg -Value $AutoloadCommand
                Write-Host "Added ReplayToTrainingPlugin to $BakkesModPluginsCfg"
            }
        }
    }
}
finally {
    Pop-Location
}
