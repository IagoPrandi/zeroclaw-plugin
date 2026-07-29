# Reproducible release build

## Canonical environment

Release artifacts are built in this pinned Linux container:

```text
rust@sha256:a339861ae23e9abb272cea45dfafde21760d2ce6577a70f8a926153677902663
Rust 1.96.1
target wasm32-wasip2
```

The source directory is mounted read-only and Cargo writes to an empty,
separate target directory.

## Reproduction result

Two independent clean target directories produced byte-identical components:

```text
size    775829 bytes
sha256  780d7a88aa09eadcb345a7bfa6fd58e80cd93de27baa425d71439d6987e5c7e5
```

The first container also ran the complete CI script:

- 60 native tests passed;
- formatting passed;
- strict Clippy passed;
- locked optimized WASI build passed.

## Cross-host note

The Windows GNU toolchain produced a functionally equivalent component with a
different size/hash. Cross-host LLVM output is therefore not declared
bit-reproducible. The pinned Linux container is the sole canonical release
environment; release checksums must be generated from that artifact.
