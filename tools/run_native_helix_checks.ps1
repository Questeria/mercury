param(
    [string]$HelixRoot = "C:\Projects\Helix"
)

$ErrorActionPreference = "Stop"

$MercuryRoot = (Resolve-Path (Join-Path $PSScriptRoot "..")).Path
$BootstrapRoot = Join-Path $HelixRoot "stage0\helixc-bootstrap"

function Convert-ToWslPath {
    param([string]$WindowsPath)

    $resolved = (Resolve-Path -LiteralPath $WindowsPath).Path
    $drive = $resolved.Substring(0, 1).ToLowerInvariant()
    $rest = $resolved.Substring(2).Replace("\", "/")
    return "/mnt/$drive$rest"
}

if (-not (Get-Command wsl.exe -ErrorAction SilentlyContinue)) {
    Write-Host "SKIP: wsl.exe not found; from-raw Helix cross-check needs WSL"
    exit 0
}

if (-not (Test-Path -LiteralPath $BootstrapRoot)) {
    Write-Host "SKIP: Helix bootstrap not found: $BootstrapRoot"
    exit 0
}

Push-Location $MercuryRoot
try {
    python .\tools\gen_policy_attestations.py --check

    $wslMercury = Convert-ToWslPath $MercuryRoot
    $wslBootstrap = Convert-ToWslPath $BootstrapRoot

    Write-Host "Running from-raw Helix cross-check."
    Write-Host "Helix access: read-only copy to /tmp"
    wsl.exe bash -lc "cd '$wslMercury' && bash tools/run_fromraw_helix_check.sh '$wslBootstrap'"
    if ($LASTEXITCODE -ne 0) {
        throw "from-raw Helix cross-check failed with exit code $LASTEXITCODE"
    }
}
finally {
    Pop-Location
}
