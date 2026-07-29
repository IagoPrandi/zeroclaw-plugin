# Clean demo safe-transfer transcript

- Date: 2026-07-29
- Source: clean install of the signed release package
- Host: ZeroClaw v0.8.3
- Model: local Ollama 0.32.0 / `qwen3.5:9b`
- Session: new profile with no previous conversation
- Tool input: public unsigned one-lamport devnet fixture
- Agent duration: 271 seconds on the reference CPU-only host
- Sanitized transcript SHA-256:
  `030caba94e424ee6a3648410fac06b6c3108f7eb22a7000d9e9c9a241ccbb37e`

Captured response:

> **Decision:** allow
>
> **Evidence & Decoded Actions:** The transaction is a single legacy System
> Program transfer from the fee payer to the expected recipient.
>
> **Amount transferred:** 1 lamport
>
> All top-level instructions were decoded, the lookup-table requirement was
> resolved, and simulation succeeded with 150 compute units.
>
> **Limitation:** Base-fee estimation was unavailable due to an RPC protocol
> constraint.

The model preserved the canonical decision, integer amount, decoded action,
coverage, and limitation. It did not request a key or claim to sign or
broadcast.

An earlier clean take converted one lamport to SOL incorrectly and inferred a
purpose for the transfer. That take was rejected. The final demo request
therefore requires exact raw integer amounts and prohibits inferred purpose.
