param(
    [string]$HelixRoot = "",
    [switch]$SkipHelix,
    [switch]$RunFromRawHelix
)

$ErrorActionPreference = "Stop"

$MercuryRoot = (Resolve-Path (Join-Path $PSScriptRoot "..")).Path
$VsDevCmd = "C:\Program Files (x86)\Microsoft Visual Studio\18\BuildTools\Common7\Tools\VsDevCmd.bat"

function Invoke-Step {
    param(
        [string]$Name,
        [scriptblock]$Command
    )

    Write-Host ""
    Write-Host "==> $Name"
    & $Command
    Write-Host "OK: $Name"
}

function Invoke-Cargo {
    param([string[]]$CargoArgs)

    if (Test-Path $VsDevCmd) {
        $argLine = $CargoArgs -join " "
        cmd.exe /d /s /c "`"$VsDevCmd`" -arch=x64 -host_arch=x64 >NUL && cargo $argLine"
        if ($LASTEXITCODE -ne 0) {
            throw "cargo $argLine failed with exit code $LASTEXITCODE"
        }
    } else {
        & cargo @CargoArgs
        if ($LASTEXITCODE -ne 0) {
            throw "cargo $($CargoArgs -join ' ') failed with exit code $LASTEXITCODE"
        }
    }
}

function Invoke-Python {
    param([string[]]$PythonArgs)

    $python = Get-Command python -ErrorAction SilentlyContinue
    if (-not $python) {
        $python = Get-Command python3 -ErrorAction SilentlyContinue
    }
    if (-not $python) {
        throw "Neither python nor python3 was found on PATH"
    }

    & $python.Source @PythonArgs
    if ($LASTEXITCODE -ne 0) {
        throw "$($python.Source) $($PythonArgs -join ' ') failed with exit code $LASTEXITCODE"
    }
}

Push-Location $MercuryRoot
try {
    Invoke-Step "Rust formatting" {
        Invoke-Cargo -CargoArgs @("fmt", "--check")
    }

    Invoke-Step "Rust workspace tests" {
        Invoke-Cargo -CargoArgs @("test", "--workspace")
    }

    Invoke-Step "Rust serde decision-view test" {
        Invoke-Cargo -CargoArgs @("test", "-p", "mercury-core", "--features", "serde", "decision_view_serializes_compact_fields")
    }

    Invoke-Step "Python policy contract" {
        Invoke-Python -PythonArgs @(".\tools\check_policy_contract.py")
    }

    Invoke-Step "Policy attestation manifest" {
        Invoke-Python -PythonArgs @(".\tools\gen_policy_attestations.py", "--check")
    }

    Invoke-Step "Python envelope vectors" {
        Invoke-Python -PythonArgs @(".\tools\check_envelope_vectors.py")
    }

    Invoke-Step "Python AI grant vectors" {
        Invoke-Python -PythonArgs @(".\tools\check_ai_grant_vectors.py")
    }

    Invoke-Step "Python AI grant lifecycle vectors" {
        Invoke-Python -PythonArgs @(".\tools\check_ai_grant_lifecycle_vectors.py")
    }

    Invoke-Step "Python room epoch vectors" {
        Invoke-Python -PythonArgs @(".\tools\check_room_epoch_vectors.py")
    }

    Invoke-Step "Python policy pipeline vectors" {
        Invoke-Python -PythonArgs @(".\tools\check_policy_pipeline_vectors.py")
    }

    Invoke-Step "Python relay submit vectors" {
        Invoke-Python -PythonArgs @(".\tools\check_relay_submit_vectors.py")
    }

    if (-not $SkipHelix) {
        Invoke-Step "Helix policy checks" {
            $args = @("-ExecutionPolicy", "Bypass", "-File", ".\tools\run_helix_checks.ps1")
            if (-not [string]::IsNullOrWhiteSpace($HelixRoot)) {
                $args += @("-HelixRoot", $HelixRoot)
            }
            powershell @args
            if ($LASTEXITCODE -ne 0) {
                throw "Helix checks failed with exit code $LASTEXITCODE"
            }
        }
    } else {
        Write-Host ""
        Write-Host "SKIP: Helix policy checks"
    }

    if ($RunFromRawHelix) {
        Invoke-Step "From-raw Helix cross-check" {
            powershell -ExecutionPolicy Bypass -File ".\tools\run_native_helix_checks.ps1"
            if ($LASTEXITCODE -ne 0) {
                throw "From-raw Helix cross-check failed with exit code $LASTEXITCODE"
            }
        }
    }

    Invoke-Step "Git whitespace check" {
        git diff --check
        if ($LASTEXITCODE -ne 0) {
            throw "git diff --check failed with exit code $LASTEXITCODE"
        }
    }

    Invoke-Step "ASCII scan" {
        $rg = Get-Command rg -ErrorAction SilentlyContinue
        if (-not $rg) {
            Write-Host "SKIP: rg not found"
            return
        }

        & $rg.Source -n "[^[:ascii:]]" README.md docs core/rust fixtures tools helix policy vectors
        if ($LASTEXITCODE -eq 0) {
            throw "ASCII scan found non-ASCII text"
        }
        if ($LASTEXITCODE -ne 1) {
            throw "ASCII scan failed with exit code $LASTEXITCODE"
        }
    }

    Write-Host ""
    Write-Host "Mercury preflight: OK"
}
finally {
    Pop-Location
}
