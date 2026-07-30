# Configuration

Guardian does not configure or select an LLM. ZeroClaw supplies the tool to
the agent already selected by the operator, regardless of its provider or
model. The plugin receives only its own optional configuration under
`[[plugins.entries]].config`.

## Zero-config defaults

An installed Guardian with no `plugins.entries` configuration uses this
explicit, safe baseline:

| Setting | Default |
|---|---|
| Clusters and RPCs | `mainnet-beta` via `https://api.mainnet-beta.solana.com`; `devnet` via `https://api.devnet.solana.com` |
| Fail-closed | `true` |
| Candidate simulation | enabled |
| RPC timeout | 5,000 ms |
| RPC budget | 6 calls |
| Unknown program | `review` |
| Unresolved ALT / unavailable simulation | `block` |

No omitted value is guessed from tool input. If an operator supplies a value,
it is validated; malformed or unsafe values return `INVALID_CONFIG`.

## Custom policy or RPC

Use [config/zeroclaw.guardian.example.toml](../config/zeroclaw.guardian.example.toml)
as a fragment to add to an existing profile. It deliberately contains no
`[providers.models.*]` or `[agents.*]` sections.

The baseline fields that may be overridden are shown below. Any omitted field
continues to use the zero-config baseline; an explicitly supplied value is
validated.

| Key | Constraint |
|---|---|
| `rpc_endpoints_json` | non-empty cluster → HTTPS URL map; localhost HTTP allowed only for tests |
| `allowed_clusters_json` | non-empty subset of configured endpoints |
| `request_timeout_ms` | 100–60,000 |
| `max_rpc_calls` | 1–64 |
| `max_http_response_bytes` | 1,024–16,777,216 |
| `max_transaction_bytes` | 1–16,384 |
| `max_output_bytes` | 1,024–1,048,576 |
| `fail_closed` | boolean |
| `enable_simulation` | boolean |
| `policy_version` | 1–64 bytes |

Mainnet is enabled by default. Replace the public endpoint only when an
operator needs a private RPC or stronger availability guarantees:

```toml
[plugins.entries.config]
rpc_endpoints_json = "{\"devnet\":\"https://api.devnet.solana.com\",\"mainnet-beta\":\"https://YOUR_RPC\"}"
allowed_clusters_json = "[\"devnet\",\"mainnet-beta\"]"
```

Keep private RPC credentials in ZeroClaw's secret-management mechanism, never
in tool input, logs, examples, or committed files.

## Optional policy fields

| Key | Default |
|---|---:|
| `allowed_programs_json` | `[]` |
| `blocked_programs_json` | `[]` |
| `known_recipients_json` | `[]` |
| `blocked_recipients_json` | `[]` |
| `sol_out_review_lamports` | `100000000` |
| `sol_out_block_lamports` | `1000000000` |
| `priority_fee_review_lamports` | `100000` |
| `priority_fee_block_lamports` | `1000000` |
| `minimum_sol_reserve_lamports` | `10000000` |
| `unknown_program_policy` | `review` |
| `unresolved_alt_policy` | `block` |
| `simulation_unavailable_policy` | `block` |
| `token2022_transfer_hook_policy` | `review` |
| `token2022_permanent_delegate_policy` | `block` |

Review thresholds must not exceed their block thresholds. Amounts are
lamports/raw units represented as integer strings; floating-point currency is
not accepted.

## Tool input

The model supplies one transaction source, an allowed cluster, and optional
observed wallets or expected intent. RPC URLs never come from the model:

```json
{"source":{"type":"serialized","transaction_base64":"<BASE64>"},"cluster":"devnet"}
```

The authoritative JSON Schema is
[tests/fixtures/tool-schema.json](../tests/fixtures/tool-schema.json).
