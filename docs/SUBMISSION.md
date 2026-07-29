# Solana Transaction Guardian — submission one-pager

## Problem and audience

Solana users, treasury operators, developers, and agents routinely receive
serialized transactions whose visible request may omit authority changes,
delegate approvals, unknown programs, unexpected recipients, or excessive
outflow. Natural-language explanation alone is not an adequate financial
control.

The Solana Transaction Guardian analyzes a candidate before signing or audits
a confirmed transaction. It returns deterministic actions, effects, findings,
coverage, confidence, and one canonical `allow`, `review`, or `block`
decision.

## What is different

The Guardian is a transaction firewall for understanding, not a generic RPC
wrapper. It combines Solana wire decoding, v0 Address Lookup Table resolution,
simulation/confirmed effects, structured intent comparison, 54 stable risk
rules, and explicit fail-closed coverage in one bounded WASM component. The
local language model presents the result but cannot change it.

## Solana and ZeroClaw

The plugin supports legacy and version-0 Solana transactions, Address Lookup
Tables, System, Compute Budget, SPL Token, Token-2022, account state, inner
instructions, fees, return data, SOL/token deltas, simulation, and confirmed
metadata.

It is a real ZeroClaw v0.8.3 tool component targeting `wasm32-wasip2`. It
implements exactly one tool and declares only `config_read` and `http_client`.
The host injects operator configuration and mediates HTTPS to configured
Solana JSON-RPC endpoints.

## Local model and custody

The reference agent uses Ollama 0.32.0 on `127.0.0.1` with
`qwen3.5:9b`, digest:

```text
6488c96fa5faab64bb65cbd30d4289e20e6130ef535a93ef9a49f42eda893ea7
```

No OpenAI, Anthropic, OpenRouter, Gemini, Groq, or other cloud provider or
fallback is configured. Ollama/model unavailability is an explicit error.

Custody is **T0/read-only**. The component has no private-key, signing,
transaction-modification, submission, filesystem, socket, or key-store path.
The report is advisory and cannot stop a user signing elsewhere.

## Safety and evidence

- 60 native tests plus actual ZeroClaw host tests;
- 30/30 controlled Qwen conversations with 100% canonical-decision
  preservation and no omitted critical/high finding;
- strict Ed25519 manifest verification;
- Gitleaks and Semgrep with zero findings;
- OSV/RustSec with no known vulnerability and one explicit unmaintained
  `bincode 1.3.3` residual advisory;
- byte-reproducible canonical WASI builds;
- four devnet/offline demo fixtures, including hidden delegate, unknown
  program, and v0/ALT;
- prompt-injection custody evidence in
  [`evidence/agent-e2e/prompt-injection.md`](evidence/agent-e2e/prompt-injection.md).

## Reproduce

1. Follow [`INSTALLATION.md`](INSTALLATION.md).
2. Use the strict localhost-only profile in
   [`../config/zeroclaw.guardian.example.toml`](../config/zeroclaw.guardian.example.toml).
3. Verify the publisher key and release hashes.
4. Run the requests in [`EXAMPLES.md`](EXAMPLES.md) or the four
   [`demo fixtures`](../demo/README.md).
5. Compare the deterministic JSON report with the fixture expectation.

## Honest limitations

The MVP does not decode every DeFi protocol, supports only legacy/v0
transactions, depends on point-in-time RPC state, and cannot guarantee future
execution. Unknown programs and unavailable critical evidence reduce coverage
and fail closed under the reference policy. Qwen presentation remains
probabilistic; the WASM JSON report is authoritative. See
[`LIMITATIONS.md`](LIMITATIONS.md).

## Links

- Repository: <https://github.com/IagoPrandi/zeroclaw-plugin>
- Installation: [`INSTALLATION.md`](INSTALLATION.md)
- Architecture: [`ARCHITECTURE.md`](ARCHITECTURE.md)
- Threat model: [`THREAT_MODEL.md`](THREAT_MODEL.md)
- Test evidence: [`TEST_MATRIX.md`](TEST_MATRIX.md)
- Demo script: [`../demo/DEMO_SCRIPT.md`](../demo/DEMO_SCRIPT.md)

The public release and demo-video URLs are inserted after publication.
