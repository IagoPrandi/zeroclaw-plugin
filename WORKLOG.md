# Worklog

## 2026-07-27 — M0 discovery baseline

- Opened and read `PRD.md` in full before implementation.
- Opened `AGENTS.md`.
- Inspected `.claude/skills` and used the relevant `solana-dev` hub, its core
  skill, and its Token-2022 reference.
- Preserved pre-existing deleted skill files and the pre-existing
  `AGENTS.md` modification.
- Reviewed the live competition listing and captured deadline, submission
  format, judging weights, custody expectations, and Tier 3 plugin guidance.
- Pinned ZeroClaw v0.8.3 at commit
  `24476b71d33eb1672a9495a7ce3d155377a60ce8`.
- Read the pinned tool-plugin guide and WIT definitions.
- Installed the `wasm32-wasip2` Rust target.
- Started downloading the required local Ollama model `qwen3.5:9b`.
- Added the M0 component/dependency compatibility baseline and tracking docs.
- Corrected the Rust pin from 1.91.1 to the ZeroClaw v0.8.3 minimum, 1.96.1,
  after the host build's MSRV validation rejected the older toolchain.
- Validated the plugin with 4 native tests, strict clippy, and an optimized
  `wasm32-wasip2` build.
- Completed the `qwen3.5:9b` download and verified that it emits a schema-valid
  native Ollama tool call with no cloud provider.
- Recorded the reference hardware, model storage, context, and cold tool-call
  observations.
- Completed the mandatory M0 agent-skill assessment with a `DO NOT CREATE`
  decision because the existing Solana skill already covers the reusable
  material and host validation is still release-specific.

## 2026-07-28 — M0 Gate G0

- Built ZeroClaw v0.8.3 at the pinned commit with
  `plugins-wasm,plugins-wasm-cranelift`.
- Re-ran Rust 1.96.1 format, 4 native tests, strict clippy, and the optimized
  `wasm32-wasip2` build.
- Recorded the G0 WASM size as 330,126 bytes and SHA-256 as
  `02476df52376ba174e5556f9abc70df02bd6df5013d0fa1078eb15dd5f4c7b46`.
- Installed and inspected the plugin in an isolated real ZeroClaw config.
- Verified a local-only no-tool response from `qwen3.5:9b`.
- Verified Qwen selected the plugin with schema-valid arguments.
- Verified the component received host-injected `__config` and completed a
  live `getHealth` call to Solana devnet through `waki`.
- Pointed the only Ollama alias at an unavailable local port and confirmed an
  explicit exit-code-1 error with no fallback.
- Reviewed the completed milestone for inconsistencies and approved Gate G0.

M0 is complete. M1 started only after all G0 validations passed.

## 2026-07-28 — M1 foundation and Gate G1

- Added repository license, security/contribution guidance, bootstrap/build/check
  scripts, and initial CI.
- Implemented the independent address type, strict input/config types, complete
  public JSON Schema, canonical output structures, typed error envelope, and
  per-analysis budget.
- Kept `__config` reserved and absent from the model-visible schema.
- Added a conservative, fail-closed foundation report and golden assertions.
- Passed 12 native tests, strict clippy, formatting, and the optimized WASI
  component build.
- Installed the updated component in the pinned host and received a controlled
  validation response through its tool interface.
- Reviewed threshold precedence, endpoint restrictions, decimal precision,
  error redaction, and public-schema consistency.
- Completed the M1 skill assessment and approved Gate G1.

## 2026-07-28 — M2 RPC and Gate G2

- Implemented bounded POST transport with monotonic IDs, timeout propagation,
  response limits, HTTP/JSON-RPC validation, call accounting, and one 429 retry.
- Added typed contracts for all five required RPC methods.
- Preserved `getMultipleAccounts` ordering/null semantics and enforced its
  100-address cap.
- Covered success, RPC error, wrong ID, 429 exhaustion, timeout category,
  oversized response, null result, and every method in 17 passing tests.
- Rebuilt the WASI component and retained the G0 live-devnet proof for the same
  `waki` transport boundary.
- Reviewed endpoint provenance and payload-redaction rules, recorded the M2
  skill assessment, and approved Gate G2.

## 2026-07-28 — M3 parsing and Gate G3

- Implemented bounded Base64/wire decoding and full structural sanitization.
- Normalized signatures, message hash, blockhash, fee payer, account flags,
  instructions, and legacy/v0 versions behind `Address32`.
- Decoded ALT state and metadata, resolved writable then readonly loaded keys
  in canonical order, and rejected missing, inactive, malformed, or
  out-of-range tables.
- Proved equivalent legacy/v0 logical models and arbitrary input non-panicking
  behavior in 21 passing tests.
- Integrated normalized participants and coverage into the core report.
- Re-ran strict clippy and the optimized WASI build, reviewed index arithmetic
  and fail-closed lookup behavior, completed the skill assessment, and approved
  Gate G3.

## 2026-07-28 — M4 decoders and Gate G4

- Added a static registry for System, Compute Budget, SPL Token, Token-2022,
  Associated Token Account, Memo, Address Lookup Table, and Upgradeable Loader.
- Covered every required built-in discriminant and critical token operation.
- Preserved amount precision as decimal strings and represented authority,
  delegate, close-destination, and mint/account roles through details and
  resolved account order.
- Added prioritized Token-2022 TLV extension discovery, including transfer fee,
  hook, permanent delegate, confidential, default state, non-transferable,
  pointers, CPI guard, memo transfer, and immutable owner types.
- Preserved unknown programs with instruction index, accounts, byte length, and
  SHA-256 evidence.
- Passed 27 tests, strict clippy, and WASI release build; reviewed malformed
  data behavior, completed the skill assessment, and approved Gate G4.

## 2026-07-28 — M5 effects and Gate G5

- Parsed simulation and confirmed execution status without conflating success
  with safety.
- Reconciled signed SOL deltas and raw token deltas by account index, mint,
  owner, and token program.
- Captured fee, compute units, bounded logs, simulation errors, and confirmed
  errors.
- Decoded inner instructions through the same registry while preserving
  top-level index, inner index, and stack height.
- Added checked priority-fee ceiling arithmetic and durable-nonce detection.
- Passed 31 tests, strict clippy, and WASI build; reviewed overflow and
  inconsistent-RPC paths, completed the skill assessment, and approved Gate G5.

## 2026-07-28 — M6 risk and Gate G6

- Implemented deterministic coverage, execution, transfer, authority/account,
  Token-2022, program, fee, and structured-intent rule families.
- Enforced operator hard caps before intent and prevented input from relaxing
  policy.
- Added canonical finding order, evidence, capped score, documented confidence,
  and block-first decision reduction.
- Kept malformed versions, unresolved ALTs, missing critical state, and
  inconsistent RPC as typed fail-closed errors.
- Integrated findings, bilingual summary choice, risk, score, confidence, and
  decision into the report.
- Manually reviewed critical rule mappings and precedence; passed 34 tests,
  strict clippy, and WASI build; completed the skill assessment and approved
  Gate G6.

## 2026-07-28 — M7 partial integration hardening

- Reopened `PRD.md` and retained the existing M0–M6 gate evidence.
- Enforced SOL operator and intent caps from decoded System instructions when
  candidate balance deltas are unavailable.
- Included CPI actions in deterministic risk evaluation and canonical execution
  ordering.
- Made simulation, fee-estimation, and unknown-program limitations explicit.
- Corrected confirmed fee decomposition to base 5,000 + priority 750 = total
  5,750 lamports in the live fixture.
- Added the versioned Guardian system prompt, structured payload-free host
  logging, and sanitized candidate-host evidence.
- Passed 37 native tests, strict clippy, formatting, and the optimized
  `wasm32-wasip2` build (692,330 bytes).
- Executed a confirmed devnet signature through the pinned ZeroClaw host and
  obtained the corrected report. The temporary external harness failed only
  because its old assertion still expected total fee in the base-fee field;
  M7 remains in progress and G7 is not marked complete.
- Stopped at the user's requested M0/M1 boundary. M0 and M1 were already fully
  validated and remain marked complete; no later milestone was newly closed.

## 2026-07-28 — PRD checklist reconciliation

- Updated `PRD.md` to mark only items backed by existing tests, builds, host
  runs, or recorded assessments.
- Corrected the Progress Tracker after finding that some milestone-level items
  had been declared complete while their detailed PRD checkboxes were not
  actually satisfied.
- Left the uncommitted Cargo lock gate, M4 account-state integration, M5 return
  data/state/close-rent work, and 13 missing M6 rule IDs open.
- Re-ran the confirmed devnet signature through the pinned ZeroClaw host with
  corrected fee assertions; the test passed and evidence was saved under
  `docs/evidence/host-e2e/`.

## 2026-07-28 — M4–M6 audit remediation

- Added bounded account-state acquisition for observed wallets and required
  SPL Token/Token-2022 accounts.
- Added owner compatibility, missing/malformed state, and inconsistent RPC
  evidence; decoded prioritized Token-2022 extensions from fetched TLV state.
- Captured simulation/confirmed return data as bounded length and SHA-256
  evidence, retained confirmed post-balances, and populated authority changes.
- Implemented executable paths for all 54 required PRD rule IDs, including
  reserve, fee-payer, undeclared asset/outflow, delegate, upgrade authority,
  close-rent destination, and account-owner rules.
- Corrected a cross-asset bug where token deltas could be counted as SOL
  outflow.
- Passed 49 native tests, strict clippy, formatting, and the optimized WASI
  build (776,561 bytes).
- Re-ran both Base64 and confirmed-signature devnet flows through the pinned
  ZeroClaw host; both passed with canonical fee and coverage behavior.
- Reviewed the result for inconsistent fee totals, hidden limitations, action
  order, CPI risk coverage, and cross-asset arithmetic before restoring G4–G6
  to complete.
