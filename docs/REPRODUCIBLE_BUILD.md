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
size    775945 bytes
sha256  c375d0319693e110afa4f1cef579b1b763e68ce371f5b64f19d63c65c099ba00
```

The first container also ran the complete CI script:

- 59 native tests passed;
- formatting passed;
- strict Clippy passed;
- locked optimized WASI build passed.

## Cross-host note

The Windows GNU toolchain produced a functionally equivalent component with a
different size/hash. Cross-host LLVM output is therefore not declared
bit-reproducible. The pinned Linux container is the sole canonical release
environment; release checksums must be generated from that artifact.
