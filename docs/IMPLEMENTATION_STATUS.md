# Current state

- Current milestone: M9 — Documentation, demonstration, and submission.
- Current submilestone: M9.4 authenticated showcase/submission; M9.5 user
  onboarding is complete.
- Last completed task: removed the Ollama/Qwen product dependency, added safe
  zero-config devnet defaults and a package installer that preserves the
  user's existing ZeroClaw model/provider.
- Next executable task: create a new signed release for the M9.5 changes, then
  post the prepared showcase in the ZeroClaw Discord `#solana-bounty` channel.
- Blocker: the showcase requires the operator's authenticated Discord account;
  no Discord connector is available in this workspace.
- Open risks: `bincode 1.3.3` is unmaintained with no compatible fixed release;
  the canonical release build must run in the pinned Linux container.
- Public release: <https://github.com/IagoPrandi/zeroclaw-plugin/releases/tag/v0.1.0>.
- Public walkthrough:
  <https://github.com/IagoPrandi/zeroclaw-plugin/releases/download/v0.1.0/guardian-demo-walkthrough.mp4>.
- Stable release tag: `v0.1.0` at
  `e138f2d2b1547a72343f51a2fa5305956565458c`.
- Last commands executed: regenerated the walkthrough, inspected representative
  frames, calculated its SHA-256, uploaded it to the public GitHub release,
  and matched the release API digest with the local artifact.
- Test status: 61 native tests pass, including zero-config regression,
  arbitrary-wire property tests, schema/golden contract checks, and RPC-backed
  v0/ALT lookup reuse; strict Clippy and the PowerShell installer test pass in
  the pinned Rust 1.96.1 Linux container.
- Build status: the current source builds for `wasm32-wasip2` in the pinned
  Linux container. The published v0.1.0 hash remains historical; a new signed
  release is required before publishing a new hash.
- Performance status: 20/20 live devnet candidate analyses passed under the
  six-RPC budget; p95 was 1,653 ms against an 8,000 ms target.
- Evidence produced: competition requirements, ADRs, model-independent
  onboarding validation, local Ollama lock, and
  successful native/pinned-host Qwen probes, live host execution, and
  sanitized allow/review/block/error/custody agent transcripts, an approved
  30-conversation local behavior matrix, reproducible-build evidence, and
  formal security scan results.
