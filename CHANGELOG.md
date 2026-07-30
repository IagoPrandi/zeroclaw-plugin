# Changelog

All notable changes follow semantic versioning.

## Unreleased

- Made the plugin independent of the ZeroClaw model/provider selected by the
  user; Ollama/Qwen remains reference evidence only.
- Added explicit, fail-closed devnet defaults so a plugin entry is not needed
  for first use.
- Added a PowerShell package installer that preserves an existing ZeroClaw
  profile and model choice, plus installer coverage.
- Reworked installation and configuration guidance around zero-config devnet
  use and opt-in mainnet/RPC/policy customization.

## 0.1.0 - 2026-07-29

- Promoted the corrected RC2 component to the first stable release.
- Added the signed deterministic ZIP/WASM distribution and published
  checksums.
- Added the validated 2:38 terminal demo, clean-install evidence,
  prompt-injection transcript, submission one-pager, and final security review.
- Added a 2:46 public phone-and-terminal walkthrough based on sanitized,
  recorded agent exchanges and matching deterministic terminal evidence.

## 0.1.0-rc.2 - 2026-07-29

- Fixed RPC-backed version-0 analysis so resolved Address Lookup Table keys are
  reused during normalization instead of being discarded and falsely reported
  as unresolved.
- Added a native regression test, raising the passing suite to 60 tests.
- Added signed deterministic packaging, clean-install validation, four public
  demo fixtures, a three-minute script, and complete release/submission docs.
- Supersedes RC1 for release and demonstration.

## 0.1.0-rc.1 - 2026-07-29

- Added a real ZeroClaw v0.8.3 `wasm32-wasip2` tool component.
- Added strict serialized/confirmed inputs, legacy/v0 parsing, ALT resolution,
  state acquisition, candidate simulation, and confirmed-effect analysis.
- Added System, Compute Budget, SPL Token, Token-2022, inner-instruction,
  fee, log, return-data, SOL-delta, and token-delta interpretation.
- Added 54 deterministic policy/intent rules with stable decisions, coverage,
  confidence, and fail-closed prerequisite findings.
- Added local Ollama 0.32.0 / `qwen3.5:9b` integration and a 30-conversation
  behavior matrix with 100% decision preservation.
- Added reproducible canonical builds, strict-signature/resource host tests,
  formal security scans, performance evidence, and release documentation.
