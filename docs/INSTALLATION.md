# Installation

## Use it with your existing ZeroClaw agent

Guardian is model- and provider-independent. It does not require Ollama,
Qwen, a new agent, a new prompt, or a separate ZeroClaw profile. Install it
into the ZeroClaw profile you already use; the same agent and model that the
user selected will receive `solana_transaction_guardian` as a tool.

1. Install and onboard ZeroClaw with the model provider you prefer.
2. Download and verify a Guardian release archive.
3. Extract it and run its installer from PowerShell:

```powershell
Expand-Archive .\solana-transaction-guardian-<VERSION>.zip -DestinationPath .
.\solana-transaction-guardian-<VERSION>\install-guardian.ps1 `
  -PluginPath .\solana-transaction-guardian-<VERSION>
```

If you use a non-default ZeroClaw profile, add `-ProfilePath`:

```powershell
.\solana-transaction-guardian-<VERSION>\install-guardian.ps1 `
  -PluginPath .\solana-transaction-guardian-<VERSION> `
  -ProfilePath C:\path\to\your\zeroclaw-profile
```

The installer validates the package shape, invokes `zeroclaw plugin install`,
and confirms that ZeroClaw can inspect the installed plugin. It never changes
the model provider, agent identity, prompt, credentials, or policy already in
the user's profile.

With no Guardian configuration, the plugin is ready for devnet analysis using
the official Solana devnet RPC and a fail-closed policy. Ask the existing agent
to analyze a serialized devnet transaction or a confirmed devnet signature.

## Strict plugin signatures

If the profile uses `signature_mode = "strict"`, add Guardian's public publisher
key to that profile before installation. The optional
[configuration example](../config/zeroclaw.guardian.example.toml) shows the
exact key. Verify it against [PUBLISHER_KEY.md](PUBLISHER_KEY.md) and the
release manifest. The public key is not a secret.

## Optional configuration

No plugin configuration is required for the devnet default. Use
[CONFIGURATION.md](CONFIGURATION.md) only when you need mainnet, a private RPC
endpoint, a different policy, or lower resource limits. Add the relevant
`[plugins]` sections to the existing profile; do not replace the user's whole
`config.toml` and do not add a second model provider just for Guardian.

## Reference environment

ZeroClaw v0.8.3 at commit `24476b71d33eb1672a9495a7ce3d155377a60ce8`, Ollama
0.32.0, and `qwen3.5:9b` are the reproducible demonstration environment, not
a user requirement. Its behavior evidence is retained in
[LLM_RUNTIME.md](LLM_RUNTIME.md).

## Build from source

```bash
rustup toolchain install 1.96.1
rustup target add wasm32-wasip2 --toolchain 1.96.1
cargo +1.96.1 test --locked --all-targets
cargo +1.96.1 clippy --locked --all-targets --all-features -- -D warnings
cargo +1.96.1 build --locked --release --target wasm32-wasip2
```

The canonical release is built in the pinned Linux container. See
[REPRODUCIBLE_BUILD.md](REPRODUCIBLE_BUILD.md) for the image digest and
canonical hash.
