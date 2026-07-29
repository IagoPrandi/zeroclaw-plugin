# M8 security review

Date: 2026-07-29

## Automated checks

- `cargo audit --json`: 220 locked dependencies, zero vulnerabilities.
- Informational advisory: `RUSTSEC-2025-0141`, `bincode 1.3.3` unmaintained.
  The crate remains pinned because Solana legacy/v0 transaction wire decoding
  uses its established format. It is not a reported vulnerability; migration
  requires a separately validated Solana wire implementation.
- Runtime dependency tree contains none of `reqwest`, `tokio`,
  `solana-client`, `solana-rpc-client`, OpenSSL, signer, or keypair crates.
- `unsafe_code = "deny"`, `unwrap_used = "deny"`, and
  `expect_used = "deny"` are active. Repository scans found no `unsafe`,
  `unwrap`, `expect`, `panic!`, `todo!`, or `unimplemented!` in `src`.
- Strict Clippy with all targets and all features passed.

## Manual review

- Permissions: `manifest.toml` declares exactly `config_read` and
  `http_client`.
- Endpoint provenance: public arguments expose only a cluster alias. HTTPS
  endpoints and localhost HTTP are accepted from operator configuration.
- Redirects: Waki's WASI request path performs one outgoing request and has no
  redirect-following loop; any 3xx response fails the client's 2xx check.
- Response limits: both the transport and RPC client enforce the configured
  byte cap. RPC calls and account batches are bounded.
- Logs: host logs contain lifecycle metadata only. Transaction Base64,
  signatures, account data, arguments, and complete reports are not emitted by
  the plugin logger. RPC logs are bounded to 1,000 lines and 4,096 characters
  per line with `logs_truncated=true`.
- Arithmetic: monetary and fee paths use integer units, checked arithmetic,
  `u128` intermediates, and controlled overflow findings/errors.
- Panics: hostile wire data has property coverage and explicit malformed
  Base64, signature-count, index, ALT, TLV, RPC, and return-data cases.

## Resource limits

The actual Guardian component was instantiated by the pinned ZeroClaw v0.8.3
host:

- a one-unit fuel budget trapped the call;
- a one-byte memory cap rejected instantiation;
- the documented 256 MiB memory cap instantiated and returned metadata;
- the optimized WASM remained below 1 MiB.

Result: no critical security defect is open.
