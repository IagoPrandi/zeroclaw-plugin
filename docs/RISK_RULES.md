# Deterministic risk rules

The reducer keeps execution status, risk severity, decision, and confidence
separate. Any block finding wins. If minimum analysis is incomplete and
`fail_closed=true`, the result is block even with a low numeric score. Intent
can only add restrictions; it cannot raise an operator hard cap.

Implemented rule families:

- `COV-*`: unsupported/malformed versions and ALTs fail before reporting;
  unknown observed programs, unavailable simulation, and missing inner
  instructions produce findings.
- `EXEC-*`: simulation/confirmed failure, duplicate compute budget, and
  near-limit consumption.
- `XFER-*`: SOL/token outflow, blocked/unknown recipients, and structured
  minimum/maximum outcomes.
- `AUTH-*` / `ACCT-*`: owner/mint/freeze/nonce/program authority, delegates,
  permanent delegates, close, freeze, burn, and mint.
- `T22-*`: transfer hook/fee, confidential coverage, default state,
  non-transferable assets, and CPI guard.
- `PROG-*`: blocklist, strict allowlist, upgrade/deploy/authority/close, and
  high-impact unknown programs.
- `FEE-*`: priority-fee review/block thresholds, arithmetic overflow, and
  excessive requested compute.
- `INT-*`: extra program/recipient/authority, asset/value mismatch, and minimum
  result.

Critical missing account state, incompatible owners, inconsistent RPC, and
unresolved lookup tables are typed fail-closed analysis errors instead of
partial findings. This prevents a malformed prerequisite from yielding a
normal-looking report.

Score weights are critical 40, high 25, medium 10, and low 3, capped at 100.
Confidence starts at 1.0 and subtracts 0.15 per unknown program (maximum 0.45),
0.15 for missing expected inner instructions, and 0.35 for unavailable
candidate simulation.
