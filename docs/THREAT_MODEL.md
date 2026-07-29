# Threat model

## Scope and security objective

Solana Transaction Guardian is a read-only ZeroClaw WASM tool. Its security
objective is to turn untrusted transaction bytes, public signatures, operator
configuration, and bounded Solana RPC responses into deterministic evidence
without signing, submitting, persisting secrets, or silently claiming safety
when critical analysis is incomplete.

## Assets

- Operator policy and RPC endpoint configuration.
- Integrity of the canonical `allow`, `review`, or `block` decision.
- Integrity and availability of transaction, account-state, and simulation
  evidence.
- User privacy: serialized transaction bytes, public signatures, and account
  contents should not leak through default logs.
- Host availability under hostile inputs or RPC responses.

Private keys and seed phrases are deliberately outside the accepted data
model. The plugin has no signing or submission capability.

## Trust boundaries

1. User/model arguments enter through the strict public JSON Schema and Serde
   models.
2. Operator configuration enters only through the host-injected `__config`.
3. The pinned ZeroClaw host grants only `config_read` and `http_client`.
4. Solana RPC is external and untrusted; every envelope, ID, status, size, and
   required field is validated.
5. Ollama/Qwen may choose and present the tool call but cannot change the
   canonical report produced by the WASM component.

## Threats and mitigations

| Threat | Mitigation | Residual risk |
|---|---|---|
| Caller injects RPC URL or forged `__config` | Unknown public fields are rejected; ZeroClaw strips caller `__config` and injects operator config | Compromised host configuration remains trusted |
| SSRF or redirect to a different origin | Endpoints are preconfigured aliases; plain HTTP is limited to localhost; Waki does not follow redirects and 3xx is rejected as non-success | A compromised configured HTTPS origin can return hostile data |
| Oversized Base64, HTTP response, account batch, logs, or report | Pre-decode size check, HTTP/body/output budgets, RPC-call budget, 100-account batches, bounded logs with explicit truncation | Maximum allowed responses still consume bounded CPU/memory |
| Malformed transaction, ALT, TLV, metadata, or JSON causes panic | Typed errors, checked indices/arithmetic, property tests, no `unsafe`, and lint-denied `unwrap`/`expect` | Parser dependencies may contain unknown defects |
| Unsupported version or unresolved ALT is treated as safe | Controlled rejection and fail-closed coverage rules | Future formats require an explicit upgrade |
| Unknown program/CPI hides dangerous behavior | Unknown instructions retain program, accounts, size, and digest; coverage becomes incomplete; policy determines review/block | No universal decoder exists |
| RPC inconsistency or simulation success is mistaken for safety | Execution, risk, coverage, confidence, and decision are separate fields; point-in-time limitations are explicit | State can change after analysis |
| Intent weakens operator hard caps | Operator policy evaluates independently and block precedence is deterministic | Incorrect operator policy can still be too permissive |
| LLM changes decision, omits findings, or asks for secrets | Versioned prompt, local-only model configuration, agent behavior matrix, literal-decision checks, and read-only refusal test | Model presentation remains probabilistic and must keep being tested |
| Cloud fallback leaks data | Reference config contains only localhost Ollama and no provider credentials or fallback | Local machine compromise is out of scope |
| Resource exhaustion in WASM | Host fuel, 256 MiB memory, table/instance caps; plugin-level byte/RPC/output budgets | Slow external RPC can consume configured timeout |
| Dependency compromise | Locked dependencies, RustSec audit, CI locked builds, minimal runtime tree | `bincode 1.3.3` is unmaintained but needed for the current Solana wire contract |

## Abuse cases explicitly refused

- Seed/private-key collection.
- Transaction signing, modification, submission, or broadcast.
- Arbitrary RPC endpoint selection in tool arguments.
- Treating missing evidence, execution success, or an LLM explanation as a
  guarantee of safety.

## Review result

Reviewed on 2026-07-29 against M8. No open critical threat was found. The
unmaintained `bincode` informational advisory and the point-in-time nature of
RPC/simulation remain documented limitations.
