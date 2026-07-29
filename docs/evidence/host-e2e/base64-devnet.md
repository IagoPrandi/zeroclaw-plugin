# ZeroClaw Base64/devnet E2E evidence

- Date: 2026-07-28
- ZeroClaw: v0.8.3, commit
  `24476b71d33eb1672a9495a7ce3d155377a60ce8`
- Host features: `plugins-wasm,plugins-wasm-cranelift`
- Plugin: locally built `solana-transaction-guardian` 0.1.0
- Cluster: Solana devnet
- Input: sanitized legacy System transfer of 1 lamport
- Transaction Base64: intentionally omitted

The component was loaded through the real ZeroClaw plugin runtime. The host
injected the plugin configuration and the component called live devnet through
WASI HTTP. The test executed the tool export and asserted the structured
report.

Observed report fields:

```json
{
  "decision": "allow",
  "risk_level": "low",
  "risk_score": 0,
  "confidence": 1.0,
  "analysis_complete": true,
  "execution": {
    "status": "simulation_succeeded",
    "units_consumed": 150
  },
  "actions": [
    {
      "instruction_index": 0,
      "kind": "transfer",
      "details": {
        "lamports": "1"
      }
    }
  ],
  "coverage": {
    "top_level_instructions": {
      "decoded": 1,
      "total": 1
    },
    "inner_instructions_available": true,
    "address_lookup_tables_resolved": true,
    "simulation_available": true
  }
}
```

The report also contained a valid RFC 3339 `generated_at` value. Base-fee
estimation was unavailable because the unsigned fixture used a replaceable
blockhash; the final implementation reports that condition as an explicit
typed limitation rather than silently discarding it.

Validation command:

```powershell
$env:GUARDIAN_WASM = "<repo>\target\wasm32-wasip2\release\solana_transaction_guardian.wasm"
cargo +1.96.1-x86_64-pc-windows-gnu test `
  -p zeroclaw-plugins `
  --features plugins-wasm-cranelift `
  --test guardian_live -- --nocapture
```

Result: passed. No signer, private key, transaction submission, filesystem, or
auxiliary service was used.
