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
    # The crate's [lib] name is "tem_recorder", so the built cdylib is
    # target/release/tem_recorder.dll (the short name the plugin loads).
    cargo build -p subtr-actor-tem-recorder --release
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
        -Path (Join-Path $RepoRoot "target/release/tem_recorder.dll") `
        -Destination (Join-Path $PluginOutDir "tem_recorder.dll")

    $InstallLayoutDir = Join-Path $PluginOutDir "bakkesmod-install"
    $InstallLayoutPluginsDir = Join-Path $InstallLayoutDir "plugins"
    $InstallLayoutDataDir = Join-Path $InstallLayoutDir "data\tem-recorder"
    New-Item -ItemType Directory -Force -Path $InstallLayoutPluginsDir | Out-Null
    New-Item -ItemType Directory -Force -Path $InstallLayoutDataDir | Out-Null
    Copy-Item `
        -Force `
        -Path (Join-Path $PluginOutDir "TemRecorderPlugin.dll") `
        -Destination (Join-Path $InstallLayoutPluginsDir "TemRecorderPlugin.dll")
    Copy-Item `
        -Force `
        -Path (Join-Path $PluginOutDir "tem_recorder.dll") `
        -Destination (Join-Path $InstallLayoutDataDir "tem_recorder.dll")

    Write-Host "Built plugin artifacts in $PluginOutDir"
    Write-Host "Prepared install layout in $InstallLayoutDir"

    if ($Install) {
        $BakkesModPluginsDir = Join-Path $BakkesModRoot "plugins"
        $BakkesModDataDir = Join-Path $BakkesModRoot "data\tem-recorder"
        $BakkesModPluginsCfg = Join-Path $BakkesModRoot "cfg\plugins.cfg"
        if (-not (Test-Path $BakkesModPluginsDir)) {
            throw "BakkesMod plugins directory not found: $BakkesModPluginsDir"
        }

        New-Item -ItemType Directory -Force -Path $BakkesModDataDir | Out-Null
        Copy-Item `
            -Force `
            -Path (Join-Path $PluginOutDir "TemRecorderPlugin.dll") `
            -Destination (Join-Path $BakkesModPluginsDir "TemRecorderPlugin.dll")
        Copy-Item `
            -Force `
            -Path (Join-Path $PluginOutDir "tem_recorder.dll") `
            -Destination (Join-Path $BakkesModDataDir "tem_recorder.dll")

        Write-Host "Installed TemRecorderPlugin.dll to $BakkesModPluginsDir"
        Write-Host "Installed tem_recorder.dll to $BakkesModDataDir"

        if ($EnableAutoload) {
            if (-not (Test-Path $BakkesModPluginsCfg)) {
                New-Item -ItemType File -Force -Path $BakkesModPluginsCfg | Out-Null
            }

            $AutoloadCommand = "plugin load TemRecorderPlugin"
            $ExistingAutoload = Get-Content $BakkesModPluginsCfg | Where-Object {
                $_.Trim().ToLowerInvariant() -eq $AutoloadCommand.ToLowerInvariant()
            }
            if (-not $ExistingAutoload) {
                Add-Content -Path $BakkesModPluginsCfg -Value $AutoloadCommand
                Write-Host "Added TemRecorderPlugin to $BakkesModPluginsCfg"
            }
        }
    }
}
finally {
    Pop-Location
}
