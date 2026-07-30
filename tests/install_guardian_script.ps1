$ErrorActionPreference = 'Stop'
Set-StrictMode -Version Latest

$root = Split-Path -Parent $PSScriptRoot
$temporaryDirectory = Join-Path ([IO.Path]::GetTempPath()) ("guardian-installer-test-" + [guid]::NewGuid())

try {
    $pluginDirectory = Join-Path $temporaryDirectory 'plugin'
    $profileDirectory = Join-Path $temporaryDirectory 'profile'
    New-Item -ItemType Directory -Force -Path $pluginDirectory, $profileDirectory | Out-Null
    Set-Content -LiteralPath (Join-Path $pluginDirectory 'manifest.toml') -NoNewline -Value @'
name = "solana-transaction-guardian"
wasm_path = "solana_transaction_guardian.wasm"
'@
    New-Item -ItemType File -Path (Join-Path $pluginDirectory 'solana_transaction_guardian.wasm') | Out-Null
    New-Item -ItemType File -Path (Join-Path $profileDirectory 'config.toml') | Out-Null

    $mockPath = Join-Path $temporaryDirectory 'zeroclaw.ps1'
    Set-Content -LiteralPath $mockPath -NoNewline -Value @'
param([Parameter(ValueFromRemainingArguments = $true)][string[]]$Arguments)
if ($Arguments[0] -notin @('plugin')) { exit 1 }
if ($Arguments[1] -notin @('install', 'info')) { exit 1 }
exit 0
'@

    & (Join-Path $root 'scripts\install-guardian.ps1') `
        -PluginPath $pluginDirectory `
        -ProfilePath $profileDirectory `
        -ZeroClawPath $mockPath

    if ($LASTEXITCODE -ne 0) {
        throw "Installer returned exit code $LASTEXITCODE."
    }

    $archivePath = Join-Path $temporaryDirectory 'guardian.zip'
    Compress-Archive -LiteralPath $pluginDirectory -DestinationPath $archivePath
    & (Join-Path $root 'scripts\install-guardian.ps1') `
        -PluginPath $archivePath `
        -ProfilePath $profileDirectory `
        -ZeroClawPath $mockPath

    if ($LASTEXITCODE -ne 0) {
        throw "Archive installer returned exit code $LASTEXITCODE."
    }
} finally {
    Remove-Item -LiteralPath $temporaryDirectory -Recurse -Force -ErrorAction SilentlyContinue
}
