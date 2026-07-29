# Publisher key

The v0.1.x release line is signed by this Ed25519 public key:

```text
d743b2cd62da45564844b273760776c076642cec487700fdedfc601100e5c96d
```

SHA-256 fingerprint of the 32 raw public-key bytes:

```text
b0651b156e8631f8f4c894e3d4fbdc3584bde2da9fb8a9184ab42996aea4bcbd
```

This public key is not a credential. Add it to ZeroClaw's
`plugins.security.trusted_publisher_keys` only after comparing it with the
signed manifest and GitHub release notes.

The private PKCS#8 key is stored outside the repository with a user-only ACL.
It is never included in source, build context, test fixtures, logs, or release
archives.
