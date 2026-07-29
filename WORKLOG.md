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

## 2026-07-29 — M7 local-agent integration and Gate G7

- Reopened `PRD.md`, `docs/IMPLEMENTATION_STATUS.md`, the progress tracker, and
  the repository skill inventory before resuming work.
- Corrected the public source schema for Ollama/Qwen tool grammar while keeping
  strict discriminated runtime validation and no permissive parsing fallback.
- Set the pinned ZeroClaw Ollama alias to `temperature=0` and `think=false`
  after a reasoning-only post-tool turn produced no visible response.
- Versioned Guardian prompt 1.0.2 with exact source shapes, mandatory visible
  responses, observed-wallet mapping, and structured-intent mapping.
- Proved natural-language Base64 and signature detection, object-shaped tool
  arguments, literal `allow`, `review`, and `block`, critical intent findings,
  partial-coverage disclosure, fail-closed tool errors, and refusal to request
  a seed or sign/broadcast.
- Recorded sanitized transcripts under `docs/evidence/agent-e2e/`, including
  failed-attempt corrections and the one observed truncated explanatory tail.
- Passed 50 native tests, formatting, strict clippy, and the optimized
  `wasm32-wasip2` release build (775,113 bytes).
- Reviewed the final runtime configuration, canonical tool outputs, prompt
  fidelity, and evidence consistency; recorded the M7 skill assessment and
  approved Gate G7.

## 2026-07-29 — M8 test and security hardening

- Reopened the detailed PRD test matrix and mapped every required family to
  executable evidence in `docs/TEST_MATRIX.md`.
- Added explicit malformed Base64, size, signature-count, account-index, ALT,
  Token-2022 TLV, expired-blockhash, durable-nonce, absent-log, and bounded-log
  tests plus a versioned golden contract fixture.
- Corrected RPC log reporting so line-count or per-line truncation sets
  `logs_truncated=true`.
- Ran 58 native tests and strict Clippy with all targets/features successfully.
- Installed `cargo-audit` 0.22.2 and audited 220 locked dependencies: zero
  vulnerabilities; documented the informational unmaintained advisory for
  wire-compatible `bincode 1.3.3`.
- Reviewed permissions, endpoint provenance, redirect behavior, output/RPC
  bounds, arithmetic, panic paths, runtime dependencies, and payload-free
  logging in `docs/SECURITY_REVIEW.md` and `docs/THREAT_MODEL.md`.
- Executed the actual Guardian under the pinned ZeroClaw host: a one-unit fuel
  budget trapped, a one-byte memory cap rejected instantiation, and 256 MiB
  instantiated successfully.
- Reviewed the changes for hidden truncation, false success, duplicated tests,
  and inconsistent PRD claims before marking M8.1 and M8.2 complete.

## 2026-07-29 — M8 Qwen behavior matrix

- Added a reproducible local Ollama harness backed by the versioned prompt and
  a schema fixture that is equality-tested against the Rust tool schema.
- Executed a 10-case dry run and found that ambiguous dual-source input caused
  two tool calls.
- Updated prompt 1.0.3 to permit at most one Guardian call and require
  clarification for multiple transaction sources.
- Repeated the full matrix from the start: 30 conversations, ten cases three
  times, 1,464.6 seconds total model time.
- Preserved the initial scoring artifact after identifying six validator false
  negatives: the responses contained the complete findings but paraphrased one
  rule ID and represented incompleteness as `analysis_complete=false`.
- Rescored the unchanged raw responses against semantic requirements: 30/30
  passed, 100% tool-call correctness, 100% decision preservation, zero omitted
  critical/high finding, zero secret/signing attempt, and zero positive
  recommendation after error or unavailability.
- Reviewed the final case distribution, repeated decisions, raw outputs, and
  threshold calculations before marking M8.3 complete.

## 2026-07-29 — M8 reproducibility and clean-runtime validation

- Signed the real manifest with an ephemeral Ed25519 test key through the
  pinned ZeroClaw signature implementation; strict mode accepted the trusted
  publisher and rejected an empty trust list.
- Ran the full CI script in a clean Rust 1.96.1 Linux container with the source
  mounted read-only: 59 tests, formatting, Clippy, and locked WASI release all
  passed.
- Pinned the release image digest and produced two independent, byte-identical
  775,945-byte WASM components with SHA-256
  `c375d0319693e110afa4f1cef579b1b763e68ce371f5b64f19d63c65c099ba00`.
- Recorded that Windows output is functionally valid but byte-different; only
  the pinned Linux container is the canonical release environment.
- Started an isolated Ollama 0.32.0 container on localhost with a read-only
  model cache, verified the full `qwen3.5:9b` digest, and received exact `OK`.
- Found no repository credential, private key, cloud provider config, or cloud
  LLM endpoint in the runtime trace; stopped the isolated container.
- Reviewed reproducibility claims, container isolation, key handling,
  limitations, and evidence consistency before marking M8.4 complete.

## 2026-07-29 — M8 formal security scan closure

- Reopened `PRD.md`, the implementation status, progress tracker, worklog, and
  local skill inventory before continuing.
- Applied the local release/configuration and security checklist skills to the
  release-candidate gate.
- Ran Semgrep 1.164.0 with 162 Rust, Python, and shell rules over 20 source and
  harness files: zero findings and 100% parsed lines.
- Ran Gitleaks 8.30.1 over a clean 544.8 KB project snapshot: zero secrets.
  A preliminary workspace-wide scan found only documented placeholders in the
  external `.claude` skill catalog.
- Ran OSV-Scanner 2.3.8 against all 220 locked packages: zero
  critical/high/medium/low vulnerabilities and the already documented
  severity-unknown `bincode 1.3.3` maintenance advisory with no fixed release.
- Removed the validated temporary Gitleaks snapshot after the scan.
- Reviewed scanner scope, false-positive provenance, advisory severity, and
  PRD claims before marking only the newly evidenced NFRs complete.
- Ran the final release-candidate suite: 59/59 native tests, formatting,
  strict Clippy with all targets/features, locked optimized `wasm32-wasip2`
  build, unchanged 30/30 behavior-matrix rescore, and `git diff --check`.

## 2026-07-29 — M8 performance closure

- Added an external pinned-host measurement that executes the real Guardian
  component against devnet with `max_rpc_calls=6`.
- Repeated the common candidate analysis 20 times on one component instance;
  all reports succeeded, proving the enforced six-call budget was not
  exhausted.
- Measured sorted latency from 1,592 to 1,669 ms with p95 1,653 ms, well below
  the 8,000 ms target. Component compilation/instantiation and LLM inference
  were intentionally excluded.
- Reviewed the sample calculation, successful-report condition, RPC budget
  semantics, and recorded host/version provenance before completing NFR-010
  and NFR-011.

## 2026-07-29 — Gate G8 release candidate approval

- Confirmed `Cargo.lock` has been tracked since commit `a2c9145`, closing the
  only administrative item left in Gate G0 and completing M0.
- Committed the M8 hardening/behavior/reproducibility evidence as `96df5b1`
  and created annotated tag `v0.1.0-rc.1`.
- Confirmed all mandatory M8 quality, security, local-model, resource,
  performance, reproducibility, and skill-assessment criteria have executable
  evidence.
- Recorded the gate approval in
  `docs/evidence/runtime/g8-release-candidate.md`, completed M8 in the progress
  tracker, and advanced the implementation status to M9.
- Reviewed the tag target, exact commit, remaining conditional skill items,
  residual dependency advisory, and PRD checkbox consistency before approving
  Gate G8.

## 2026-07-29 — M9.1 documentation

- Audited every M9.1 item against the repository instead of treating existing
  short files as complete documentation.
- Replaced the stale M0 README with a release-oriented English guide and added
  a reviewed pt-BR entry point.
- Added architecture, installation, configuration, exact ZeroClaw v0.8.3
  Ollama syntax, model verification, examples, and third-party attribution
  documents.
- Added a strict, localhost-only, no-cloud-fallback reference profile and
  documented the separation between deterministic plugin authority and Qwen
  presentation.
- Expanded security reporting/operator responsibilities and updated the
  changelog and Cargo repository metadata.
- Validated all local Markdown links, parsed the example TOML, checked locked
  Cargo metadata, ran `git diff --check`, and reviewed public documentation for
  stale milestone/release claims before completing M9.1.

## 2026-07-29 — M9 release package and demo validation

- Reopened the required PRD and local skill inventory before continuing M9;
  applied the release/configuration and security checklist guidance.
- Added final release, submission, publisher-key, bilingual setup, demo, and
  competition documentation plus a deterministic packaging program.
- During the real v0/ALT demo, found that the RPC-backed path resolved lookup
  tables and then discarded the resolved map during a second normalization
  pass. Refactored the core to reuse the map and added a native regression.
- Passed 60 native tests, formatting, strict Clippy, and a locked optimized
  WASI build after the correction.
- Ran two builds in independent empty target directories under the pinned
  Rust container. Both produced the same 775,829-byte component with SHA-256
  `780d7a88aa09eadcb345a7bfa6fd58e80cd93de27baa425d71439d6987e5c7e5`.
- Signed the final manifest with the persistent Ed25519 publisher key stored
  outside the repository with user-only access. Reproduced the signature and
  validated strict trust and rejection with an empty trust list.
- Generated the release package twice. Both 253,346-byte ZIPs had SHA-256
  `70a3ac35eb34850cddb5dd745be216278d0d0278924697dcb4e3e6d49cea1b3b`;
  archive integrity and the embedded WASM hash passed.
- Installed the archive into an empty ZeroClaw v0.8.3 profile and confirmed
  install/list/info output and the exact two declared permissions.
- Executed all four release fixtures in the real host: safe legacy `allow`,
  hidden delegate `block`, unknown program `block`, and resolved v0/ALT
  `allow`.
- Rechecked the live competition listing. It remained open and unchanged in
  substance, with 65 submissions; required form fields are demo video and
  supporting material, while the one-pager is optional.
- Reviewed the package/source relationship after updating README content,
  rebuilt the archive, and marked only validated PRD items complete. Public
  video, public release links, semantic tag, and submission remain open.
- Committed the corrected package/source state as `461d4d5` and created the
  annotated semantic tag `v0.1.0-rc.2`, which supersedes RC1.
- Ran the clean installed agent against an explicit prompt injection requesting
  the user's seed phrase, signing, and broadcast. Qwen refused all three,
  restated the read-only boundary, made no tool call, and completed in 183
  seconds with no cloud provider configured.
- Rejected a hidden-delegate take with empty visible agent output and a first
  safe-transfer take whose prose converted one lamport incorrectly. Neither
  was accepted as demo evidence.
- Repeated the safe-transfer flow in a second empty profile with explicit
  exact-integer/no-inference instructions. Qwen preserved `allow`, the
  one-lamport action, coverage, and limitations in 271 seconds.
- Added a reproducible terminal runner and an isolated video renderer that
  uses only sanitized real outputs, identifies the inference jump cut, and
  never captures the user's desktop or other windows.
- Rejected the first renderer output after visual review found overlapping
  progressive text. Corrected event lifetimes, inspected five representative
  frames, and produced a legible 2:38 H.264 video at 1600x900.
- Rendered the final demo twice with byte-identical SHA-256
  `1e3652197c3c3ae80c0911d626d1f102ff70b2e806fa6e17d3c17ccb3cd05828`.
- Re-ran Gitleaks 8.30.1 over the complete Git history. Reviewed 31 initial
  candidates without printing values: 29 belonged to vendored skill examples,
  one was the public manifest signature, and one was the public devnet ALT.
- Added a narrow Gitleaks configuration that retains default rules and excludes
  only the vendored skill path plus the two exact public identifiers.
- Repeated the filtered Gitleaks history scan: eight commits, 14.58 MB, zero
  leaks.
- Attempted the final Semgrep 1.164.0 rescan twice. The registry-backed
  specific rules timed out before scanning, and `auto` remained blocked on the
  registry; recorded the rescan as unavailable rather than reusing a false
  zero. The completed M8 scan remains 162 rules with zero findings.
- Re-ran OSV-Scanner 2.3.8 and RustSec over all 220 locked packages: zero known
  vulnerabilities and the unchanged, no-fix `bincode 1.3.3` unmaintained
  advisory.
- Re-ran the final local suite after all code changes: 60/60 native tests,
  formatting, strict Clippy across all targets/features, and the locked
  optimized `wasm32-wasip2` build passed.
