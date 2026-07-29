# Security

Solana Transaction Guardian is read-only. It must never receive a private key,
seed phrase, signing callback, or transaction-submission capability.

## Reporting

Use GitHub's private vulnerability-reporting feature for the repository when
available. Otherwise contact the maintainers through a private channel listed
on their GitHub profiles before publishing details. Do not open a public issue
for an exploitable policy bypass, sandbox escape, or secret exposure.

Do not include secrets, complete private RPC URLs, or sensitive transaction
payloads in a report. Provide a minimal redacted reproducer, affected version,
expected/actual decision, and impact.

Security fixes are prioritized by custody impact, policy bypass, incorrect
`allow` decisions, sandbox escape, and secret exposure. There is no bug-bounty
promise unless a current GitHub security policy explicitly states one.

## Supported versions

Supported versions are listed in GitHub releases. During the release-candidate
phase, only the latest published RC and latest default-branch revision receive
security fixes.

## Operator responsibilities

- verify the release SHA-256 and manifest publisher key;
- use ZeroClaw strict signature mode outside local development;
- configure RPC endpoints only through the host profile;
- keep `fail_closed=true` for value-bearing workflows;
- keep Ollama bound to localhost and do not configure cloud LLM fallback;
- treat the deterministic tool JSON, not the LLM prose, as authoritative;
- never provide the plugin or agent with a private key or seed phrase.

The threat model, residual risks, dependency advisory, and completed review
are documented in `docs/THREAT_MODEL.md`, `docs/LIMITATIONS.md`, and
`docs/SECURITY_REVIEW.md`.
