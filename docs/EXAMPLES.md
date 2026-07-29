# Examples

These requests use devnet and contain no secret. The complete machine-readable
tool schema is in `tests/fixtures/tool-schema.json`.

## Analyze a serialized candidate

```json
{
  "source": {
    "type": "serialized",
    "transaction_base64": "AQAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAABAAEDAQEBAQEBAQEBAQEBAQEBAQEBAQEBAQEBAQEBAQEBAQECAgICAgICAgICAgICAgICAgICAgICAgICAgICAgICAgAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAABAgIAAQwCAAAAAQAAAAAAAAA="
  },
  "cluster": "devnet",
  "observed_wallets": [
    "4vJ9JU1bJJE96FWSJKvHsmmFADCg4gpZQff4P3bkLKi"
  ],
  "output_language": "en"
}
```

This versioned fixture is a simple System transfer candidate. Because its
blockhash is intentionally static, live simulation may report expiry; the
Guardian must expose that execution limitation rather than silently claim
success.

## Analyze a confirmed signature

```json
{
  "source": {
    "type": "confirmed",
    "signature": "49TXiM9rVcpspUZtT5LynRePwWyazTERXAhYHAsTSAz7fi4TtYspwiUM1zuyGmoRoddJKM9M5K47FPfPcimhufu4"
  },
  "cluster": "devnet",
  "output_language": "en"
}
```

The pinned evidence transaction is expected to produce `block` with partial
coverage finding `COV-003`; the exact confirmed slot and fees are recorded in
`docs/evidence/host-e2e/confirmed-devnet.md`.

## Add structured intent

```json
{
  "source": {
    "type": "serialized",
    "transaction_base64": "<BASE64>"
  },
  "cluster": "devnet",
  "observed_wallets": ["<PAYER_ADDRESS>"],
  "expected_intent": {
    "description": "Pay 0.05 SOL to the documented recipient",
    "allowed_programs": ["11111111111111111111111111111111"],
    "allowed_recipients": ["<RECIPIENT_ADDRESS>"],
    "max_sol_out_lamports": "50000000",
    "token_limits": []
  },
  "output_language": "en"
}
```

An unexpected recipient, extra program, or excess outgoing amount produces
stable intent findings. Limits use decimal strings in base units.

## Ask the local agent

```bash
zeroclaw agent --agent guardian \
  --config-dir /path/to/guardian-profile \
  --message "Analyze this devnet transaction and preserve the Guardian decision: <BASE64>"
```

Expected presentation:

1. literal `allow`, `review`, or `block`;
2. major decoded actions and value effects;
3. every critical/high blocking finding;
4. coverage/confidence limitations;
5. no request for a seed, signature, or broadcast permission.

Additional demo fixtures are kept in `demo/fixtures/` and are generated from
transparent source descriptions rather than opaque secrets.
