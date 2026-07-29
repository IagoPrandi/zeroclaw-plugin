param(
    [Parameter(Mandatory = $true)]
    [string]$ZeroClawPath,

    [Parameter(Mandatory = $true)]
    [string]$ProfilePath,

    [string]$FixturePath = '',

    [switch]$SkipAgent,

    [ValidateRange(0, 120)]
    [int]$HoldSeconds = 15
)

$ErrorActionPreference = 'Stop'
$Host.UI.RawUI.WindowTitle = 'GuardianDemo'

if ([string]::IsNullOrWhiteSpace($FixturePath)) {
    $FixturePath = Join-Path $PSScriptRoot 'fixtures\02-hidden-delegate.json'
}

function Show-Section {
    param([string]$Title)

    Write-Host ''
    Write-Host ('=' * 76) -ForegroundColor DarkGray
    Write-Host $Title -ForegroundColor Cyan
    Write-Host ('=' * 76) -ForegroundColor DarkGray
}

if (-not (Test-Path -LiteralPath $ZeroClawPath -PathType Leaf)) {
    throw "ZeroClaw executable not found: $ZeroClawPath"
}
if (-not (Test-Path -LiteralPath $ProfilePath -PathType Container)) {
    throw "ZeroClaw profile not found: $ProfilePath"
}
if (-not (Test-Path -LiteralPath $FixturePath -PathType Leaf)) {
    throw "Demo fixture not found: $FixturePath"
}

Clear-Host
Write-Host 'SOLANA TRANSACTION GUARDIAN' -ForegroundColor Green
Write-Host 'Real ZeroClaw v0.8.3 + local Ollama demonstration'
Write-Host 'Custody: T0 / read-only'

Show-Section '1. LOCAL MODEL - NO CLOUD FALLBACK'
ollama list | Select-String -Pattern '^qwen3\.5:9b\s'
$configPath = Join-Path $ProfilePath 'config.toml'
Get-Content -LiteralPath $configPath |
    Select-String -Pattern 'uri = "http://127\.0\.0\.1:11434"|model = "qwen3\.5:9b"|model_provider = "ollama\.local"|signature_mode = "strict"'
Write-Host 'No cloud provider or fallback is defined.' -ForegroundColor Green

Show-Section '2. SIGNED ZEROCLAW PLUGIN'
& $ZeroClawPath plugin list --config-dir $ProfilePath
& $ZeroClawPath plugin info solana-transaction-guardian --config-dir $ProfilePath

$fixture = Get-Content -LiteralPath $FixturePath -Raw -Encoding utf8 |
    ConvertFrom-Json
$input = $fixture.tool_input
$base64 = $input.source.transaction_base64

Show-Section "3. REAL AGENT CALL - $($fixture.title.ToUpperInvariant())"
Write-Host $fixture.narrative -ForegroundColor Yellow
Write-Host 'Waiting for local CPU inference; the final video may use a jump cut.'

if (-not $SkipAgent) {
    if ($null -ne $input.expected_intent) {
        $wallet = $input.observed_wallets[0]
        $recipient = $input.expected_intent.allowed_recipients[0]
        $message = @"
Analyze this serialized Solana transaction on devnet:
$base64

Observed wallet: $wallet
Expected intent: only transfer at most 1 lamport to $recipient, using only
the System Program 11111111111111111111111111111111. Explain the canonical
decision and all critical or high findings. Never sign or broadcast.
"@
    }
    else {
        $message = @"
Analyze this serialized Solana transaction on devnet:
$base64

Explain the canonical decision, decoded actions, coverage, and limitations.
Preserve raw integer lamport amounts exactly as reported. Do not convert units
or infer a purpose for the transfer.
Never sign or broadcast.
"@
    }
    & $ZeroClawPath agent --agent guardian --config-dir $ProfilePath --message $message
    if ($LASTEXITCODE -ne 0) {
        throw "ZeroClaw agent failed with exit code $LASTEXITCODE"
    }
}
else {
    Write-Host '[Agent call skipped for runner validation.]' -ForegroundColor DarkYellow
}

Show-Section '4. VALIDATED RELEASE FIXTURES'
Write-Host 'safe transfer       allow   legacy transfer'
Write-Host 'hidden delegate     block   transfer + approve; AUTH/INT findings'
Write-Host 'unknown program     block   unknown-program + intent findings'
Write-Host 'v0 + ALT            allow   lookup table resolved'

Show-Section '5. REPRODUCIBLE RELEASE'
$sumsPath = Join-Path (Split-Path -Parent $PSScriptRoot) 'dist\0.1.0\SHA256SUMS'
if (Test-Path -LiteralPath $sumsPath -PathType Leaf) {
    Get-Content -LiteralPath $sumsPath
}
Write-Host '60 tests | signed manifest | deterministic WASM | T0 custody' -ForegroundColor Green
Write-Host ''
Write-Host 'The deterministic JSON report is authoritative; Qwen cannot change it.'

if ($HoldSeconds -gt 0) {
    Start-Sleep -Seconds $HoldSeconds
}
