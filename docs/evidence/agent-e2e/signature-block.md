# Confirmed signature block transcript

- Prompt version: 1.0.0
- Input: natural-language request containing the public confirmed devnet
  signature recorded in `../host-e2e/confirmed-devnet.md`
- Tool arguments: `source` object with `type=confirmed` and `signature`,
  `cluster=devnet`, `output_language=en`
- Canonical decision: `block`
- Duration: 273.7 seconds on the reference CPU-only host

Captured response:

> **Decision:** `block`
>
> **Evidence & Findings:**
> - **Risk Level:** High (Score: 25)
> - **Critical Finding:** The transaction interacts with unknown program
>   `844HVKb4Fq7btdDwefRAEJZrFiBWwbiKfHRcKSk88WYE`, flagged as `COV-003`
>   with high severity.
> - **Asset Movement:** SOL moves from the fee payer to two other accounts.
>
> **Coverage & Limitations:**
> - Analysis is incomplete because that program has no decoder.
>
> **Conclusion:** Do **not** execute.

The runtime trace recorded the nested `source` value as an object, not a JSON
string.
