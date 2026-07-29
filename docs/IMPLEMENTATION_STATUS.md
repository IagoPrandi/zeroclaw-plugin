# Current state

- Current milestone: M8 — Hardening and validation.
- Current submilestone: Gate G8 release-candidate versioning.
- Last completed task: completed M8.4 with clean-container CI, byte-identical
  canonical WASM builds, strict-signature host validation, isolated local
  Ollama/Qwen verification, and formal SAST/SCA/secrets scans.
- Next executable task: run the final validation suite and version the release
  candidate.
- Blockers: none.
- Open risks: `bincode 1.3.3` is unmaintained with no compatible fixed release;
  the canonical release build must run in the pinned Linux container.
- Last passing commit: not yet committed.
- Last commands executed: Gitleaks, Semgrep, OSV-Scanner, clean-container CI,
  strict-host resource/signature tests, and the 30-conversation Qwen matrix.
- Test status: 59 native tests pass, including arbitrary-wire property tests
  and schema/golden contract checks; strict clippy passes on Rust 1.96.1.
- Build status: plugin release component passes for `wasm32-wasip2`
  (canonical Linux size 775,945 bytes; SHA-256
  `c375d0319693e110afa4f1cef579b1b763e68ce371f5b64f19d63c65c099ba00`).
- Evidence produced: competition requirements, ADRs, local Ollama lock, and
  successful native/pinned-host Qwen probes, live host execution, and
  sanitized allow/review/block/error/custody agent transcripts, an approved
  30-conversation local behavior matrix, reproducible-build evidence, and
  formal security scan results.
