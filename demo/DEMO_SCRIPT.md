# Three-minute demo script

Target duration: 2 minutes 55 seconds.

## 0:00–0:20 — Problem

“A Solana transaction presented to an agent can contain more than the action
the user described. The Guardian decodes and simulates the transaction before
trust, exposes hidden authority changes and unknown programs, and returns a
deterministic decision.”

## 0:20–0:45 — Local and read-only

Show `ollama list` with `qwen3.5:9b`, the localhost-only provider section, and
`zeroclaw plugin list`.

“The agent runs locally through Ollama with no cloud fallback. The WASM plugin
has only configuration-read and HTTP-client permissions. It cannot access a
seed, sign, or broadcast.”

## 0:45–1:15 — Safe transfer

Submit `01-safe-transfer.json`.

“This candidate contains one one-lamport System transfer. The plugin shows the
signer, recipient, simulation status, full instruction coverage, and the
canonical `allow`. Allow means only that no configured blocking rule was found
with this reported coverage.”

## 1:15–1:50 — Hidden delegate

Submit `02-hidden-delegate.json`.

“The request looks like the same payment, but the second instruction approves
a token delegate. Structured intent allowed only the System Program and the
documented recipient. The deterministic engine reports the approval and
intent mismatch and returns `block`; Qwen preserves that decision.”

## 1:50–2:20 — Unknown program

Submit `03-unknown-program.json`.

“This instruction calls an undeclared program without a decoder. The Guardian
does not silently label it safe: it exposes unknown-program coverage and the
intent mismatch, then blocks under the reference fail-closed policy.”

## 2:20–2:40 — v0 and ALT

Submit `04-v0-alt.json`.

“This candidate is version 0 and loads its recipient from a devnet lookup
table. The report preserves the v0 source, resolves the address set, and
simulates the transfer without hiding the ALT dependency.”

## 2:40–2:55 — Evidence and limit

Show the GitHub release, `SHA256SUMS`, test matrix, and limitations.

“The release has 60 tests, reproducible WASM, a signed manifest, formal
security scans, and 30 out of 30 local-Qwen behavior checks. The Guardian does
not ask users to trust AI prose: the deterministic JSON report is the
authority.”
