# Solana Transaction Guardian

Solana Transaction Guardian is a read-only ZeroClaw tool plugin that decodes,
simulates, and evaluates Solana transactions before an agent or user trusts
them. It accepts either a serialized transaction or a confirmed signature and
returns one deterministic JSON report with actions, participants, balance
deltas, fees, coverage, findings, and an `allow`, `review`, or `block`
decision.

The Rust/WASM plugin is the security authority. The local `qwen3.5:9b` model
only selects the tool, builds arguments, and explains the report; it cannot
change the canonical decision.

> Custody tier T0: the Guardian has no private-key, signing, or transaction
> submission capability.

[Português do Brasil](README.pt-BR.md)

[Download v0.1.0](https://github.com/IagoPrandi/zeroclaw-plugin/releases/tag/v0.1.0)
·
[Watch the 2:38 public demo](https://github.com/IagoPrandi/zeroclaw-plugin/releases/download/v0.1.0/guardian-demo.mp4)

## Why it exists

A transaction can look like a payment while also approving a delegate,
changing an authority, calling an unknown program, or spending more than the
stated intent. Explorers explain confirmed history. The Guardian can also
analyze an unsigned candidate before signing, compare it with structured
intent, apply operator policy, and fail closed when evidence is incomplete.

## What it covers

- legacy and version-0 transactions, including Address Lookup Tables;
- System, Compute Budget, SPL Token, and Token-2022 instructions;
- simulation or confirmed execution effects, inner instructions, fees, logs,
  return data, SOL deltas, and token deltas;
- deterministic policy and intent checks across 54 stable rule IDs;
- explicit coverage/confidence reporting and controlled errors;
- exactly one ZeroClaw tool: `solana_transaction_guardian`.

See [risk rules](docs/RISK_RULES.md), [architecture](docs/ARCHITECTURE.md), and
[limitations](docs/LIMITATIONS.md).

## Execution model

```text
User
  -> ZeroClaw v0.8.3
     -> local Ollama 0.32.0 / qwen3.5:9b
        -> solana_transaction_guardian (WASM)
           -> configured Solana JSON-RPC
           -> deterministic report
        -> faithful presentation of that report
```

There is no cloud LLM provider or fallback in the reference configuration.
If Ollama or the pinned model is unavailable, the agent flow stops with an
actionable error.

## Quick start

Prerequisites:

- ZeroClaw v0.8.3 at commit
  `24476b71d33eb1672a9495a7ce3d155377a60ce8`;
- Ollama 0.32.0 listening on `127.0.0.1:11434`;
- `qwen3.5:9b` with digest
  `6488c96fa5faab64bb65cbd30d4289e20e6130ef535a93ef9a49f42eda893ea7`;
- the
  [v0.1.0 release archive](https://github.com/IagoPrandi/zeroclaw-plugin/releases/download/v0.1.0/solana-transaction-guardian-0.1.0.zip)
  or Rust 1.96.1 with target `wasm32-wasip2`.

```bash
ollama pull qwen3.5:9b
ollama list
```

Extract the release archive and install its plugin directory:

```bash
zeroclaw plugin install ./solana-transaction-guardian-0.1.0 \
  --config-dir /path/to/guardian-profile
zeroclaw plugin list --config-dir /path/to/guardian-profile
```

Copy [config/zeroclaw.guardian.example.toml](config/zeroclaw.guardian.example.toml)
to the profile as `config.toml`, verify its publisher key against
[docs/PUBLISHER_KEY.md](docs/PUBLISHER_KEY.md), and copy
[prompts/GUARDIAN_SYSTEM.md](prompts/GUARDIAN_SYSTEM.md) to
`agents/guardian/workspace/SOUL.md`. Then run:

```bash
zeroclaw agent --agent guardian \
  --config-dir /path/to/guardian-profile \
  --message "Analyze this devnet transaction: <BASE64>"
```

Use the full [installation guide](docs/INSTALLATION.md) for source builds,
strict signature verification, platform-specific commands, and hash checks.
Configuration and policy fields are documented in
[docs/CONFIGURATION.md](docs/CONFIGURATION.md).

## Tool input

Serialized candidate:

```json
{
  "source": {
    "type": "serialized",
    "transaction_base64": "<BASE64>"
  },
  "cluster": "devnet",
  "observed_wallets": ["<SOLANA_ADDRESS>"],
  "output_language": "en"
}
```

Confirmed transaction:

```json
{
  "source": {
    "type": "confirmed",
    "signature": "<BASE58_SIGNATURE>"
  },
  "cluster": "devnet"
}
```

RPC endpoints never come from tool arguments. Operators map the fixed cluster
aliases to endpoints in ZeroClaw's per-plugin configuration. More requests and
expected decisions are in [docs/EXAMPLES.md](docs/EXAMPLES.md).

## Verification

The v0.1.0 release passed:

- 60 native tests, formatting, and strict Clippy;
- clean locked `wasm32-wasip2` builds with byte-identical canonical output;
- actual ZeroClaw fuel, 256 MiB memory, and strict-signature enforcement;
- Gitleaks, Semgrep, OSV-Scanner, and RustSec review;
- 30/30 controlled local-Qwen conversations with 100% decision preservation;
- 20 live devnet analyses at 1,653 ms p95 under a six-RPC budget.

Evidence is indexed in [docs/TEST_MATRIX.md](docs/TEST_MATRIX.md) and
[docs/AGENT_BEHAVIOR_TESTS.md](docs/AGENT_BEHAVIOR_TESTS.md).

## Security

Read [SECURITY.md](SECURITY.md) before production use. The report is advisory:
simulation and RPC state are point-in-time observations, unknown protocols can
reduce coverage, and signing outside ZeroClaw is beyond the plugin's control.
Keep `fail_closed=true`, use strict plugin signatures, verify published
SHA-256 values, and never provide a seed phrase or private key.

## License and attribution

Project code is MIT licensed. Third-party software and model attributions are
listed in [THIRD_PARTY_NOTICES.md](THIRD_PARTY_NOTICES.md). Solana, ZeroClaw,
Ollama, and Qwen names belong to their respective owners; no endorsement is
implied.
