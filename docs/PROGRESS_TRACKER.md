# Progress Tracker

| Milestone | Status | Gate | Validation |
|---|---|---|---|
| M0 — Discovery and compatibility | Complete | G0 passed | Functional validation passed; Cargo.lock committed in `a2c9145` |
| M1 — Foundation and contracts | Complete | G1 passed | 12 tests, strict clippy, WASI build, host response |
| M2 — RPC client | Complete | G2 passed | 5 method contracts + failure matrix + G0 live WASI HTTP |
| M3 — Parsing and ALT | Complete | G3 passed | Legacy/v0 equivalence, ALT order/fail-closed, property test |
| M4 — Decoders | Complete | G4 passed | Built-ins, token state/TLV, malformed and unknown coverage |
| M5 — Simulation and deltas | Complete | G5 passed | State fetches, return evidence, deltas, CPI, close/rent, and fees |
| M6 — Intent, policy and risk | Complete | G6 passed | All 54 rule IDs, family positive cases, clean negatives, and fail-closed precedence |
| M7 — Report and ZeroClaw integration | Complete | G7 passed | Base64/signature, allow-review-block, critical/error/custody agent evidence |
| M8 — Hardening | Complete | G8 passed | 60 tests, security scans, 30/30 Qwen matrix, p95 1,653 ms, reproducible RC |
| M9 — Documentation and submission | In progress | G9 pending | Public v0.1.0 package/demo and all URL/hash checks complete; M9.5 model-independent onboarding plus zero-config mainnet/devnet read access validated; authenticated Discord/Superteam submission and a new signed release remain pending |

Milestones are marked complete only after every gate validation passes.
