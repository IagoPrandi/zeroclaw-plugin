#!/usr/bin/env python3
"""Create a deterministic, signed ZeroClaw plugin release archive."""

from __future__ import annotations

import argparse
import hashlib
import stat
import tomllib
import zipfile
from pathlib import Path

ARCHIVE_TIMESTAMP = (2026, 1, 1, 0, 0, 0)
PLUGIN_NAME = "solana-transaction-guardian"
WASM_NAME = "solana_transaction_guardian.wasm"
PACKAGE_TEXT_FILES = ("manifest.toml", "README.md", "LICENSE")


def sha256(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as stream:
        for chunk in iter(lambda: stream.read(1024 * 1024), b""):
            digest.update(chunk)
    return digest.hexdigest()


def write_zip_entry(
    archive: zipfile.ZipFile, archive_name: str, content: bytes, executable: bool = False
) -> None:
    info = zipfile.ZipInfo(archive_name, ARCHIVE_TIMESTAMP)
    info.create_system = 3
    mode = stat.S_IFREG | (0o755 if executable else 0o644)
    info.external_attr = mode << 16
    info.compress_type = zipfile.ZIP_DEFLATED
    archive.writestr(info, content, compress_type=zipfile.ZIP_DEFLATED, compresslevel=9)


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--wasm", type=Path, required=True)
    parser.add_argument("--output-dir", type=Path, required=True)
    parser.add_argument("--version", default="0.1.0")
    parser.add_argument("--expected-wasm-sha256")
    args = parser.parse_args()

    root = Path(__file__).resolve().parent.parent
    manifest_path = root / "manifest.toml"
    with manifest_path.open("rb") as stream:
        manifest = tomllib.load(stream)

    if manifest.get("name") != PLUGIN_NAME:
        raise SystemExit("manifest name does not match the release package")
    if manifest.get("version") != args.version:
        raise SystemExit("manifest version does not match --version")
    if manifest.get("wasm_path") != WASM_NAME:
        raise SystemExit("manifest wasm_path does not match the canonical filename")
    if not manifest.get("signature") or not manifest.get("publisher_key"):
        raise SystemExit("manifest must contain signature and publisher_key")
    if not args.wasm.is_file():
        raise SystemExit(f"WASM does not exist: {args.wasm}")

    wasm_hash = sha256(args.wasm)
    if args.expected_wasm_sha256 and wasm_hash != args.expected_wasm_sha256.lower():
        raise SystemExit(
            f"WASM SHA-256 mismatch: expected {args.expected_wasm_sha256}, got {wasm_hash}"
        )

    output_dir = args.output_dir.resolve()
    output_dir.mkdir(parents=True, exist_ok=False)
    archive_name = f"{PLUGIN_NAME}-{args.version}.zip"
    archive_path = output_dir / archive_name
    archive_root = f"{PLUGIN_NAME}-{args.version}"

    with zipfile.ZipFile(archive_path, "x", allowZip64=False) as archive:
        for source_name in PACKAGE_TEXT_FILES:
            source = root / source_name
            if not source.is_file():
                raise SystemExit(f"required package file missing: {source_name}")
            write_zip_entry(
                archive,
                f"{archive_root}/{source_name}",
                source.read_bytes(),
            )
        write_zip_entry(
            archive,
            f"{archive_root}/{WASM_NAME}",
            args.wasm.read_bytes(),
        )

    standalone_wasm = output_dir / WASM_NAME
    standalone_wasm.write_bytes(args.wasm.read_bytes())
    (output_dir / "manifest.toml").write_bytes(manifest_path.read_bytes())
    archive_hash = sha256(archive_path)
    sums = (
        f"{archive_hash}  {archive_name}\n"
        f"{wasm_hash}  {WASM_NAME}\n"
    )
    (output_dir / "SHA256SUMS").write_text(sums, encoding="utf-8", newline="\n")

    print(f"archive={archive_path}")
    print(f"archive_sha256={archive_hash}")
    print(f"wasm_sha256={wasm_hash}")
    print(f"wasm_size={args.wasm.stat().st_size}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
