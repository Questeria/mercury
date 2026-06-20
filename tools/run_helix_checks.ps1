param(
    [string]$HelixRoot = ""
)

$ErrorActionPreference = "Stop"

$MercuryRoot = (Resolve-Path (Join-Path $PSScriptRoot "..")).Path
$Policy = Join-Path $MercuryRoot "helix\policy\envelope.hx"
$Test = Join-Path $MercuryRoot "helix\tests\envelope_test.hx"
$AiGrantPolicy = Join-Path $MercuryRoot "helix\policy\ai_grant.hx"
$AiGrantTest = Join-Path $MercuryRoot "helix\tests\ai_grant_test.hx"
$AiGrantLifecyclePolicy = Join-Path $MercuryRoot "helix\policy\ai_grant_lifecycle.hx"
$AiGrantLifecycleTest = Join-Path $MercuryRoot "helix\tests\ai_grant_lifecycle_test.hx"
$RoomEpochPolicy = Join-Path $MercuryRoot "helix\policy\room_epoch.hx"
$RoomEpochTest = Join-Path $MercuryRoot "helix\tests\room_epoch_test.hx"
$PolicyPipelinePolicy = Join-Path $MercuryRoot "helix\policy\policy_pipeline.hx"
$PolicyPipelineTest = Join-Path $MercuryRoot "helix\tests\policy_pipeline_test.hx"
$RelaySubmitPolicy = Join-Path $MercuryRoot "helix\policy\relay_submit.hx"
$RelaySubmitTest = Join-Path $MercuryRoot "helix\tests\relay_submit_test.hx"
$PlatformDecisionPolicy = Join-Path $MercuryRoot "helix\policy\platform_decision.hx"
$PlatformDecisionTest = Join-Path $MercuryRoot "helix\tests\platform_decision_test.hx"
$OutboundDecidePolicy = Join-Path $MercuryRoot "helix\policy\outbound_decide.hx"
$OutboundDecideTest = Join-Path $MercuryRoot "helix\tests\outbound_decide_test.hx"
$ReceiveDecidePolicy = Join-Path $MercuryRoot "helix\policy\receive_decide.hx"
$ReceiveDecideTest = Join-Path $MercuryRoot "helix\tests\receive_decide_test.hx"
$BootstrapDecidePolicy = Join-Path $MercuryRoot "helix\policy\bootstrap_decide.hx"
$BootstrapDecideTest = Join-Path $MercuryRoot "helix\tests\bootstrap_decide_test.hx"
$InboundSyncPolicy = Join-Path $MercuryRoot "helix\policy\inbound_sync.hx"
$InboundSyncTest = Join-Path $MercuryRoot "helix\tests\inbound_sync_test.hx"
$AccountRecoveryPolicy = Join-Path $MercuryRoot "helix\policy\account_recovery.hx"
$AccountRecoveryTest = Join-Path $MercuryRoot "helix\tests\account_recovery_test.hx"
$OutDir = Join-Path $MercuryRoot "build\helix"
$TestBin = Join-Path $OutDir "envelope_test.bin"
$AiGrantTestBin = Join-Path $OutDir "ai_grant_test.bin"
$AiGrantLifecycleTestBin = Join-Path $OutDir "ai_grant_lifecycle_test.bin"
$RoomEpochTestBin = Join-Path $OutDir "room_epoch_test.bin"
$PolicyPipelineTestBin = Join-Path $OutDir "policy_pipeline_test.bin"
$RelaySubmitTestBin = Join-Path $OutDir "relay_submit_test.bin"
$PlatformDecisionTestBin = Join-Path $OutDir "platform_decision_test.bin"
$OutboundDecideTestBin = Join-Path $OutDir "outbound_decide_test.bin"
$ReceiveDecideTestBin = Join-Path $OutDir "receive_decide_test.bin"
$BootstrapDecideTestBin = Join-Path $OutDir "bootstrap_decide_test.bin"
$InboundSyncTestBin = Join-Path $OutDir "inbound_sync_test.bin"
$AccountRecoveryTestBin = Join-Path $OutDir "account_recovery_test.bin"

if ([string]::IsNullOrWhiteSpace($HelixRoot)) {
    $HelixRoot = Join-Path $MercuryRoot "third_party\helix"
}

if (-not (Test-Path $HelixRoot)) {
    throw "HelixRoot not found: $HelixRoot"
}

$oldPythonPath = $env:PYTHONPATH
$env:PYTHONPATH = if ([string]::IsNullOrWhiteSpace($oldPythonPath)) {
    $HelixRoot
} else {
    "$HelixRoot;$oldPythonPath"
}

Push-Location $MercuryRoot
try {
    New-Item -ItemType Directory -Force -Path $OutDir | Out-Null

    # Drift gates (platform-independent). Proof manifests first — attestation.json folds their hashes
    # in — then the attestation, then the exhaustive-differential test sources.
    python .\tools\gen_proof_manifests.py --check
    python .\tools\gen_exhaustive_helix_diff.py --check
    python .\tools\gen_policy_attestations.py --check
    python -m helixc.check $Policy --no-stdlib --check-only --strict --hash
    python -m helixc.check $Policy --no-stdlib --emit-proof-obligations --strict
    python -m helixc.check $Test --no-stdlib --check-only --strict --hash
    python -m helixc.check $Test --no-stdlib -O1 -o $TestBin
    python -m helixc.check $AiGrantPolicy --no-stdlib --check-only --strict --hash
    python -m helixc.check $AiGrantPolicy --no-stdlib --emit-proof-obligations --strict
    python -m helixc.check $AiGrantTest --no-stdlib --check-only --strict --hash
    python -m helixc.check $AiGrantTest --no-stdlib -O1 -o $AiGrantTestBin
    python -m helixc.check $AiGrantLifecyclePolicy --no-stdlib --check-only --strict --hash
    python -m helixc.check $AiGrantLifecyclePolicy --no-stdlib --emit-proof-obligations --strict
    python -m helixc.check $AiGrantLifecycleTest --no-stdlib --check-only --strict --hash
    python -m helixc.check $AiGrantLifecycleTest --no-stdlib -O1 -o $AiGrantLifecycleTestBin
    python -m helixc.check $RoomEpochPolicy --no-stdlib --check-only --strict --hash
    python -m helixc.check $RoomEpochPolicy --no-stdlib --emit-proof-obligations --strict
    python -m helixc.check $RoomEpochTest --no-stdlib --check-only --strict --hash
    python -m helixc.check $RoomEpochTest --no-stdlib -O1 -o $RoomEpochTestBin
    python -m helixc.check $PolicyPipelinePolicy --no-stdlib --check-only --strict --hash
    python -m helixc.check $PolicyPipelinePolicy --no-stdlib --emit-proof-obligations --strict
    python -m helixc.check $PolicyPipelineTest --no-stdlib --check-only --strict --hash
    python -m helixc.check $PolicyPipelineTest --no-stdlib -O1 -o $PolicyPipelineTestBin
    python -m helixc.check $RelaySubmitPolicy --no-stdlib --check-only --strict --hash
    python -m helixc.check $RelaySubmitPolicy --no-stdlib --emit-proof-obligations --strict
    python -m helixc.check $RelaySubmitTest --no-stdlib --check-only --strict --hash
    python -m helixc.check $RelaySubmitTest --no-stdlib -O1 -o $RelaySubmitTestBin
    python -m helixc.check $PlatformDecisionPolicy --no-stdlib --check-only --strict --hash
    python -m helixc.check $PlatformDecisionPolicy --no-stdlib --emit-proof-obligations --strict
    python -m helixc.check $PlatformDecisionTest --no-stdlib --check-only --strict --hash
    python -m helixc.check $PlatformDecisionTest --no-stdlib -O1 -o $PlatformDecisionTestBin
    python -m helixc.check $OutboundDecidePolicy --no-stdlib --check-only --strict --hash
    python -m helixc.check $OutboundDecidePolicy --no-stdlib --emit-proof-obligations --strict
    python -m helixc.check $OutboundDecideTest --no-stdlib --check-only --strict --hash
    python -m helixc.check $OutboundDecideTest --no-stdlib -O1 -o $OutboundDecideTestBin
    python -m helixc.check $ReceiveDecidePolicy --no-stdlib --check-only --strict --hash
    python -m helixc.check $ReceiveDecidePolicy --no-stdlib --emit-proof-obligations --strict
    python -m helixc.check $ReceiveDecideTest --no-stdlib --check-only --strict --hash
    python -m helixc.check $ReceiveDecideTest --no-stdlib -O1 -o $ReceiveDecideTestBin
    python -m helixc.check $BootstrapDecidePolicy --no-stdlib --check-only --strict --hash
    python -m helixc.check $BootstrapDecidePolicy --no-stdlib --emit-proof-obligations --strict
    python -m helixc.check $BootstrapDecideTest --no-stdlib --check-only --strict --hash
    python -m helixc.check $BootstrapDecideTest --no-stdlib -O1 -o $BootstrapDecideTestBin
    python -m helixc.check $InboundSyncPolicy --no-stdlib --check-only --strict --hash
    python -m helixc.check $InboundSyncPolicy --no-stdlib --emit-proof-obligations --strict
    python -m helixc.check $InboundSyncTest --no-stdlib --check-only --strict --hash
    python -m helixc.check $InboundSyncTest --no-stdlib -O1 -o $InboundSyncTestBin
    python -m helixc.check $AccountRecoveryPolicy --no-stdlib --check-only --strict --hash
    python -m helixc.check $AccountRecoveryPolicy --no-stdlib --emit-proof-obligations --strict
    python -m helixc.check $AccountRecoveryTest --no-stdlib --check-only --strict --hash
    python -m helixc.check $AccountRecoveryTest --no-stdlib -O1 -o $AccountRecoveryTestBin

    if (Get-Command wsl.exe -ErrorAction SilentlyContinue) {
        $drive = $MercuryRoot.Substring(0, 1).ToLowerInvariant()
        $rest = $MercuryRoot.Substring(2).Replace("\", "/")
        $wslMercuryRoot = "/mnt/$drive$rest"

        wsl.exe bash -lc "cd $wslMercuryRoot && chmod +x ./build/helix/envelope_test.bin && ./build/helix/envelope_test.bin"
        $runtimeExit = $LASTEXITCODE
        if ($runtimeExit -ne 42) {
            throw "Helix runtime test expected exit 42, got $runtimeExit"
        }
        Write-Host "runtime: OK (envelope_test.bin exit 42)"

        wsl.exe bash -lc "cd $wslMercuryRoot && chmod +x ./build/helix/ai_grant_test.bin && ./build/helix/ai_grant_test.bin"
        $runtimeExit = $LASTEXITCODE
        if ($runtimeExit -ne 42) {
            throw "Helix runtime test expected exit 42, got $runtimeExit"
        }
        Write-Host "runtime: OK (ai_grant_test.bin exit 42)"

        wsl.exe bash -lc "cd $wslMercuryRoot && chmod +x ./build/helix/ai_grant_lifecycle_test.bin && ./build/helix/ai_grant_lifecycle_test.bin"
        $runtimeExit = $LASTEXITCODE
        if ($runtimeExit -ne 42) {
            throw "Helix runtime test expected exit 42, got $runtimeExit"
        }
        Write-Host "runtime: OK (ai_grant_lifecycle_test.bin exit 42)"

        wsl.exe bash -lc "cd $wslMercuryRoot && chmod +x ./build/helix/room_epoch_test.bin && ./build/helix/room_epoch_test.bin"
        $runtimeExit = $LASTEXITCODE
        if ($runtimeExit -ne 42) {
            throw "Helix runtime test expected exit 42, got $runtimeExit"
        }
        Write-Host "runtime: OK (room_epoch_test.bin exit 42)"

        wsl.exe bash -lc "cd $wslMercuryRoot && chmod +x ./build/helix/policy_pipeline_test.bin && ./build/helix/policy_pipeline_test.bin"
        $runtimeExit = $LASTEXITCODE
        if ($runtimeExit -ne 42) {
            throw "Helix runtime test expected exit 42, got $runtimeExit"
        }
        Write-Host "runtime: OK (policy_pipeline_test.bin exit 42)"

        wsl.exe bash -lc "cd $wslMercuryRoot && chmod +x ./build/helix/relay_submit_test.bin && ./build/helix/relay_submit_test.bin"
        $runtimeExit = $LASTEXITCODE
        if ($runtimeExit -ne 42) {
            throw "Helix runtime test expected exit 42, got $runtimeExit"
        }
        Write-Host "runtime: OK (relay_submit_test.bin exit 42)"

        wsl.exe bash -lc "cd $wslMercuryRoot && chmod +x ./build/helix/platform_decision_test.bin && ./build/helix/platform_decision_test.bin"
        $runtimeExit = $LASTEXITCODE
        if ($runtimeExit -ne 42) {
            throw "Helix runtime test expected exit 42, got $runtimeExit"
        }
        Write-Host "runtime: OK (platform_decision_test.bin exit 42)"

        wsl.exe bash -lc "cd $wslMercuryRoot && chmod +x ./build/helix/outbound_decide_test.bin && ./build/helix/outbound_decide_test.bin"
        $runtimeExit = $LASTEXITCODE
        if ($runtimeExit -ne 42) {
            throw "Helix runtime test expected exit 42, got $runtimeExit"
        }
        Write-Host "runtime: OK (outbound_decide_test.bin exit 42)"

        wsl.exe bash -lc "cd $wslMercuryRoot && chmod +x ./build/helix/receive_decide_test.bin && ./build/helix/receive_decide_test.bin"
        $runtimeExit = $LASTEXITCODE
        if ($runtimeExit -ne 42) {
            throw "Helix runtime test expected exit 42, got $runtimeExit"
        }
        Write-Host "runtime: OK (receive_decide_test.bin exit 42)"

        wsl.exe bash -lc "cd $wslMercuryRoot && chmod +x ./build/helix/bootstrap_decide_test.bin && ./build/helix/bootstrap_decide_test.bin"
        $runtimeExit = $LASTEXITCODE
        if ($runtimeExit -ne 42) {
            throw "Helix runtime test expected exit 42, got $runtimeExit"
        }
        Write-Host "runtime: OK (bootstrap_decide_test.bin exit 42)"

        wsl.exe bash -lc "cd $wslMercuryRoot && chmod +x ./build/helix/inbound_sync_test.bin && ./build/helix/inbound_sync_test.bin"
        $runtimeExit = $LASTEXITCODE
        if ($runtimeExit -ne 42) {
            throw "Helix runtime test expected exit 42, got $runtimeExit"
        }
        Write-Host "runtime: OK (inbound_sync_test.bin exit 42)"

        wsl.exe bash -lc "cd $wslMercuryRoot && chmod +x ./build/helix/account_recovery_test.bin && ./build/helix/account_recovery_test.bin"
        $runtimeExit = $LASTEXITCODE
        if ($runtimeExit -ne 42) {
            throw "Helix runtime test expected exit 42, got $runtimeExit"
        }
        Write-Host "runtime: OK (account_recovery_test.bin exit 42)"
    } else {
        Write-Host "runtime: SKIP (wsl.exe not found; ELF was built but not executed)"
    }
}
finally {
    Pop-Location
    $env:PYTHONPATH = $oldPythonPath
}
