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

For a terminal-only recording, run the real agent with
[`run_demo.ps1`](run_demo.ps1), review the sanitized transcript, and render the
validated story with:

```bash
python demo/render_video.py --output dist/0.1.0/guardian-demo.mp4
```

The rendered video identifies the removed idle-inference interval and never
captures the desktop, private windows, credentials, or raw transaction bytes.
The validated release render is 2:38, H.264 at 1600×900, with SHA-256
`1e3652197c3c3ae80c0911d626d1f102ff70b2e806fa6e17d3c17ccb3cd05828`.

Watch or download the
[public demo](https://github.com/IagoPrandi/zeroclaw-plugin/releases/download/v0.1.0/guardian-demo.mp4).
