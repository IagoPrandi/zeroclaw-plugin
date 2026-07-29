# Release 0.1.0

## Canonical artifacts

Source tag: `v0.1.0`

RC2 component source: `461d4d5ad7cb65bd919bdf5875e28812a3f5f8dc`

The release component is built in the pinned `rust:1.96.1-bookworm` image
identified by:

```text
sha256:a339861ae23e9abb272cea45dfafde21760d2ce6577a70f8a926153677902663
```

Two independent empty target directories produced the same component:

```text
file    solana_transaction_guardian.wasm
size    775829 bytes
sha256  780d7a88aa09eadcb345a7bfa6fd58e80cd93de27baa425d71439d6987e5c7e5
```

The release archive is generated twice with
[`scripts/package_release.py`](../scripts/package_release.py). The two ZIP
hashes match:

```text
file    solana-transaction-guardian-0.1.0.zip
size    253346 bytes
sha256  70a3ac35eb34850cddb5dd745be216278d0d0278924697dcb4e3e6d49cea1b3b
```

The same values are recorded in the package `SHA256SUMS` and must be copied to
the GitHub release notes without modification.

The separate public-demo asset is deterministic:

```text
file      guardian-demo.mp4
duration  158 seconds
size      1333678 bytes
sha256    1e3652197c3c3ae80c0911d626d1f102ff70b2e806fa6e17d3c17ccb3cd05828
```

## Signature

The manifest is signed with the Ed25519 publisher key documented in
[`PUBLISHER_KEY.md`](PUBLISHER_KEY.md). The key is trusted explicitly in the
reference ZeroClaw profile; strict mode rejects the same plugin when the trust
list is empty.

## Validation

- 60 native tests, formatting, and strict Clippy pass on Rust 1.96.1.
- Two canonical WASI release builds are byte-identical.
- The archive is structurally valid and deterministic.
- A clean ZeroClaw v0.8.3 profile installs, lists, and describes the plugin.
- The installed package passes strict-signature discovery.
- All four demo fixtures pass through the actual pinned host.
- The sanitized terminal demo is H.264, 1600×900, 2:38, visually reviewed,
  and reproduced byte-identically in two renders.

Public release:
<https://github.com/IagoPrandi/zeroclaw-plugin/releases/tag/v0.1.0>
