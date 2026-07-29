# Architecture

## System boundary

The Guardian is a monolithic ZeroClaw WASI component with one public tool. It
does not run a sidecar, database, signer, wallet, or transaction relay.

```text
untrusted user text
  -> local Qwen presentation layer
  -> strict tool schema
  -> WASI adapter
  -> deterministic core
       input/config validation
       transaction parsing + ALT resolution
       instruction/state decoding
       simulation or confirmed-state acquisition
       effects + intent/policy comparison
       risk rules + report serialization
  -> bounded JSON report
```

The component can reach only two host capabilities declared in
`manifest.toml`:

- `config_read`, for its own resolved string configuration;
- `http_client`, for bounded JSON-RPC POST requests to operator-configured
  endpoints.

It has no ambient filesystem, environment, socket, process, key-store, signing,
or broadcast access.

## Modules

| Module | Responsibility |
|---|---|
| `lib.rs` | WIT binding and thin host adapter |
| `schema.rs` | one published ZeroClaw tool schema |
| `input.rs`, `config.rs`, `limits.rs` | strict trust-boundary validation |
| `rpc/` | synchronous bounded JSON-RPC client over WASI HTTP |
| `transaction/` | legacy/v0 parsing and ALT normalization |
| `decoders/` | built-in, token, Token-2022, compute, and unknown evidence |
| `state.rs`, `simulation.rs` | account state, simulation/confirmed effects, deltas, fees |
| `risk/` | deterministic 54-rule policy/intent engine |
| `output.rs`, `core.rs` | stable report contract and orchestration |

The WIT contract is copied from the pinned ZeroClaw v0.8.3 host under
`wit/v0/`. Dependencies that assume ambient sockets or a Tokio runtime are not
part of the WASM runtime graph.

## Analysis paths

### Serialized candidate

1. Validate Base64 and decoded-size bounds.
2. Parse legacy or v0 wire format.
3. Resolve every required ALT from the configured cluster.
4. Decode top-level actions and fetch bounded relevant account state.
5. Call `simulateTransaction` when enabled.
6. Reconcile predicted effects, CPI, logs, return data, and fees.
7. Compare with optional structured intent and operator policy.
8. Evaluate ordered deterministic rules and emit a report.

### Confirmed signature

1. Validate the base58 signature.
2. Fetch `getTransaction` with supported version 0.
3. Normalize the transaction, metadata, loaded addresses, and effects.
4. Decode actions, CPI, deltas, fees, logs, and return data.
5. Compare intent/policy, evaluate rules, and emit a report.

## Decision authority

Rule evaluation is pure Rust logic. Severity, score, `risk_level`, `decision`,
coverage, and confidence come only from the report. The LLM is outside this
boundary and is considered defective if it contradicts or suppresses a
blocking/high-severity result.

`allow` requires complete mandatory coverage and no rule requiring review or
block. Prerequisite, transport, parse, ALT, or coverage failures produce a
controlled error or fail-closed report rather than a silent positive result.

## Trust boundaries

| Boundary | Threat | Control |
|---|---|---|
| user → LLM | ambiguity, prompt injection, secret request | at most one tool call, clarification, custody refusal |
| LLM → tool | malformed or invented fields | published schema plus Serde `deny_unknown_fields` |
| host config → plugin | unsafe endpoint/policy | mandatory typed parsing, endpoint allow rules, bounds |
| plugin → RPC | SSRF, redirect, oversized/invalid response | configured endpoints only, no argument URL, no redirect following, byte/time limits |
| RPC → analysis | malicious or inconsistent data | strict decoding, checked arithmetic, explicit coverage/confidence |
| package → host | tampering/untrusted publisher | archive SHA-256 plus Ed25519 manifest signature and strict host mode |

See [THREAT_MODEL.md](THREAT_MODEL.md), [SECURITY_REVIEW.md](SECURITY_REVIEW.md),
and [ADR.md](ADR.md).
