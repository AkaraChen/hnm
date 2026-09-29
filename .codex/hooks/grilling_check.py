#!/usr/bin/env python3
"""Supply the English reminder for feature clarification on each user turn."""

from __future__ import annotations

import json
import sys


CHECK_CONTEXT = """[grilling-check]
Apply this check only while feature-dev clarification is active, including confirmation of its closing checklist. Outside that phase, continue the current task normally.

First reconcile the latest answer. Track three categories: already decided, verifiable from the code, and still requiring the user's decision. Do not ask again about settled matters.
Internally assess the value of the next question: would different answers change user-visible behavior, acceptance criteria, scope, compatibility, or a major architectural constraint? Different outcomes do not by themselves mean the user must decide. Ask only when missing user goals, preferences, or constraints prevent you from making a reasonable choice. Decide independently when confirmed requirements and repository facts are sufficient. Do not expose this internal check to the user.
Do not skip important user-visible behavior merely because it can be changed later. Check the normal path, empty or missing values, conflicts, compatibility with existing behavior, and non-goals directly relevant to this requirement. Do not mechanically expand into unrelated checklists.
Verify facts discoverable from the code yourself. Follow repository conventions for implementation details, test organization, and naming; do not ask the user to decide them separately.
If a material ambiguity remains, ask only the single highest-impact question, briefly explain why the decision is needed, and wait for the answer.
If material ambiguities are resolved, present a short closing checklist of confirmed contracts and a few default assumptions still needing confirmation. Ask the user to confirm and stop. Do not write documentation in that turn. Do not bury significant, previously unasked product disagreements in a long checklist.
After the user confirms the checklist and no new contradiction remains, write the PRD, ADR, and spec directly. Do not start another round of detailed questions. If the user corrects the checklist, incorporate the correction first; ask again only if it introduces a new material ambiguity.

When the user says "use your judgment" or "follow convention," record the scope of the decisions they have delegated. Do not request item-by-item confirmation within that scope. Raise it again only if you discover a conflict with confirmed requirements."""


def main() -> int:
    try:
        payload = json.load(sys.stdin)
    except (ValueError, OSError):
        return 0
    if not isinstance(payload, dict):
        return 0
    if payload.get("hook_event_name") != "UserPromptSubmit":
        return 0
    if not isinstance(payload.get("prompt"), str):
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
