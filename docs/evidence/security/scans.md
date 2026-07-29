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
