# Demo fixtures

All fixtures are public devnet data or offline-generated unsigned candidates.
They contain no private key, seed phrase, API credential, or live authority.

| Fixture | Demonstrates |
|---|---|
| `01-safe-transfer.json` | one transparent System transfer and canonical `allow` under the reference policy |
| `02-hidden-delegate.json` | payment-shaped candidate with an undeclared SPL Token delegate approval |
| `03-unknown-program.json` | unknown program plus structured-intent mismatch |
| `04-v0-alt.json` | version-0 candidate with an active devnet ALT |

The serialized candidates are reproducible:

```bash
cargo +1.96.1 run --locked --example generate_candidate -- simple
cargo +1.96.1 run --locked --example generate_candidate -- delegate
cargo +1.96.1 run --locked --example generate_candidate -- unknown
cargo +1.96.1 run --locked --example generate_candidate -- v0-alt
```

The v0 fixture references a public active devnet ALT. If an operator later
deactivates that table or the RPC cannot retrieve it, the correct result is a
fail-closed `COV-002` error; regenerate the fixture against another active
table rather than suppressing the failure.

The spoken walkthrough is in [DEMO_SCRIPT.md](DEMO_SCRIPT.md).
