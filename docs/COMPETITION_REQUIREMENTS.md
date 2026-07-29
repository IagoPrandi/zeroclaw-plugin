# Competition Requirements

Source reviewed on 2026-07-27 and rechecked on 2026-07-29:
<https://superteam.fun/earn/listing/zeroclaw>

## Official requirements

- Deadline: 2026-08-07 02:59:59 UTC (2026-08-06 23:59:59 BRT).
- Winner announcement target: 2026-08-21 02:59:59 UTC.
- Total rewards: 5,000 USDG.
- Submission format: a showcase post in the ZeroClaw Discord
  `#solana-bounty` channel.
- Required demo video: three minutes or less, showing a real agent on a real
  channel doing a real Solana-related job. Slides alone are not accepted.
- Required write-up: purpose, audience, ZeroClaw features, custom code,
  custody tier, threat model, reproducible config/SOPs/skills/code, and
  redacted secrets.
- A GitHub repository link is required for custom plugin code.
- The Superteam form requires a demo video link and a supporting-material
  link. A one-pager link is optional.
- A standalone plugin is not a valid submission; the plugin must support a
  working use case.
- Do not open a ZeroClaw registry PR during the bounty.
- Reproducibility is explicitly scored.
- A prompt-injection test transcript is required when the use case touches
  funds.
- The listing does not impose an additional source-license or IP-assignment
  term. This repository uses the MIT license; the submission must still respect
  dependency licenses and third-party attribution.

## Judging

- Use case: 30%.
- Safety and custody design: 25%.
- Craft: 20%.
- Reproducibility: 15%.
- Showcase: 10%.
- Tiebreak: public build logs on X during the bounty.

## Product alignment

The listing explicitly names deterministic Token-2022 TLV risk checks as a
valid Tier 3 WASM plugin use case. The Guardian is T0/read-only and uses a
pure Rust core with a thin WASM adapter. It must be demonstrated as a real
transaction-review workflow, not presented as a component in isolation.

## Submission form fields observed

- Demo video link (required).
- One-pager link (optional).
- Supporting material (required).

The final review found the listing open with 70 submissions. The substantive
requirements, deadline, scoring, disqualifiers, and custody guidance were
unchanged from the initial review.

## Evidence

The source page's JSON-LD reports:

- `datePosted`: `2026-07-16T21:42:01.525Z`
- `validThrough`: `2026-08-07T02:59:59.000Z`
- `baseSalary`: `5000 USDG TOTAL`
