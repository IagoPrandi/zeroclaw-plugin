[CmdletBinding()]
param(
    [Parameter(Mandatory)]
    [ValidateNotNullOrEmpty()]
    [string]$PluginPath,

    [ValidateNotNullOrEmpty()]
    [string]$ZeroClawPath = 'zeroclaw',

    [string]$ProfilePath
)

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

function Test-GuardianArchive {
    param([Parameter(Mandatory)][string]$Path)

    $archive = [IO.Compression.ZipFile]::OpenRead($Path)
    try {
        if ($archive.Entries.Count -gt 16) {
            throw 'The release archive contains too many entries.'
        }

        $totalSize = 0L
        foreach ($entry in $archive.Entries) {
            if ($entry.FullName -match '(^|[\\/])\.\.([\\/]|$)|^[\\/]|^[A-Za-z]:') {
                throw "The release archive contains an unsafe path: $($entry.FullName)"
            }
            $totalSize += $entry.Length
            if ($totalSize -gt 50MB) {
                throw 'The release archive expands beyond the 50 MiB installation limit.'
            }
        }
    } finally {
        $archive.Dispose()
    }
}

function Get-GuardianPluginDirectory {
    param([Parameter(Mandatory)][string]$Path)

    $resolvedPath = (Resolve-Path -LiteralPath $Path).Path
    if (Test-Path -LiteralPath $resolvedPath -PathType Container) {
        return [pscustomobject]@{ Directory = $resolvedPath; TemporaryDirectory = $null }
    }

    if ([IO.Path]::GetExtension($resolvedPath) -ne '.zip') {
        throw 'PluginPath must be a plugin directory or a .zip release archive.'
    }

    Test-GuardianArchive -Path $resolvedPath
    $temporaryDirectory = Join-Path ([IO.Path]::GetTempPath()) ("guardian-install-" + [guid]::NewGuid())
    New-Item -ItemType Directory -Path $temporaryDirectory | Out-Null
    try {
        Expand-Archive -LiteralPath $resolvedPath -DestinationPath $temporaryDirectory
        $manifests = @(Get-ChildItem -LiteralPath $temporaryDirectory -Recurse -File -Filter manifest.toml)
        if ($manifests.Count -ne 1) {
            throw 'The release archive must contain exactly one manifest.toml.'
        }

        return [pscustomobject]@{
            Directory = $manifests[0].DirectoryName
            TemporaryDirectory = $temporaryDirectory
        }
    } catch {
        Remove-Item -LiteralPath $temporaryDirectory -Recurse -Force -ErrorAction SilentlyContinue
        throw
    }
}

$package = Get-GuardianPluginDirectory -Path $PluginPath
$pluginDirectory = $package.Directory
$temporaryPluginDirectory = $package.TemporaryDirectory

try {
    $manifestPath = Join-Path $pluginDirectory 'manifest.toml'
    $wasmPath = Join-Path $pluginDirectory 'solana_transaction_guardian.wasm'
    if (-not (Test-Path -LiteralPath $manifestPath -PathType Leaf)) {
        throw "manifest.toml was not found in $pluginDirectory"
    }
    if (-not (Test-Path -LiteralPath $wasmPath -PathType Leaf)) {
        throw "solana_transaction_guardian.wasm was not found in $pluginDirectory"
    }

    $manifest = Get-Content -LiteralPath $manifestPath -Raw
    if ($manifest -notmatch '(?m)^name = "solana-transaction-guardian"$') {
        throw 'The package manifest is not Solana Transaction Guardian.'
    }

    if ($ProfilePath) {
        $resolvedProfile = (Resolve-Path -LiteralPath $ProfilePath).Path
        if (-not (Test-Path -LiteralPath (Join-Path $resolvedProfile 'config.toml') -PathType Leaf)) {
            throw "No config.toml was found in profile: $resolvedProfile"
        }
    }

    $zeroClawCommand = Get-Command $ZeroClawPath -ErrorAction Stop
    $installArguments = @('plugin', 'install', $pluginDirectory)
    if ($ProfilePath) {
        $installArguments += @('--config-dir', $resolvedProfile)
    }

    & $zeroClawCommand.Source @installArguments
    if ($LASTEXITCODE -ne 0) {
        throw "ZeroClaw refused the plugin installation (exit code $LASTEXITCODE)."
    }

    $infoArguments = @('plugin', 'info', 'solana-transaction-guardian')
    if ($ProfilePath) {
        $infoArguments += @('--config-dir', $resolvedProfile)
    }
    & $zeroClawCommand.Source @infoArguments
    if ($LASTEXITCODE -ne 0) {
        throw "The plugin installed but ZeroClaw could not inspect it (exit code $LASTEXITCODE)."
    }

    Write-Host 'Guardian is installed. It uses your current ZeroClaw agent and model.' -ForegroundColor Green
    Write-Host 'No Ollama setup or plugin configuration is needed for the safe devnet default.'
    Write-Host 'For mainnet, a private RPC, or custom policy, use config/zeroclaw.guardian.example.toml.'
} finally {
    if ($temporaryPluginDirectory) {
        Remove-Item -LiteralPath $temporaryPluginDirectory -Recurse -Force -ErrorAction SilentlyContinue
    }
}
