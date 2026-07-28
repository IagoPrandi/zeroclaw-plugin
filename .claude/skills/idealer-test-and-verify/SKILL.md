---
name: idealer-test-and-verify
description: >-
  How to run tests and verifications correctly in the idealer monorepo (TS apps
  apps/worker, apps/api, apps/web + the Rust Solana program in contracts/solana)
  before claiming any work done. Use whenever verifying a change, running
  typecheck/test/lint/build, validating contract or SBF changes, or performing
  live devnet verification (settlement, claims, input seeding). Covers the
  per-package verification ladder, node:test conventions, Rust/SBF testing via
  Docker, the no-mocks / test-only-gating rule, live devnet dry-run then broadcast
  with on-chain plus API plus audit reconciliation and idempotency, and the
  Windows/Docker gotchas that silently block correct verification.
---

# Idealer — Test & Verify

Verification is not optional and not "it compiles". A task is **done** only when
the relevant checks below are green and you have observed the real behavior.
**Never claim done until verified. If a check fails, say so and paste the output.**

## The verification ladder (run in this order)

For every package you touched, run all four and require each green before the next:

```bash
pnpm --filter @idealer/<pkg> typecheck
pnpm --filter @idealer/<pkg> test
pnpm --filter @idealer/<pkg> lint     # eslint --max-warnings=0: warnings fail
pnpm --filter @idealer/<pkg> build
```

Packages: `@idealer/api`, `@idealer/worker`, `@idealer/web`, plus shared
`@idealer/game-engine` / `@idealer/rule-runtime` / `@idealer/rule-compiler` /
`@idealer/rule-schema`. A change in `apps/*` that uses a shared package means you
verify both. Lint is a real gate (`--max-warnings=0`): one warning is a failure —
fix it, don't ignore it.

Adding a required field to a shared type (e.g. a config interface) breaks every
test fixture that builds that object literally — typecheck lists them; fix the
fixtures, don't loosen the type.

## TS test conventions (`node:test`)

- Tests run as `node --test --import tsx "src/**/*.test.ts"` — co-located
  `*.test.ts` next to the source, `import { test } from "node:test"` + `assert`
  from `node:assert/strict`.
- Prefer **pure functions + injected dependencies** so logic is testable without a
  cluster: planners/decoders/instruction-builders are unit-tested directly;
  orchestrators take a `deps` object and tests pass fakes (`as unknown as <Iface>`).
- When you fix a bug, add a regression test that fails before the fix.
- Determinism: code must not rely on wall-clock/random in a way that breaks replay;
  pass timestamps in and vary by index instead.

## Rust / Solana contract testing

The Windows host has `cargo` but **no** `anchor`/`solana`/`cargo-build-sbf`; all
contract work runs in Docker. See `contracts/solana/README.md` for the exact
`docker run` invocation (mounts `idealer-solana-home` for the toolchain + deploy
keypairs).

- **Native cargo checks** (compile + `core.rs`/domain unit tests):
  `pnpm --filter @idealer/solana-programs typecheck|lint|test` (runs in the pinned
  `rust:1.91-bookworm` container via `scripts/run-cargo.mjs`).
- **Clippy must pass for BOTH feature sets** when a change is `#[cfg(feature=...)]`-
  gated: the default build *and* `--all-features` (enables `devnet-test`/`sb-devnet`).
  The `lint` script uses `--all-features`; also run a default-feature `clippy` so the
  production `cfg(not(...))` branch is checked.
- **SBF instruction tests** are `#[ignore]` (need the SBF build) and are **flaky in
  parallel** — always `--test-threads=1`. Run via
  `contracts/solana/scripts/sbf.sh test-sbf` inside the toolchain container.
- A contract change is only verified after an **SBF build succeeds** and (for a
  devnet change) the **program is upgraded and re-read on-chain**.

## No mocks, no fallbacks (project rule)

Production paths must not contain mocks or fallbacks that hide errors. Test-only
behavior must be **explicitly gated and excluded from production**:

- Rust: a cargo feature (`devnet-test`) — e.g. a short devnet-only constant via
  `#[cfg(feature = "devnet-test")]`, with the real value under `#[cfg(not(...))]`.
- TS: an env flag (`IDEALER_TURN_MANUAL_CONTROL_ENABLED`, `IDEALER_*_BROADCAST`,
  `IDEALER_SETTLEMENT_AUTOMATION_ENABLED`) that defaults OFF.

A test that passes only because a production path silently fell back to a mock is
not a passing test — it is a hidden defect.

## Live devnet verification

Local green is necessary but **not sufficient** for anything that touches the
chain (settlement, draw, election, claims, input seeding). Before broadcasting
anything irreversible, and to reconcile on-chain ↔ API ↔ audit state, read
[references/devnet-verification.md](references/devnet-verification.md).

That file also documents the **Windows/Docker gotchas that silently make
verification lie** (swallowed command output, stale running code, RPC timeouts,
wrong/slow LLM model). Read it before running any worker container,
`docker compose`, or live operator script — otherwise you will misread empty
output as success or a stale container as a passing test.
