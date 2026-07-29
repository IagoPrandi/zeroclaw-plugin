# Clean container validation

Date: 2026-07-29

## Build and tests

The repository was mounted read-only into a clean
`rust:1.96.1-bookworm` image. Cargo used an empty target directory and ran
`scripts/check.sh` successfully: 59 tests, formatting, strict Clippy, and
release `wasm32-wasip2`.

Two builds pinned by image digest produced the same 775,945-byte WASM and
SHA-256:

```text
c375d0319693e110afa4f1cef579b1b763e68ce371f5b64f19d63c65c099ba00
```

## Ollama and Qwen

An isolated `ollama/ollama:0.32.0` container was bound only to
`127.0.0.1:11435`. The existing model cache was mounted read-only.

- API version: 0.32.0
- Model: `qwen3.5:9b`
- Digest:
  `6488c96fa5faab64bb65cbd30d4289e20e6130ef535a93ef9a49f42eda893ea7`
- Format/quantization: GGUF, Q4_K_M
- Capability list included tools and thinking
- Controlled generation returned exactly `OK`
- Model execution time: 55.8 seconds on CPU

The container was stopped after validation.

## Strict signature

The actual Guardian manifest was signed with an ephemeral Ed25519 test key
using the pinned host's official signature functions. ZeroClaw strict mode
loaded it with the trusted public key and rejected the same plugin with an
empty trust list. No private key was written to the repository.

## Cloud/credential check

Repository scanning found no private-key block, provider API-key assignment,
or `sk-*` credential. The reference config contains only
`http://127.0.0.1:11434`; its ZeroClaw trace contained 41 local Ollama
references and zero OpenAI, Anthropic, OpenRouter, Gemini, or Groq endpoints.
