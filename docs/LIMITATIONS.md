# Limitations

- Analysis is read-only and advisory; it cannot prevent signing outside the
  ZeroClaw flow.
- Simulation and account state are point-in-time observations and do not
  guarantee later execution.
- Unknown programs and unsupported Token-2022 behavior can leave coverage
  incomplete; fail-closed policy should remain enabled for critical flows.
- The MVP does not decode protocol-specific DeFi semantics such as Jupiter,
  Raydium, or Kamino.
- RPC availability, retention, rate limits, and inconsistent responses can
  prevent a complete report.
- Version 1 Solana transactions are rejected; the MVP supports legacy and v0.
- Return data and logs are represented as bounded evidence, not trusted
  authorization.
- `qwen3.5:9b` presentation is probabilistic even at temperature zero. The
  deterministic WASM report remains authoritative.
- CPU-only local inference on the reference host takes roughly 36–324 seconds
  per controlled turn/E2E flow.
- The canonical Linux container build is bit-reproducible. Windows produces a
  functionally equivalent but byte-different WASM.
- `bincode 1.3.3` is unmaintained but currently required for the validated
  legacy/v0 Solana wire format; RustSec reports no vulnerability for it.
