# Installation

Guardian is model- and provider-independent. It installs into the existing
ZeroClaw profile; the agent and model already configured in that profile then
receive `solana_transaction_guardian` as a tool. Guardian does not install or
configure ZeroClaw, an agent, a model provider, Ollama, or credentials.

## 1. Prerequisites

Install and onboard ZeroClaw with the model provider you want to use. Its
binary must include the WASM plugin host. Check that the plugin command exists:

```powershell
zeroclaw plugin --help
```

If this command is unavailable, install a ZeroClaw build with
`plugins-wasm` and a WASM backend such as `plugins-wasm-cranelift`; Guardian
cannot be loaded by a host built without plugin support. The validated host is
ZeroClaw v0.8.3.

## 2. Download and verify the release

Download only from the [GitHub release page](https://github.com/IagoPrandi/zeroclaw-plugin/releases).
For the current v0.1.0 release, use:

```powershell
Invoke-WebRequest `
  -Uri "https://github.com/IagoPrandi/zeroclaw-plugin/releases/download/v0.1.0/solana-transaction-guardian-0.1.0.zip" `
  -OutFile .\solana-transaction-guardian-0.1.0.zip

Get-FileHash .\solana-transaction-guardian-0.1.0.zip -Algorithm SHA256
# Expected: 70a3ac35eb34850cddb5dd745be216278d0d0278924697dcb4e3e6d49cea1b3b
```

Do not continue if the hash differs. The published release assets also include
the signed `manifest.toml`, `solana_transaction_guardian.wasm`, and
`SHA256SUMS` for independent verification.

## 3. Strict plugin signatures (recommended)

Before installing in a profile that uses strict signatures, compare the
publisher key in [PUBLISHER_KEY.md](PUBLISHER_KEY.md), the downloaded
`manifest.toml`, and the release notes. Then configure the trusted public key:

```powershell
zeroclaw config set plugins.security.signature_mode strict
zeroclaw config set plugins.security.trusted_publisher_keys '["d743b2cd62da45564844b273760776c076642cec487700fdedfc601100e5c96d"]'
```

The publisher key is public, not a credential. Never place a seed phrase,
private key, or signing secret in the plugin configuration.

## 4. Install it into the active ZeroClaw profile

Extract the verified archive, enable plugins, install the resulting directory,
and inspect the installed component:

```powershell
Expand-Archive .\solana-transaction-guardian-0.1.0.zip -DestinationPath .
zeroclaw config set plugins.enabled true
zeroclaw plugin install .\solana-transaction-guardian-0.1.0
zeroclaw plugin list
zeroclaw plugin info solana-transaction-guardian
```

The final command must show `solana-transaction-guardian`; that is the
installation check that makes the `solana_transaction_guardian` tool available
to the agent. The v0.1.0 archive must be installed with the native
`zeroclaw plugin install` command because it does not contain an
`install-guardian.ps1` script.

For a non-default profile, pass the same profile directory to every command:

```powershell
zeroclaw config set plugins.enabled true --config-dir C:\path\to\zeroclaw-profile
zeroclaw plugin install .\solana-transaction-guardian-0.1.0 --config-dir C:\path\to\zeroclaw-profile
zeroclaw plugin info solana-transaction-guardian --config-dir C:\path\to\zeroclaw-profile
```

## 5. Use the installed tool

With no Guardian-specific configuration, the installed plugin has fail-closed
mainnet and devnet defaults that use the official Solana RPC endpoints. Ask
the existing agent to analyze either a serialized candidate transaction or a
confirmed signature:

```powershell
zeroclaw agent --agent <your-agent> `
  --message "Analyze this devnet transaction: <BASE64>"
```

Private RPC endpoints and policy changes are configured only when needed; see
[CONFIGURATION.md](CONFIGURATION.md). RPC URLs never belong in an agent/tool
request.

## Build from source

The release archive is the recommended user installation path. Building from
source is for developers who need to reproduce or modify the component:

```bash
rustup toolchain install 1.96.1
rustup target add wasm32-wasip2 --toolchain 1.96.1
cargo +1.96.1 test --locked --all-targets
cargo +1.96.1 clippy --locked --all-targets --all-features -- -D warnings
cargo +1.96.1 build --locked --release --target wasm32-wasip2
```

The canonical release is built in the pinned Linux container. See
[REPRODUCIBLE_BUILD.md](REPRODUCIBLE_BUILD.md) for its image digest and
canonical hash.
