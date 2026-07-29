# Gate G8 release candidate approval

Date: 2026-07-29

## Version

- Tag: `v0.1.0-rc.1`
- Commit: `96df5b12e74c5d934a990f4bd3a32224320b9b56`
- Cargo/manifest version: `0.1.0`

## Gate evidence

- Zero open critical defects and no known flaky tests.
- 59/59 native tests passed.
- Formatting and strict Clippy passed on Rust 1.96.1.
- Locked optimized `wasm32-wasip2` build passed.
- Two canonical clean builds were byte-identical.
- Gitleaks and Semgrep reported zero product findings.
- OSV and RustSec reported zero known vulnerabilities; the unmaintained
  `bincode 1.3.3` advisory remains an explicit residual limitation.
- Strict ZeroClaw signature verification accepted the trusted signature and
  rejected an empty trust list.
- The 30-conversation `qwen3.5:9b` matrix passed 30/30 with 100% canonical
  decision preservation and 100% correct tool-call behavior.
- Ollama 0.32.0 and model digest
  `6488c96fa5faab64bb65cbd30d4289e20e6130ef535a93ef9a49f42eda893ea7`
  were verified in an isolated localhost-only container.
- Twenty live devnet analyses passed under a six-RPC budget with p95 1,653 ms.
- Limitations are explicit in `docs/LIMITATIONS.md`.

Result: Gate G8 passed; M9 may begin.
