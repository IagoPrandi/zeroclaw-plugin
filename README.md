# Solana Transaction Guardian for ZeroClaw

Solana Transaction Guardian is a read-only Rust/WASM plugin that helps a ZeroClaw agent explain a Solana transaction before someone signs it. It can also inspect a confirmed transaction by signature. The plugin reports what moves, who receives it, what permissions change, and whether the evidence supports an `allow`, `review`, or `block` decision.

[Watch the 2:46 demo](https://github.com/IagoPrandi/zeroclaw-plugin/releases/download/v0.1.0/guardian-demo-walkthrough.mp4) · [Download v0.1.0](https://github.com/IagoPrandi/zeroclaw-plugin/releases/tag/v0.1.0) · [Português do Brasil](README.pt-BR.md)

The walkthrough pairs recorded agent exchanges with the terminal report. It compares a straightforward transfer with a payment-shaped transaction that also grants an SPL Token delegate permission.

## Install

Use an existing ZeroClaw profile with a tool-capable model. **ZeroClaw v0.8.3** is the validated host version. Check for WASM plugin support with `zeroclaw plugin --help`. If the command is unavailable, use a build with `plugins-wasm` and a WASM backend such as `plugins-wasm-cranelift`.

From a working directory in PowerShell, download and verify the release archive, then install it into the active profile.

```powershell
$archive = "solana-transaction-guardian-0.1.0.zip"
$release = "https://github.com/IagoPrandi/zeroclaw-plugin/releases/download/v0.1.0"
Invoke-WebRequest "$release/$archive" -OutFile $archive

$expected = "70a3ac35eb34850cddb5dd745be216278d0d0278924697dcb4e3e6d49cea1b3b"
if ((Get-FileHash $archive -Algorithm SHA256).Hash -ne $expected) {
    throw "The Guardian release archive failed SHA-256 verification."
}

Expand-Archive $archive -DestinationPath .
zeroclaw config set plugins.enabled true
zeroclaw plugin install ./solana-transaction-guardian-0.1.0
zeroclaw plugin info solana-transaction-guardian
```

The last command should display the installed plugin. Use the [installation guide](docs/INSTALLATION.md) for strict publisher signatures, a custom profile directory, and source builds. The plugin uses the model already configured in your profile.

## Try a devnet transaction

Replace `YOUR_AGENT` with the name of your configured ZeroClaw agent. This example inspects a public, confirmed devnet transaction.

```powershell
$signature = "49TXiM9rVcpspUZtT5LynRePwWyazTERXAhYHAsTSAz7fi4TtYspwiUM1zuyGmoRoddJKM9M5K47FPfPcimhufu4"
zeroclaw agent --agent YOUR_AGENT --message "Use solana_transaction_guardian to analyze devnet signature $signature. Report the exact decision, actions, and coverage."
```

The [recorded devnet result](docs/evidence/host-e2e/confirmed-devnet.md) shows a `block` decision with partial coverage (`COV-003`). A fresh run depends on the RPC's availability and transaction retention. For a transaction awaiting review, pass its serialized base64 form and specify the network, observed wallets, and any expected recipients or spending limits. See the [tool request examples](docs/EXAMPLES.md) and [demo fixtures](demo/README.md).

## How it works

1. ZeroClaw calls `solana_transaction_guardian` with a serialized candidate or confirmed signature.
2. The WASM plugin decodes the transaction and uses the configured Solana RPC to simulate the candidate or fetch the confirmed effects.
3. The plugin compares the evidence with operator policy and optional structured intent. The report includes decoded actions, SOL and token balance changes, fees, coverage, and findings tied to 54 stable rule IDs.
4. ZeroClaw's model presents the decision calculated by the plugin.

| Decision | What to do with the report |
| --- | --- |
| `allow` | Review the decoded effects before proceeding. Available evidence passed the configured checks. |
| `review` | Inspect the flagged conditions before approving the transaction. |
| `block` | Stop the workflow at the signing step and inspect the blocking findings or coverage errors. |

Guardian supports legacy and v0 transactions, including Address Lookup Tables, and decodes System, Compute Budget, SPL Token, and Token-2022 instructions within its documented coverage. It is a custody-tier T0 tool with read-only RPC access. The report is advisory and any signing workflow must enforce its own approval gate. Simulation reflects the state at the time it runs.

## Develop and verify

`src/` contains the Rust implementation, `wit/` holds the pinned ZeroClaw plugin interface, and `tests/` and `demo/` contain reproducible scenarios.

```bash
cargo test --locked --all-targets
cargo build --locked --release --target wasm32-wasip2
```

The repository pins Rust 1.96.1 and the WASI Preview 2 target. See the [reproducible build guide](docs/REPRODUCIBLE_BUILD.md) for the Linux release build.

- [Configuration and policy](docs/CONFIGURATION.md)
- [Architecture](docs/ARCHITECTURE.md) and [risk rules](docs/RISK_RULES.md)
- [Verification evidence](docs/TEST_MATRIX.md) and [agent behavior tests](docs/AGENT_BEHAVIOR_TESTS.md)
- [Security policy](SECURITY.md) and [coverage limitations](docs/LIMITATIONS.md)

[MIT license](LICENSE) · [Third-party notices](THIRD_PARTY_NOTICES.md)
