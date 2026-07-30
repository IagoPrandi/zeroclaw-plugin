# Reference LLM runtime

This document records the reproducible environment used for the published
demonstration and behavior evidence. It is not required to install or use
Guardian. Guardian does not configure an LLM; it uses the model/provider
already selected in the user's ZeroClaw profile.

## Pinned runtime

- Ollama: 0.32.0
- Model tag: `qwen3.5:9b`
- Digest:
  `6488c96fa5faab64bb65cbd30d4289e20e6130ef535a93ef9a49f42eda893ea7`
- Format/quantization: GGUF / Q4_K_M
- Endpoint: `http://127.0.0.1:11434`

The version, model metadata, digest, and reference hardware are preserved under
`docs/evidence/runtime/`.

## Installation and verification

```bash
ollama serve
ollama pull qwen3.5:9b
ollama list
ollama show qwen3.5:9b
curl --fail --silent http://127.0.0.1:11434/api/tags
```

Match the full digest before running official behavior or demo flows. The
validated Ollama container was localhost-only and used a read-only model cache.

## Exact ZeroClaw v0.8.3 provider syntax

```toml
[providers.models.ollama.local]
uri = "http://127.0.0.1:11434"
model = "qwen3.5:9b"
num_ctx = 8192
temperature = 0.0
think = false
timeout_secs = 600
max_tokens = 1000

[agents.guardian]
model_provider = "ollama.local"
risk_profile = "guardian"
runtime_profile = "guardian_local"
```

This syntax was copied from and executed against the pinned host. Do not use
provider syntax from another ZeroClaw version without repeating compatibility
and behavior tests.

## Expected behavior

The model:

- detects one supplied Base64 transaction or confirmed signature;
- calls `solana_transaction_guardian` at most once;
- asks for clarification when multiple transaction sources are ambiguous;
- does not call the tool when input is absent;
- preserves the literal canonical decision;
- includes critical/high blocking findings and coverage limitations;
- never requests a seed/private key or claims to sign/broadcast;
- never turns a tool error or unavailability into a positive recommendation.

The versioned system prompt is
[prompts/GUARDIAN_SYSTEM.md](../prompts/GUARDIAN_SYSTEM.md). The controlled
matrix passed 30/30 conversations; raw results are under
`docs/evidence/agent-e2e/`.

## Reference failure behavior

The reference profile intentionally has no cloud fallback. If Ollama is
unreachable, the model is absent, or the tool is unavailable, that reference
flow stops with an actionable diagnostic. Users may select another provider or
model for their own ZeroClaw profile; doing so does not alter the Guardian WASM
or its deterministic security decision. It does require separate tool-calling
validation before claiming equivalent presentation behavior.

Local inference is probabilistic and can be slow on CPU-only hardware. It is a
presentation layer, not the risk engine. Always treat the plugin JSON as
authoritative.
