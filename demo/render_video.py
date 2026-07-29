#!/usr/bin/env python3
"""Render the sanitized, validated Guardian terminal demo to an MP4."""

from __future__ import annotations

import argparse
import shutil
import subprocess
import tempfile
from dataclasses import dataclass
from pathlib import Path


@dataclass(frozen=True)
class Scene:
    start: float
    end: float
    title: str
    lines: tuple[str, ...]


SCENES = (
    Scene(
        0,
        14,
        "SOLANA TRANSACTION GUARDIAN",
        (
            "$ ollama list",
            "qwen3.5:9b   6488c96fa5fa   6.6 GB",
            "Provider: http://127.0.0.1:11434",
            "Cloud fallback: none",
            "Custody: T0 / read-only",
        ),
    ),
    Scene(
        14,
        29,
        "SIGNED ZEROCLAW V0.8.3 PLUGIN",
        (
            "$ zeroclaw plugin list",
            "solana-transaction-guardian v0.1.0",
            "Capabilities: [Tool]",
            "Permissions: [ConfigRead, HttpClient]",
            "Signature mode: strict",
        ),
    ),
    Scene(
        29,
        42,
        "REAL AGENT CALL - SAFE TRANSFER",
        (
            "$ zeroclaw agent --agent guardian",
            "Analyze this serialized devnet transaction.",
            "Preserve raw integer lamports exactly.",
            "Never sign or broadcast.",
            "[271 s local CPU inference removed with a jump cut]",
        ),
    ),
    Scene(
        42,
        73,
        "QWEN PRESENTS THE CANONICAL REPORT",
        (
            "Decision: allow",
            "Risk level: low",
            "Action: System Program transfer",
            "From: 4vJ9...bkLKi",
            "To:   8qbH...VfeR",
            "Amount transferred: 1 lamport",
            "All top-level instructions decoded (1/1)",
            "Simulation succeeded; 150 compute units",
            "Limitation: base-fee estimation unavailable",
            "Guardian JSON remains authoritative",
        ),
    ),
    Scene(
        73,
        94,
        "HIDDEN DELEGATE FIXTURE - ACTUAL HOST RESULT",
        (
            "$ guardian fixture 02-hidden-delegate.json",
            "Actions: transfer + approve",
            "Decision: block",
            "Rules: AUTH-004, AUTH-005, COV-006",
            "       INT-003, EXEC-001, INT-001",
            "The extra token delegate is not hidden.",
        ),
    ),
    Scene(
        94,
        111,
        "UNKNOWN PROGRAM - FAIL CLOSED",
        (
            "$ guardian fixture 03-unknown-program.json",
            "Action: unknown_program",
            "Decision: block",
            "Rules: EXEC-001, COV-003, INT-001",
            "Unknown behavior is never silently allowed.",
        ),
    ),
    Scene(
        111,
        128,
        "VERSION 0 + ADDRESS LOOKUP TABLE",
        (
            "$ guardian fixture 04-v0-alt.json",
            "Transaction version: v0",
            "Address lookup table resolved: true",
            "Action: transfer",
            "Decision: allow",
        ),
    ),
    Scene(
        128,
        147,
        "REPRODUCIBLE RELEASE",
        (
            "60 native tests | strict Clippy | WASI",
            "30/30 local-Qwen behavior checks",
            "WASM  SHA-256  780d7a88...c7e5",
            "ZIP   SHA-256  70a3ac35...1b3b",
            "Signed manifest | strict trust verified",
        ),
    ),
    Scene(
        147,
        158,
        "VERIFY BEFORE YOU TRUST",
        (
            "Read-only and advisory; it never signs.",
            "RPC state and simulation are point-in-time.",
            "Unknown protocols can reduce coverage.",
            "Deterministic JSON is the authority.",
            "github.com/IagoPrandi/zeroclaw-plugin",
        ),
    ),
)


def ass_time(seconds: float) -> str:
    centiseconds = round(seconds * 100)
    hours, remainder = divmod(centiseconds, 360_000)
    minutes, remainder = divmod(remainder, 6_000)
    whole_seconds, fraction = divmod(remainder, 100)
    return f"{hours}:{minutes:02}:{whole_seconds:02}.{fraction:02}"


def escape_ass(value: str) -> str:
    return (
        value.replace("\\", r"\\")
        .replace("{", r"\{")
        .replace("}", r"\}")
        .replace("\n", r"\N")
    )


def build_ass() -> str:
    header = """[Script Info]
ScriptType: v4.00+
PlayResX: 1600
PlayResY: 900
WrapStyle: 2

[V4+ Styles]
Format: Name, Fontname, Fontsize, PrimaryColour, SecondaryColour, OutlineColour, BackColour, Bold, Italic, Underline, StrikeOut, ScaleX, ScaleY, Spacing, Angle, BorderStyle, Outline, Shadow, Alignment, MarginL, MarginR, MarginV, Encoding
Style: Title,Consolas,40,&H00A7F3D0,&H00A7F3D0,&H000B1020,&H000B1020,-1,0,0,0,100,100,0,0,1,0,0,7,85,85,62,1
Style: Body,Consolas,31,&H00E6EDF3,&H00E6EDF3,&H000B1020,&H000B1020,0,0,0,0,100,100,0,0,1,0,0,7,85,85,128,1
Style: Footer,Consolas,22,&H008B949E,&H008B949E,&H000B1020,&H000B1020,0,0,0,0,100,100,0,0,1,0,0,2,85,85,34,1

[Events]
Format: Layer, Start, End, Style, Name, MarginL, MarginR, MarginV, Effect, Text
"""
    events: list[str] = []
    for scene in SCENES:
        start = ass_time(scene.start)
        end = ass_time(scene.end)
        events.append(
            f"Dialogue: 0,{start},{end},Title,,0,0,0,,{escape_ass(scene.title)}"
        )
        for index in range(len(scene.lines)):
            line_start = min(scene.start + 1 + index * 0.55, scene.end - 0.25)
            if index + 1 < len(scene.lines):
                line_end = min(
                    scene.start + 1 + (index + 1) * 0.55,
                    scene.end,
                )
            else:
                line_end = scene.end
            visible = r"\N".join(escape_ass(line) for line in scene.lines[: index + 1])
            events.append(
                "Dialogue: 0,"
                f"{ass_time(line_start)},{ass_time(line_end)},Body,,0,0,0,,{visible}"
            )
    events.append(
        "Dialogue: 0,0:00:00.00,0:02:38.00,Footer,,0,0,0,,"
        "Recorded outputs; idle inference removed. No secrets or desktop capture."
    )
    return header + "\n".join(events) + "\n"


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()

    ffmpeg = shutil.which("ffmpeg")
    if not ffmpeg:
        raise SystemExit("ffmpeg is required")

    output = args.output.resolve()
    output.parent.mkdir(parents=True, exist_ok=True)
    with tempfile.TemporaryDirectory(prefix="guardian-demo-") as directory:
        subtitles = Path(directory) / "demo.ass"
        subtitles.write_text(build_ass(), encoding="utf-8", newline="\n")
        subtitle_filter_path = (
            subtitles.as_posix().replace("\\", r"\\").replace(":", r"\:")
        )
        subprocess.run(
            [
                ffmpeg,
                "-y",
                "-hide_banner",
                "-loglevel",
                "warning",
                "-f",
                "lavfi",
                "-i",
                "color=c=#0b1020:s=1600x900:r=30:d=158",
                "-vf",
                f"subtitles=filename='{subtitle_filter_path}'",
                "-c:v",
                "libx264",
                "-preset",
                "medium",
                "-crf",
                "18",
                "-pix_fmt",
                "yuv420p",
                "-movflags",
                "+faststart",
                str(output),
            ],
            check=True,
        )
    print(output)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
