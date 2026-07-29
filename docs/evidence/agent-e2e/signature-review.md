# Confirmed signature review transcript

- Prompt version: 1.0.0 with the stable `think=false` provider setting
- Policy variation: `fail_closed=false`; unknown-program effect remained
  `review`
- Input: the same public confirmed devnet signature as the block case
- Canonical decision: `review`
- Duration: 268.7 seconds on the reference CPU-only host

Captured response:

> **Decision:** `review`
>
> **Evidence & Findings:**
> - **Risk Level:** High (Score: 25)
> - **Analysis Complete:** No (`false`)
> - The transaction contains unknown program
>   `844HVKb4Fq7btdDwefRAEJZrFiBWwbiKfHRcKSk88WYE`.
>
> **Coverage Limitations:**
> - No decoder is available for the unknown program.
> - Only 2 out of 3 top-level instructions were decoded.
>
> **Summary:** Human review is required before trust or signing.

The default reference policy was restored to `fail_closed=true` after this
controlled policy-branch test.
