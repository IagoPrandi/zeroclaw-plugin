# Security

Solana Transaction Guardian is read-only. It must never receive a private key,
seed phrase, signing callback, or transaction-submission capability.

Report vulnerabilities privately to the repository maintainers. Do not include
secrets, complete private RPC URLs, or sensitive transaction payloads in a
report. Security fixes are prioritized by custody impact, policy bypass,
incorrect `allow` decisions, sandbox escape, and secret exposure.

Supported versions are listed in GitHub releases. Until the first release, only
the latest default-branch revision is supported.
