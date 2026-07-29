# Architecture Decision Records

## ADR-001 — ZeroClaw v0.8.3 protocol pin

- Status: accepted
- Date: 2026-07-27
- ZeroClaw tag: `v0.8.3`
- ZeroClaw commit: `24476b71d33eb1672a9495a7ce3d155377a60ce8`
- WIT package: `zeroclaw:plugin@0.1.0`
- World: `tool-plugin`

The component vendors the exact `wit/v0` tool contract from the pinned host.
The host must be built with both `plugins-wasm` and a backend such as
`plugins-wasm-cranelift`.

The shared Rust toolchain is pinned to 1.96.1 because the ZeroClaw v0.8.3
package declares that minimum version. The first 1.91.1 probe compiled the
plugin but correctly failed the host's MSRV check.

## ADR-002 — Modular Solana crates

- Status: accepted at Gate G0
- Date: 2026-07-27

Use official modular crates rather than `solana-sdk` or an RPC client:

- `solana-transaction` 4.1.5 with Serde wire decoding;
- `solana-message` 4.4.0;
- `spl-token-interface` 3.0.0;
- `spl-token-2022-interface` 3.1.1.

RPC transport is local JSON-RPC over `waki` 0.5.1 on WASI. Internal addresses
uses an independent 32-byte type at the application boundary.

## ADR-003 — Local inference only

- Status: accepted
- Date: 2026-07-27

The supported runtime is Ollama on localhost with `qwen3.5:9b`. No cloud LLM
provider or fallback is allowed in the reference configuration.

## ADR-004 — Local tool grammar and strict source validation

- Status: accepted
- Date: 2026-07-28

The public tool schema publishes `source` as a normal nested object because
Ollama/Qwen converted a nested `oneOf` schema into a string-valued tool
argument. Runtime deserialization remains a strict internally tagged enum, so
`type=serialized` still requires `transaction_base64`, `type=confirmed` still
requires `signature`, and unknown fields are rejected. This preserves the
security contract without adding a permissive parsing fallback.

## ADR-005 — Disable Qwen reasoning for deterministic presentation

- Status: accepted
- Date: 2026-07-29

The reference ZeroClaw alias sets `think = false` for `qwen3.5:9b`. During M7,
one post-tool turn consumed reasoning tokens but returned no visible answer.
Disabling model reasoning is supported by the pinned ZeroClaw Ollama provider,
reduces latency, and keeps the model's role limited to tool selection,
argument assembly, and faithful presentation. Prompt version 1.0.2 also
requires a visible response and explicit mapping of observed wallets and
declared intent constraints.
