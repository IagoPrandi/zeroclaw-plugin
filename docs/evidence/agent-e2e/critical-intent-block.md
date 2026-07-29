# Critical intent mismatch transcript

- Prompt version: 1.0.2
- Input: the sanitized one-lamport Base64 fixture, observed source wallet,
  allowed-recipient list containing only that source wallet, and maximum
  expected SOL outflow of zero lamports
- Tool arguments: included the nested source object, `observed_wallets`,
  `expected_intent.allowed_recipients`, and
  `expected_intent.max_sol_out_lamports`
- Canonical result: `block`, risk level `critical`, score 80
- Duration: 323.8 seconds on the reference CPU-only host

Captured response:

> **Decision:** block
>
> **Critical Findings (Blocking):**
> 1. **INT-002** — observed recipient was not declared in the expected intent.
> 2. **INT-005** — observed SOL outflow exceeds the declared intent.

The response also named the actual recipient and the one-lamport outflow. Its
last transaction-details line ended early; the canonical report contained two
limitations (simulation is point-in-time and base-fee estimation returned
`RPC_PROTOCOL`). Limitation preservation is separately demonstrated by the
allow and review transcripts, so this truncation is recorded rather than
hidden.
