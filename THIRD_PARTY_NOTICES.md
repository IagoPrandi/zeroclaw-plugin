# Third-party notices

Solana Transaction Guardian is MIT licensed. It depends on or interoperates
with the following independently licensed projects and model. They are not
included as modified source copies in this repository unless stated.

| Project | Use | License |
|---|---|---|
| ZeroClaw | pinned plugin host and WIT contract | MIT OR Apache-2.0 |
| Solana Rust crates | transaction/message wire types | Apache-2.0 |
| SPL Token interface crates | Token and Token-2022 decoding | Apache-2.0 |
| Waki | WASI HTTP client | Apache-2.0 |
| Ollama CLI/server | local model runtime | MIT |
| Qwen3.5-9B | local model weights used by the reference environment | Apache-2.0 |

The complete Rust dependency graph and exact versions are fixed by
`Cargo.lock`. Their license texts and notices remain governed by their
respective distributions.

Authoritative upstream references:

- ZeroClaw: https://github.com/zeroclaw-labs/zeroclaw
- Solana: https://github.com/anza-xyz/agave
- SPL programs: https://github.com/solana-program
- Waki: https://github.com/yomorun/waki
- Ollama: https://github.com/ollama/ollama
- Qwen3.5-9B: https://huggingface.co/Qwen/Qwen3.5-9B

Solana, ZeroClaw, Ollama, Qwen, and other names are trademarks or project
names of their respective owners. This project is independent and does not
claim endorsement.
