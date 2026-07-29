# Current state

- Current milestone: M8 — Hardening and validation.
- Current submilestone: M8.3 controlled Qwen behavior matrix.
- Last completed task: passed Gate G7 with local ZeroClaw/Qwen Base64 and
  signature detection, schema-correct arguments, literal allow/review/block,
  critical finding, partial coverage, tool-error, and read-only custody
  evidence.
- Next executable task: run and score the controlled 30-conversation Qwen
  behavior harness, including ten three-run consistency cases.
- Blockers: none.
- Open risks: M8 behavior-matrix runtime, clean-environment reproduction, and
  strict signing workflow.
- Last passing commit: not yet committed.
- Last commands executed: native tests, formatting, strict clippy, optimized
  WASI build, and six local-agent behavior cases.
- Test status: 58 native tests pass, including arbitrary-wire property tests;
  strict clippy passes on Rust 1.96.1.
- Build status: plugin release component passes for `wasm32-wasip2`
  (775,113 bytes after M7 schema and prompt integration).
- Evidence produced: competition requirements, ADRs, local Ollama lock, and
  successful native/pinned-host Qwen probes, live host execution, and
  sanitized allow/review/block/error/custody agent transcripts.
