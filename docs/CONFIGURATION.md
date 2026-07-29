# Configuration

ZeroClaw injects plugin configuration as strings under
`[[plugins.entries]].config`. The Guardian rejects missing, unknown input, or
malformed values; it does not infer an RPC endpoint from user input.

Use [config/zeroclaw.guardian.example.toml](../config/zeroclaw.guardian.example.toml)
as the versioned baseline.

## Local model and agent

```toml
[providers.models.ollama.local]
uri = "http://127.0.0.1:11434"
model = "qwen3.5:9b"
num_ctx = 8192
temperature = 0.0
think = false
timeout_secs = 600
max_tokens = 1000

[agents.guardian]
model_provider = "ollama.local"
risk_profile = "guardian"
runtime_profile = "guardian_local"
```

The reference profile contains no cloud provider or fallback. `temperature=0`
and `think=false` reduce presentation variance and ensure a visible answer
after tool execution. The deterministic plugin decision does not depend on
either setting.

## Signature policy

Shared or production profiles should use:

```toml
[plugins.security]
signature_mode = "strict"
trusted_publisher_keys = ["d743b2cd62da45564844b273760776c076642cec487700fdedfc601100e5c96d"]
```

The key is public, not a secret. Verify it from
[PUBLISHER_KEY.md](PUBLISHER_KEY.md), the signed manifest, and GitHub release
notes. Development profiles may temporarily use `disabled` only for a locally
built unsigned manifest.

## Required plugin fields

| Key | Validated reference | Constraint |
|---|---:|---|
| `rpc_endpoints_json` | `{"devnet":"https://api.devnet.solana.com"}` | non-empty cluster → HTTPS URL map; localhost HTTP allowed for tests |
| `allowed_clusters_json` | `["devnet"]` | non-empty subset of configured endpoints |
| `request_timeout_ms` | `5000` | 100–60,000 |
| `max_rpc_calls` | `6` | 1–64 |
| `max_http_response_bytes` | `2097152` | 1,024–16,777,216 |
| `max_transaction_bytes` | `1232` | 1–16,384 |
| `max_output_bytes` | `262144` | 1,024–1,048,576 |
| `fail_closed` | `true` | required boolean |
| `enable_simulation` | `true` | required boolean |
| `policy_version` | `default-1` | 1–64 bytes |

Keep `fail_closed=true` for value-bearing workflows. `enable_simulation=false`
is valid only when the resulting incomplete candidate coverage is acceptable
under policy.

## Optional policy fields

| Key | Default | Meaning |
|---|---:|---|
| `allowed_programs_json` | `[]` | base58 program addresses explicitly allowed |
| `blocked_programs_json` | `[]` | program addresses that force policy findings |
| `known_recipients_json` | `[]` | `{address,label}` entries, unique, label ≤100 bytes |
| `blocked_recipients_json` | `[]` | recipient addresses blocked by operator policy |
| `sol_out_review_lamports` | `100000000` | review threshold (0.1 SOL) |
| `sol_out_block_lamports` | `1000000000` | block threshold (1 SOL) |
| `priority_fee_review_lamports` | `100000` | priority-fee review threshold |
| `priority_fee_block_lamports` | `1000000` | priority-fee block threshold |
| `minimum_sol_reserve_lamports` | `10000000` | minimum observed-wallet reserve |
| `unknown_program_policy` | `review` | `none`, `review`, or `block` |
| `unresolved_alt_policy` | `block` | effect for unresolved lookup tables |
| `simulation_unavailable_policy` | `block` | effect for unavailable simulation |
| `token2022_transfer_hook_policy` | `review` | effect for transfer-hook behavior |
| `token2022_permanent_delegate_policy` | `block` | effect for permanent delegate |

Review thresholds must not exceed their corresponding block thresholds.
Integers are lamports/raw units; the plugin does not parse floating-point
currency.

## Mainnet

Mainnet is supported for read-only analysis but is not enabled in the
reference profile. Add it explicitly:

```toml
rpc_endpoints_json = "{\"devnet\":\"https://api.devnet.solana.com\",\"mainnet-beta\":\"https://YOUR_RPC\"}"
allowed_clusters_json = "[\"devnet\",\"mainnet-beta\"]"
```

Keep credentials in the host configuration or secret-management mechanism,
never in tool arguments, logs, examples, or committed files. A remote endpoint
must use HTTPS.

## Tool input

The public source object is exactly one of:

```json
{"type":"serialized","transaction_base64":"<BASE64>"}
```

```json
{"type":"confirmed","signature":"<BASE58_SIGNATURE>"}
```

Other fields are `cluster`, up to ten unique `observed_wallets`, optional
`expected_intent`, and `output_language` (`en` or `pt-BR`). Decimal limits in
intent are unsigned strings to preserve exact integer values.

The authoritative JSON Schema is
[tests/fixtures/tool-schema.json](../tests/fixtures/tool-schema.json) and is
equality-tested against the schema published by the WASM component.
