# Local LLM Lock

- Captured: 2026-07-27
- Runtime: Ollama 0.32.0
- Endpoint: `http://127.0.0.1:11434`
- Model tag: `qwen3.5:9b`
- Local model ID/digest prefix reported by `ollama list`: `6488c96fa5fa`
- Architecture: qwen35
- Parameters: 9.7B
- Quantization: Q4_K_M
- Context capacity reported by Ollama: 262,144 tokens
- Cloud fallback: prohibited and not configured in the reference design
- Sanitized pinned-host configuration:
  [`zeroclaw-m0.toml`](zeroclaw-m0.toml)

## Tool-calling probe

The model received one `health_check` function with a strict object schema and
the instruction to call it exactly once with `service = "guardian"`.

Observed result:

```json
{
  "name": "health_check",
  "arguments": {
    "service": "guardian"
  }
}
```

The cold probe took approximately 57 seconds:

- model load: 16.7 seconds;
- prompt evaluation: 16.4 seconds;
- generation: 23.7 seconds.

This validates native Ollama tool-call emission.

## Pinned-host validation

ZeroClaw v0.8.3 was built at commit
`24476b71d33eb1672a9495a7ce3d155377a60ce8` with
`plugins-wasm,plugins-wasm-cranelift`. Its agent:

1. returned `GUARDIAN_LOCAL_OK` without a tool;
2. selected `solana_transaction_guardian` with schema-valid arguments;
3. received `m0_probe=ok`, `cluster=devnet`, and `rpc_result=ok` from the WASM;
4. therefore proved host `__config` injection and a live `waki` request to the
   configured Solana devnet endpoint.

The unavailable-runtime configuration points the sole Ollama alias at local
port 1. ZeroClaw exited with code 1 and reported all three local attempts as
timeouts. There was no provider alias, model fallback, or cloud fallback.
