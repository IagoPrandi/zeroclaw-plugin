# Installation

## 1. Verify prerequisites

The validated environment is:

| Component | Pinned value |
|---|---|
| ZeroClaw | v0.8.3 / `24476b71d33eb1672a9495a7ce3d155377a60ce8` |
| Ollama | 0.32.0 |
| Model | `qwen3.5:9b` |
| Model digest | `6488c96fa5faab64bb65cbd30d4289e20e6130ef535a93ef9a49f42eda893ea7` |
| Rust | 1.96.1 |
| WASI target | `wasm32-wasip2` |

Install ZeroClaw from its official repository and check out the pinned commit
when reproducing the validated host:

```bash
git clone https://github.com/zeroclaw-labs/zeroclaw.git
cd zeroclaw
git checkout 24476b71d33eb1672a9495a7ce3d155377a60ce8
cargo build --release --features plugins-wasm-cranelift
```

Use the resulting `zeroclaw` binary. Do not silently substitute another tag:
the WIT and plugin-security behavior are host-versioned.

## 2. Start local Ollama

Install Ollama from its official distribution, bind it to localhost, and pull
the exact model:

```bash
ollama serve
ollama pull qwen3.5:9b
ollama list
ollama show qwen3.5:9b
```

The Ollama tags API exposes the full digest:

```bash
curl --fail --silent http://127.0.0.1:11434/api/tags
```

Confirm that `qwen3.5:9b` has digest
`6488c96fa5faab64bb65cbd30d4289e20e6130ef535a93ef9a49f42eda893ea7`.
Stop if it differs and repeat the validated behavior matrix before approving
the substitution.

No cloud model credential is required. Do not expose port 11434 publicly.

## 3. Verify a release archive

Download these assets from the same GitHub release:

- `solana-transaction-guardian-0.1.0.zip`;
- `SHA256SUMS`.

Unix:

```bash
sha256sum --check SHA256SUMS
unzip solana-transaction-guardian-0.1.0.zip
```

PowerShell:

```powershell
Get-FileHash .\solana-transaction-guardian-0.1.0.zip -Algorithm SHA256
Expand-Archive .\solana-transaction-guardian-0.1.0.zip -DestinationPath .
```

Compare the PowerShell result with the archive entry in `SHA256SUMS`. The
archive contains one plugin directory with one signed `manifest.toml`.

## 4. Create the ZeroClaw profile

Create a dedicated configuration directory. Copy
`config/zeroclaw.guardian.example.toml` from the repository to
`<profile>/config.toml`. Copy the versioned Guardian prompt to the agent
workspace.

Unix:

```bash
PROFILE="$PWD/guardian-profile"
mkdir -p "$PROFILE/agents/guardian/workspace"
cp config/zeroclaw.guardian.example.toml "$PROFILE/config.toml"
cp prompts/GUARDIAN_SYSTEM.md "$PROFILE/agents/guardian/workspace/SOUL.md"
```

PowerShell:

```powershell
$guardianProfile = Join-Path $PWD 'guardian-profile'
New-Item -ItemType Directory -Force `
  (Join-Path $guardianProfile 'agents\guardian\workspace') | Out-Null
Copy-Item .\config\zeroclaw.guardian.example.toml `
  (Join-Path $guardianProfile 'config.toml')
Copy-Item .\prompts\GUARDIAN_SYSTEM.md `
  (Join-Path $guardianProfile 'agents\guardian\workspace\SOUL.md')
```

Verify that the public key in `config.toml`, the signed manifest, the release
notes, and [PUBLISHER_KEY.md](PUBLISHER_KEY.md) are identical. Do not add a
cloud provider section. See [CONFIGURATION.md](CONFIGURATION.md) before adding
mainnet or custom policy.

## 5. Install and verify the plugin

```bash
zeroclaw plugin install ./solana-transaction-guardian-0.1.0 \
  --config-dir /path/to/guardian-profile
zeroclaw plugin list --config-dir /path/to/guardian-profile
zeroclaw plugin info solana-transaction-guardian \
  --config-dir /path/to/guardian-profile
```

Strict mode must reject an unsigned, modified, or untrusted manifest. If
installation fails, verify that:

1. the public key exactly matches `publisher_key` in the manifest;
2. both SHA-256 values match the release;
3. the manifest was not reformatted after signing;
4. the ZeroClaw commit and configuration directory are correct.

## 6. Run the agent

```bash
zeroclaw agent --agent guardian \
  --config-dir /path/to/guardian-profile \
  --message "Analyze this devnet transaction: <BASE64>"
```

The response must contain the literal canonical decision and preserve every
critical/high finding. Treat the JSON tool output as authoritative.

## Build from source

For local development:

```bash
rustup toolchain install 1.96.1
rustup target add wasm32-wasip2 --toolchain 1.96.1
cargo +1.96.1 test --locked --all-targets
cargo +1.96.1 clippy --locked --all-targets --all-features -- -D warnings
cargo +1.96.1 build --locked --release --target wasm32-wasip2
```

The canonical release is built in the pinned Linux container, not declared
bit-reproducible across host OSes. Follow
[REPRODUCIBLE_BUILD.md](REPRODUCIBLE_BUILD.md) for the image digest and known
canonical hash.
