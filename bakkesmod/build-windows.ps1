param(
    [string]$BuildDir = "",
    [string]$Configuration = "Release",
    [string]$BakkesModSdkDir = ""
)

$ErrorActionPreference = "Stop"

$ScriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
$RepoRoot = Resolve-Path (Join-Path $ScriptDir "..")

if ($BuildDir -eq "") {
    $BuildDir = Join-Path $ScriptDir "build"
}

Push-Location $RepoRoot
try {
    cargo build -p subtr-actor-bakkesmod --release

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
    cmake --build $BuildDir --config $Configuration

    $PluginOutDir = Join-Path $BuildDir $Configuration
    New-Item -ItemType Directory -Force -Path $PluginOutDir | Out-Null
    Copy-Item `
        -Force `
        -Path (Join-Path $RepoRoot "target/release/subtr_actor_bakkesmod.dll") `
        -Destination (Join-Path $PluginOutDir "subtr_actor_bakkesmod.dll")

    Write-Host "Built plugin artifacts in $PluginOutDir"
}
finally {
    Pop-Location
}
