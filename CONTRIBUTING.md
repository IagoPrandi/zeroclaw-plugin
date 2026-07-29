# Contributing

Read `PRD.md`, `AGENTS.md`, and the relevant `.claude/skills` instructions
before changing the project. Keep the deterministic core independent from the
WASM adapter, preserve fail-closed behavior, add positive and negative tests,
and run `scripts/check.sh`.

Never add signers, keypair handling, transaction submission, cloud-model
fallbacks, or operator endpoints supplied through tool arguments.
