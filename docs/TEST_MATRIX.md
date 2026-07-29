# M8 test matrix

Date: 2026-07-29

All automated cases run natively unless marked host/devnet. Combined decoder
tests intentionally cover multiple discriminants in one test while asserting
every required variant.

| Family | Required cases | Evidence |
|---|---|---|
| Parsing | legacy, v0, invalid signature count, invalid Base64, encoded/decoded oversize, unsupported/malformed future wire, invalid account index, missing/inactive/out-of-range ALT | `spike::tests`, `transaction::tests` |
| System | transfer, assign, create account, nonce advance/authorize/withdraw | `decoders::decodes_all_required_builtin_discriminants`, durable nonce test |
| Token | transfer, transferChecked, approve/revoke, setAuthority, mint/burn, freeze/thaw, close, malformed | required-token decoder matrix and malformed decoder paths |
| Token-2022 | transfer fee, hook, permanent delegate, immutable owner, confidential partial coverage, malformed/duplicate TLV | decoder/state tests and Token-2022 risk-family test |
| Compute | CU limit, CU price, rounding, overflow, duplicates | decoder, simulation, and fee/execution risk-family tests |
| Simulation | success, program error, expired blockhash, durable nonce, inner instructions, absent/bounded logs, oversized RPC response | core, simulation, and RPC tests |
| Risk | positive paths for all 54 rule IDs, clean negative, stable order, block precedence, fail-closed coverage, hard-cap precedence | `risk::tests` and typed prerequisite-error tests |
| Security | endpoint/`__config` rejection, HTTP/output bounds, arbitrary input, payload-free logs, forbidden dependency scan | core/RPC/property tests and `SECURITY_REVIEW.md` |
| Golden | fail-closed empty candidate contract | `tests/golden/fail_closed_empty.json` |
| Devnet | candidate Base64 and confirmed signature through pinned ZeroClaw host | `docs/evidence/host-e2e/` |
| Resources | release size, host fuel trap, host memory rejection/256 MiB success | `docs/SECURITY_REVIEW.md` |
| Performance | 20 successful candidate analyses under a six-RPC budget; p95 1,653 ms | `docs/evidence/runtime/performance.md` |

Latest native result: 59 passed, 0 failed, 0 ignored. Strict formatting and
Clippy pass on Rust 1.96.1.
