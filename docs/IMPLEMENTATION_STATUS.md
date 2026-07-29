# Current state

- Current milestone: M9 — Documentation, demonstration, and submission.
- Current submilestone: M9.3 public demonstration and M9.4 submission.
- Last completed task: produced a deterministic signed package from two
  byte-identical canonical builds, installed it into a clean ZeroClaw profile,
  and validated strict signature handling plus all four demo fixtures.
- Next executable task: finish public release/demo links and run the final
  submission validation.
- Blockers: none.
- Open risks: `bincode 1.3.3` is unmaintained with no compatible fixed release;
  the canonical release build must run in the pinned Linux container.
- Last tagged commit: `461d4d5` (`v0.1.0-rc.2`), which supersedes RC1 after
  the v0/ALT correction.
- Last commands executed: two clean-container builds, deterministic packaging,
  clean ZeroClaw installation, strict-signature discovery, and the four-case
  demo-fixture host test.
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
