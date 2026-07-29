# Gate G9 package evidence

Date: 2026-07-29

## Canonical component

Two independent builds in the pinned `rust:1.96.1-bookworm` image, each with
an empty target directory, produced:

```text
file    solana_transaction_guardian.wasm
size    775829 bytes
sha256  780d7a88aa09eadcb345a7bfa6fd58e80cd93de27baa425d71439d6987e5c7e5
```

The first build ran the complete `scripts/check.sh` pipeline: 60 tests,
formatting, strict Clippy, and a locked optimized WASI build.

## Deterministic archive

Two empty output directories produced byte-identical packages:

```text
file    solana-transaction-guardian-0.1.0.zip
size    253346 bytes
sha256  70a3ac35eb34850cddb5dd745be216278d0d0278924697dcb4e3e6d49cea1b3b
```

Archive integrity testing found no corrupt entry. The embedded WASM hash
matched the standalone canonical component.

## Clean ZeroClaw validation

The archive was extracted into an empty temporary directory and installed into
an empty ZeroClaw v0.8.3 profile. The pinned host:

- installed the plugin;
- listed `solana-transaction-guardian v0.1.0`;
- reported only `ConfigRead` and `HttpClient`;
- loaded the signed manifest in strict mode with the trusted publisher;
- rejected discovery with an empty trust list.

The installed component then passed all four demo fixtures through the actual
host:

| Fixture | Decision | Relevant result |
|---|---|---|
| safe transfer | `allow` | legacy System transfer |
| hidden delegate | `block` | transfer + approve; authority/intent findings |
| unknown program | `block` | unknown-program and intent findings |
| v0/ALT | `allow` | version `v0`; lookup table resolved |

No private key, seed, signing path, or cloud LLM provider was involved.
