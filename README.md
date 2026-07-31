# Solana Transaction Guardian

Solana Transaction Guardian is a read-only ZeroClaw tool plugin that decodes,
simulates, and evaluates Solana transactions before an agent or user trusts
them. It accepts either a serialized transaction or a confirmed signature and
returns one deterministic JSON report with actions, participants, balance
deltas, fees, coverage, findings, and an `allow`, `review`, or `block`
decision.

The Rust/WASM plugin is the security authority. It works with the ZeroClaw
agent and model the user already selected: the model can select the tool,
build arguments, and explain the report, but it cannot change the canonical
decision.

> Custody tier T0: the Guardian has no private-key, signing, or transaction
> submission capability.

[Português do Brasil](README.pt-BR.md)

[Download v0.1.0](https://github.com/IagoPrandi/zeroclaw-plugin/releases/tag/v0.1.0)
·
[Watch the 2:46 phone-and-terminal walkthrough](https://github.com/IagoPrandi/zeroclaw-plugin/releases/download/v0.1.0/guardian-demo-walkthrough.mp4)

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
     -> user's configured model/provider
        -> solana_transaction_guardian (WASM)
           -> safe mainnet/devnet RPC defaults or operator-configured Solana RPC
           -> deterministic report
        -> faithful presentation of that report
```

Guardian neither selects nor configures an LLM provider. Ollama/Qwen is the
reproducible reference environment only; it is not a prerequisite. See
[LLM runtime evidence](docs/LLM_RUNTIME.md).

## Quick start

Prerequisites: a working ZeroClaw profile with the model/provider of the
user's choice, and a ZeroClaw build that includes the WASM plugin host. Confirm
the latter before downloading the plugin:

```powershell
zeroclaw plugin --help
```

Download the release archive, verify its SHA-256, extract it, enable plugins,
and install the extracted plugin directory. These commands are for the current
`v0.1.0` release:

```powershell
Invoke-WebRequest `
  -Uri "https://github.com/IagoPrandi/zeroclaw-plugin/releases/download/v0.1.0/solana-transaction-guardian-0.1.0.zip" `
  -OutFile .\solana-transaction-guardian-0.1.0.zip

Get-FileHash .\solana-transaction-guardian-0.1.0.zip -Algorithm SHA256
# Expected: 70a3ac35eb34850cddb5dd745be216278d0d0278924697dcb4e3e6d49cea1b3b

Expand-Archive .\solana-transaction-guardian-0.1.0.zip -DestinationPath .
zeroclaw config set plugins.enabled true
zeroclaw plugin install .\solana-transaction-guardian-0.1.0
zeroclaw plugin info solana-transaction-guardian
```

`plugin info` confirms that the tool is available to the agent in that
ZeroClaw profile. The v0.1.0 archive does not contain `install-guardian.ps1`;
install it with the native `zeroclaw plugin install` command shown above.

No LLM configuration, prompt copy, or Guardian profile is needed. The default
is ready for fail-closed mainnet/devnet analysis. Then ask the user's existing
agent:

```bash
zeroclaw agent --agent <your-agent> \
  --message "Analyze this devnet transaction: <BASE64>"
```

Use the full [installation guide](docs/INSTALLATION.md) for strict signatures,
source builds, and alternate profile paths. Private RPCs and custom policies
are opt-in and documented in [docs/CONFIGURATION.md](docs/CONFIGURATION.md).

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
- 30/30 controlled local-Qwen reference conversations with 100% decision preservation;
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
