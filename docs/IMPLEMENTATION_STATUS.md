# Current state

- Current milestone: M9 — Documentation, demonstration, and submission.
- Current submilestone: M9.4 authenticated showcase/submission.
- Last completed task: published v0.1.0 and independently downloaded and
  verified every public release asset and link.
- Next executable task: post the prepared showcase in the ZeroClaw Discord
  `#solana-bounty` channel and submit the required Superteam form.
- Blocker: the showcase and form need the operator's authenticated Discord
  and Superteam accounts; no connector for either service is available in
  this workspace.
- Open risks: `bincode 1.3.3` is unmaintained with no compatible fixed release;
  the canonical release build must run in the pinned Linux container.
- Public release: <https://github.com/IagoPrandi/zeroclaw-plugin/releases/tag/v0.1.0>.
- Public demo:
  <https://github.com/IagoPrandi/zeroclaw-plugin/releases/download/v0.1.0/guardian-demo.mp4>.
- Stable release tag: `v0.1.0` at
  `e138f2d2b1547a72343f51a2fa5305956565458c`.
- Last commands executed: GitHub release publication, fresh public asset
  downloads, byte/hash comparison, and HTTP 200 checks for all public URLs.
- Test status: 60 native tests pass, including arbitrary-wire property tests,
  schema/golden contract checks, and RPC-backed v0/ALT lookup reuse; strict
  clippy passes on Rust 1.96.1.
- Build status: plugin release component passes for `wasm32-wasip2`
  (canonical Linux size 775,829 bytes; SHA-256
  `780d7a88aa09eadcb345a7bfa6fd58e80cd93de27baa425d71439d6987e5c7e5`).
- Performance status: 20/20 live devnet candidate analyses passed under the
  six-RPC budget; p95 was 1,653 ms against an 8,000 ms target.
- Evidence produced: competition requirements, ADRs, local Ollama lock, and
  successful native/pinned-host Qwen probes, live host execution, and
  sanitized allow/review/block/error/custody agent transcripts, an approved
  30-conversation local behavior matrix, reproducible-build evidence, and
  formal security scan results.
