# Clean container validation

Date: 2026-07-29

## Build and tests

The repository was mounted read-only into a clean
`rust:1.96.1-bookworm` image. Cargo used an empty target directory and ran
`scripts/check.sh` successfully: 60 tests, formatting, strict Clippy, and
release `wasm32-wasip2`.

Two builds pinned by image digest produced the same 775,829-byte WASM and
SHA-256:

```text
780d7a88aa09eadcb345a7bfa6fd58e80cd93de27baa425d71439d6987e5c7e5
```

This second clean validation supersedes the RC1 component after the v0/ALT
lookup-map regression was corrected. Both builds used independent empty
target directories.

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

The release manifest was signed with the project publisher's Ed25519 key using
the pinned host's official signature functions. ZeroClaw strict mode loaded it
with the trusted public key and rejected the same plugin with an empty trust
list. The private key remains outside the repository with user-only filesystem
access.

## Cloud/credential check

Repository scanning found no private-key block, provider API-key assignment,
or `sk-*` credential. The reference config contains only
`http://127.0.0.1:11434`; its ZeroClaw trace contained 41 local Ollama
references and zero OpenAI, Anthropic, OpenRouter, Gemini, or Groq endpoints.
