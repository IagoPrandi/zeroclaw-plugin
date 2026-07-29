# ZeroClaw confirmed-signature/devnet E2E evidence

- Date: 2026-07-28
- ZeroClaw: v0.8.3, commit
  `24476b71d33eb1672a9495a7ce3d155377a60ce8`
- Host features: `plugins-wasm,plugins-wasm-cranelift`
- Cluster: Solana devnet
- Public signature:
  `49TXiM9rVcpspUZtT5LynRePwWyazTERXAhYHAsTSAz7fi4TtYspwiUM1zuyGmoRoddJKM9M5K47FPfPcimhufu4`
- Slot: `479614616`

The real ZeroClaw component host loaded the release WASM, injected the
configured devnet endpoint, called `getTransaction` through WASI HTTP, and
returned a successful Guardian tool result.

Validated fields:

```json
{
  "source": {
    "type": "confirmed",
    "slot": 479614616,
    "transaction_version": "legacy"
  },
  "decision": "block",
  "risk_level": "high",
  "analysis_complete": false,
  "execution": {
    "status": "confirmed_succeeded",
    "units_consumed": 14589
  },
  "fees": {
    "base_fee_lamports": "5000",
    "priority_fee_lamports": "750",
    "total_estimated_fee_lamports": "5750"
  }
}
```

The report preserved the unknown top-level program as `COV-003`, exposed it in
coverage and limitations, decoded two inner System Program actions in execution
order, and reconciled three effective SOL deltas. Execution success did not
override the fail-closed `block` decision.

Result: pinned-host test passed. No key, signer, submission path, filesystem, or
auxiliary service was used.
