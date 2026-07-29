# M8 automated security scans

Date: 2026-07-29

## Secrets

Gitleaks 8.30.1 scanned a clean snapshot of the product source and
documentation. The snapshot excluded repository metadata, the external
`.claude` skill catalog, and generated `target` artifacts.

```text
scanned 544801 bytes
no leaks found
```

An initial whole-workspace pass found only placeholder credentials in the
external `.claude` reference catalog. No finding was in a product file.

## Static analysis

Semgrep 1.164.0 ran the community Rust and Python configurations against
`src/` and `scripts/`.

```text
162 rules
20 files
100% parsed lines
0 findings
```

## Dependency vulnerabilities

OSV-Scanner 2.3.8 scanned 220 packages from `Cargo.lock`.

```text
critical  0
high      0
medium    0
low       0
unknown   1
```

The single unknown-severity result is `RUSTSEC-2025-0141` for unmaintained
`bincode 1.3.3`. It has no fixed release and is independently documented in
the security review and limitations. `cargo audit` reports zero
vulnerabilities and the same maintenance warning.

## Final M9 rescan

The post-demo source state was rescanned with the same pinned tools. Gitleaks
initially reported 31 candidates: 29 documented placeholders in the vendored
`.claude/skills` catalog, the public Ed25519 manifest signature, and the public
devnet Address Lookup Table address. Manual review found no secret.

`.gitleaks.toml` keeps the default rules enabled and narrowly excludes the
vendored skill path plus the exact public ALT identifier. `.gitleaksignore`
excludes only the fingerprint of the reviewed public manifest signature. The
filtered history scan reports zero leaks.

OSV-Scanner 2.3.8 rescanned all 220 locked packages with the same result:
zero critical/high/medium/low vulnerabilities, one unknown-severity
`RUSTSEC-2025-0141` maintenance advisory, and no available fix. `cargo audit`
reported zero vulnerabilities and one allowed unmaintained warning.

Semgrep 1.164.0 could not repeat its registry-backed ruleset download during
the final M9 pass: the specific-config attempt timed out after 184 seconds and
the `auto` attempt was stopped after the registry remained unresponsive. This
is not reported as a passing rescan. The last completed Semgrep result remains
the 162-rule, zero-finding M8 scan. Post-scan Rust changes are covered by the
60-test/Clippy pipeline and manual review; the new Python and PowerShell demo
scripts passed Python compilation and PowerShell parser validation.
