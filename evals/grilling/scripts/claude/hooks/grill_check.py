#!/usr/bin/env python3
"""Inject a stop-or-ask check into every user turn while a feature 质问 is active."""

from __future__ import annotations

import json
from pathlib import Path
import sys


CHECK_CONTEXT = """[质问 check] Before you reply to this answer:
1. Record what the user just decided. Do not re-ask anything already decided.
2. Pick the candidate next question. Name the exact sentence in docs/prd, docs/adr, or docs/spec.md that would be written differently depending on the answer. If you cannot name two different sentences, do not ask it.
3. Also do not ask if the point is safely reversible, already conventional in this repository, or discoverable from the code; record it as an assumption instead.
4. If a question survives, start your reply with one line `Decides: <doc> — <what differs>` and ask exactly that one question.
5. If no question survives, stop 质问 now: list the assumptions you made in one short block for the user to veto, then proceed to writing the documents."""


def grilling_active(payload: dict[str, object]) -> bool:
    transcript = payload.get("transcript_path")
    if not isinstance(transcript, str):
        return False
    try:
        text = Path(transcript).read_text(errors="ignore")
    except OSError:
        return False
    if "feature-dev" not in text:
        return False

    cwd_value = payload.get("cwd")
    cwd = Path(cwd_value) if isinstance(cwd_value, str) else Path.cwd()
    prd_dir = cwd / "docs" / "prd"
    return not any(prd_dir.glob("*.md"))


def main() -> int:
    try:
        payload = json.load(sys.stdin)
    except (json.JSONDecodeError, OSError):
        return 0
    if not isinstance(payload, dict) or not grilling_active(payload):
        return 0

    json.dump(
        {
            "hookSpecificOutput": {
                "hookEventName": "UserPromptSubmit",
                "additionalContext": CHECK_CONTEXT,
            }
        },
        sys.stdout,
    )
    sys.stdout.write("\n")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
