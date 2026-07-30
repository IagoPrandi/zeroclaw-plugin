#!/usr/bin/env python3
"""Render a readable phone-and-terminal Guardian walkthrough to MP4.

The phone view is an explicitly labelled reconstruction of sanitized, recorded
agent exchanges. The terminal view contains the corresponding deterministic
Guardian evidence. It is not presented as a live third-party chat channel.
"""

from __future__ import annotations

import argparse
import shutil
import subprocess
import tempfile
from dataclasses import dataclass
from pathlib import Path


WIDTH = 1920
HEIGHT = 1080
DURATION = 166


@dataclass(frozen=True)
class Scene:
    start: float
    end: float
    step: str
    title: str
    user: tuple[str, ...]
    guardian: tuple[str, ...]
    terminal: tuple[str, ...]
    decision: str
    decision_style: str


SCENES = (
    Scene(0, 15, "01 / 06", "A LOCAL, READ-ONLY ZEROCLAW AGENT",
          ("Before I trust a Solana transaction,", "explain what it actually does."),
          ("I decode, simulate and apply policy.", "I cannot sign, broadcast or access keys."),
          ("$ ollama list", "qwen3.5:9b  |  local-only provider", "$ zeroclaw plugin list", "solana-transaction-guardian v0.1.0", "permissions: config_read, http_client"),
          "T0 / READ-ONLY", "DecisionInfo"),
    Scene(15, 48, "02 / 06", "WALKTHROUGH: A TRANSPARENT TRANSFER",
          ("Is this devnet payment safe to review?", "It should transfer exactly 1 lamport."),
          ("The candidate matches that intent.", "One System Program transfer was decoded.", "Simulation succeeded. No signing happened."),
          ("$ zeroclaw agent --agent guardian", "[tool] solana_transaction_guardian", "source: serialized  |  cluster: devnet", "decoded: SystemProgram::Transfer", "amount: 1 lamport  |  coverage: 1 / 1", "simulation: succeeded  |  150 compute units"),
          "ALLOW", "DecisionAllow"),
    Scene(48, 88, "03 / 06", "WALKTHROUGH: THE HIDDEN DELEGATE",
          ("Send only 1 lamport to the documented", "recipient. Is this transaction acceptable?"),
          ("No. It includes an undeclared token approval.", "A delegate would receive 1,000,000 raw units.", "The deterministic policy blocks it."),
          ("$ guardian fixture 02-hidden-delegate.json", "actions: transfer + SPL Token Approve", "unexpected authority change: delegate approval", "rules: AUTH-004, AUTH-005, INT-003", "intent: only System Program transfer permitted"),
          "BLOCK", "DecisionBlock"),
    Scene(88, 118, "04 / 06", "WALKTHROUGH: UNKNOWN DOES NOT MEAN SAFE",
          ("Does this follow my \"System Program only\"", "instruction?"),
          ("No. The transaction calls an unknown program.", "Coverage is incomplete, so policy fails closed."),
          ("$ guardian fixture 03-unknown-program.json", "action: unknown_program", "decoder: unavailable  |  coverage: incomplete", "rules: COV-003, EXEC-001, INT-001", "no unknown behavior is silently allowed"),
          "BLOCK", "DecisionBlock"),
    Scene(118, 145, "05 / 06", "WALKTHROUGH: VERSION 0 AND ADDRESS LOOKUPS",
          ("Can you also inspect a version-0", "transaction that uses an address lookup table?"),
          ("Yes. The lookup table was resolved before", "the transfer was decoded and simulated."),
          ("$ guardian fixture 04-v0-alt.json", "transaction version: v0", "address lookup table: resolved", "decoded action: transfer", "evidence remains explicit in the JSON report"),
          "ALLOW", "DecisionAllow"),
    Scene(145, 166, "06 / 06", "WHAT THE OPERATOR CAN REPRODUCE",
          ("What should I trust before I sign?",),
          ("The canonical JSON report, its findings and", "its coverage. Never an AI explanation alone."),
          ("60 native tests  |  strict Clippy  |  WASI", "30 / 30 local-Qwen decision-preservation checks", "signed manifest  |  reproducible release archive", "github.com/IagoPrandi/zeroclaw-plugin"),
          "VERIFY BEFORE YOU TRUST", "DecisionInfo"),
)


def ass_time(seconds: float) -> str:
    centiseconds = round(seconds * 100)
    hours, remainder = divmod(centiseconds, 360_000)
    minutes, remainder = divmod(remainder, 6_000)
    whole_seconds, fraction = divmod(remainder, 100)
    return f"{hours}:{minutes:02}:{whole_seconds:02}.{fraction:02}"


def escape_ass(value: str) -> str:
    return value.replace("\\", r"\\").replace("{", r"\{").replace("}", r"\}").replace("\n", r"\N")


def event(start: float, end: float, style: str, x: int, y: int, value: str) -> str:
    return f"Dialogue: 0,{ass_time(start)},{ass_time(end)},{style},,0,0,0,,{{\\pos({x},{y})}}{escape_ass(value)}"


def build_ass() -> str:
    header = f"""[Script Info]
ScriptType: v4.00+
PlayResX: {WIDTH}
PlayResY: {HEIGHT}
WrapStyle: 2

[V4+ Styles]
Format: Name,Fontname,Fontsize,PrimaryColour,SecondaryColour,OutlineColour,BackColour,Bold,Italic,Underline,StrikeOut,ScaleX,ScaleY,Spacing,Angle,BorderStyle,Outline,Shadow,Alignment,MarginL,MarginR,MarginV,Encoding
Style: Header,Arial,36,&H00E6EDF3,&H00E6EDF3,&H00000000,&H00000000,-1,0,0,0,100,100,0,0,1,0,0,7,0,0,0,1
Style: Step,Arial,22,&H0094A3B8,&H0094A3B8,&H00000000,&H00000000,-1,0,0,0,100,100,1,0,1,0,0,9,0,0,0,1
Style: PanelLabel,Arial,20,&H0094A3B8,&H0094A3B8,&H00000000,&H00000000,-1,0,0,0,100,100,1,0,1,0,0,7,0,0,0,1
Style: PhoneUser,Arial,27,&H00F8FAFC,&H00F8FAFC,&H00000000,&H00000000,0,0,0,0,100,100,0,0,1,0,0,7,0,0,0,1
Style: PhoneGuardian,Arial,25,&H00D1FAE5,&H00D1FAE5,&H00000000,&H00000000,0,0,0,0,100,100,0,0,1,0,0,7,0,0,0,1
Style: Terminal,Consolas,25,&H00E6EDF3,&H00E6EDF3,&H00000000,&H00000000,0,0,0,0,100,100,0,0,1,0,0,7,0,0,0,1
Style: TerminalDim,Consolas,22,&H008B949E,&H008B949E,&H00000000,&H00000000,0,0,0,0,100,100,0,0,1,0,0,7,0,0,0,1
Style: DecisionAllow,Arial,40,&H0086EFAC,&H0086EFAC,&H00000000,&H00000000,-1,0,0,0,100,100,1,0,1,0,0,7,0,0,0,1
Style: DecisionBlock,Arial,40,&H008CA2FF,&H008CA2FF,&H00000000,&H00000000,-1,0,0,0,100,100,1,0,1,0,0,7,0,0,0,1
Style: DecisionInfo,Arial,28,&H00A7F3D0,&H00A7F3D0,&H00000000,&H00000000,-1,0,0,0,100,100,1,0,1,0,0,7,0,0,0,1
Style: Footer,Arial,18,&H008B949E,&H008B949E,&H00000000,&H00000000,0,0,0,0,100,100,0,0,1,0,0,2,0,0,0,1

[Events]
Format: Layer,Start,End,Style,Name,MarginL,MarginR,MarginV,Effect,Text
"""
    events = [
        event(0, DURATION, "Footer", 960, 1050, "SANITIZED WALKTHROUGH • RECORDED AGENT RESULTS • NO KEYS OR RAW TRANSACTION BYTES"),
        event(0, DURATION, "PanelLabel", 105, 176, "PHONE VIEW — SANITIZED OPERATOR CHAT"),
        event(0, DURATION, "PanelLabel", 815, 176, "TERMINAL — DETERMINISTIC GUARDIAN EVIDENCE"),
        event(0, DURATION, "TerminalDim", 815, 225, "$ session: local Ollama / ZeroClaw v0.8.3 / devnet"),
    ]
    for scene in SCENES:
        events.extend((
            event(scene.start, scene.end, "Header", 80, 58, scene.title),
            event(scene.start, scene.end, "Step", 1840, 62, scene.step),
            event(scene.start + 0.8, scene.end, "PanelLabel", 108, 245, "OPERATOR"),
            event(scene.start + 1.2, scene.end, "PhoneUser", 108, 283, "\n".join(scene.user)),
            event(scene.start + 5.5, scene.end, "PanelLabel", 108, 505, "GUARDIAN"),
            event(scene.start + 5.9, scene.end, scene.decision_style, 108, 545, scene.decision),
            event(scene.start + 7.0, scene.end, "PhoneGuardian", 108, 610, "\n".join(scene.guardian)),
        ))
        for index, line in enumerate(scene.terminal):
            events.append(event(scene.start + 1.0 + index * 1.25, scene.end, "Terminal" if index < 2 else "TerminalDim", 815, 285 + index * 65, line))
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
    with tempfile.TemporaryDirectory(prefix="guardian-walkthrough-") as directory:
        subtitles = Path(directory) / "walkthrough.ass"
        subtitles.write_text(build_ass(), encoding="utf-8", newline="\n")
        subtitle_filter_path = subtitles.as_posix().replace(":", r"\:")
        panels = (
            "drawbox=x=55:y=115:w=650:h=880:color=0x020617:t=fill,drawbox=x=57:y=117:w=646:h=876:color=0x334155:t=2,"
            "drawbox=x=80:y=205:w=600:h=750:color=0x0f1b31:t=fill,drawbox=x=80:y=205:w=600:h=750:color=0x1e3a5f:t=2,"
            "drawbox=x=760:y=115:w=1105:h=880:color=0x020617:t=fill,drawbox=x=762:y=117:w=1101:h=876:color=0x334155:t=2,"
            "drawbox=x=790:y=205:w=1045:h=750:color=0x09111f:t=fill,drawbox=x=790:y=205:w=1045:h=750:color=0x1e293b:t=2,"
            "drawbox=x=80:y=190:w=600:h=2:color=0x2dd4bf:t=fill,drawbox=x=790:y=190:w=1045:h=2:color=0x2dd4bf:t=fill"
        )
        subprocess.run([ffmpeg, "-y", "-hide_banner", "-loglevel", "warning", "-f", "lavfi", "-i", f"color=c=#07111f:s={WIDTH}x{HEIGHT}:r=30:d={DURATION}", "-vf", f"{panels},subtitles=filename='{subtitle_filter_path}'", "-c:v", "libx264", "-preset", "medium", "-crf", "18", "-pix_fmt", "yuv420p", "-movflags", "+faststart", str(output)], check=True)
    print(output)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
