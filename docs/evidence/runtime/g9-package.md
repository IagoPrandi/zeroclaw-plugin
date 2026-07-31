# Gate G9 package evidence

Date: 2026-07-29

Release tag: `v0.1.0`

Release tag commit: `e138f2d2b1547a72343f51a2fa5305956565458c`

Canonical component source:
`461d4d5ad7cb65bd919bdf5875e28812a3f5f8dc`

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

## Demo artifact

The original terminal demo used a real clean-profile Qwen/ZeroClaw transcript
and actual host fixture results. Idle local inference was explicitly removed
with a labeled jump cut; no desktop, private window, secret, or raw transaction
was captured. The current release asset was replaced on 2026-07-30; the
original render-specific review evidence below is retained as historical
provenance only.

```text
file      guardian-demo.mp4
duration  129.3 seconds
format    H.264, 1920x1080
size      8445979 bytes
sha256    5a591d9e1738f74ba7e3635d8a0b42bbcb6f19c78851938a5292fbb0230cb909
```

Five representative frames of the original render were visually reviewed for
readability and content. A second original render was byte-identical.

## Public release verification

The stable non-draft, non-prerelease GitHub release is public at:

<https://github.com/IagoPrandi/zeroclaw-plugin/releases/tag/v0.1.0>

The separate public demo is available at:

<https://github.com/IagoPrandi/zeroclaw-plugin/releases/download/v0.1.0/guardian-demo.mp4>

The five original release assets were downloaded from GitHub into a new empty
directory. On 2026-07-30, `guardian-demo.mp4` was replaced; GitHub's current
asset metadata reports the same byte count and SHA-256 as the local replacement
file. The public repository, release page, and direct asset URLs returned HTTP
200 without authentication.
