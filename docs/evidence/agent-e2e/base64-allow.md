# Base64 allow transcript

- Prompt version: 1.0.0
- Input: natural-language request containing the sanitized one-lamport Base64
  fixture
- Tool arguments: `source.type=serialized`, Base64 omitted, `cluster=devnet`,
  `output_language=en`
- Canonical decision: `allow`
- Duration: approximately 256 seconds on the reference CPU-only host

Captured response:

> **Decision:** `allow`
>
> **Evidence:**
> - **Risk Level:** low (Score: 0)
> - **Simulation Status:** succeeded
> - **Actions Decoded:** A single SOL transfer of 1 lamport from signer to recipient.
> - **Programs Involved:** System Program only; no unknown programs detected.
>
> **Coverage & Limitations:**
> - Simulation reflects recent state and does not guarantee future execution.
> - Base fee estimation could not be obtained (`RPC_PROTOCOL`).

The runtime trace showed a successful WASM tool call and no remote model
provider.
