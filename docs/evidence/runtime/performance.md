# Devnet performance and RPC budget

Date: 2026-07-29

## Environment

- ZeroClaw: v0.8.3, commit
  `24476b71d33eb1672a9495a7ce3d155377a60ce8`
- Guardian: `v0.1.0-rc.1`
- Host runtime: Wasmtime/Cranelift through `zeroclaw-plugins`
- RPC: `https://api.devnet.solana.com`
- Input: the versioned candidate Base64 fixture
- Samples: 20 sequential calls on one instantiated component
- Plugin RPC budget: `max_rpc_calls=6`

Component instantiation, Rust compilation, and LLM inference were outside the
measurement. All measured calls reached a successful deterministic Guardian
report, so RPC unavailability did not affect the sample.

## Result

```text
samples_ms = [
  1592, 1595, 1597, 1600, 1600,
  1601, 1603, 1609, 1616, 1617,
  1619, 1626, 1628, 1628, 1629,
  1634, 1636, 1642, 1653, 1669
]
p95_ms = 1653
target_ms = 8000
max_rpc_calls = 6
```

The same per-analysis budget counter that guards production RPC operations
would return a controlled error on a seventh call. Twenty successful reports
therefore demonstrate that the common candidate path stays within six RPC
calls. The measured p95 is 79.3% below the eight-second target.
