#!/usr/bin/env python3
"""Grade finished 质问 runs: per-question labels, decision coverage, and programmatic metrics.

Usage: judge.py [run_dir ...]   (defaults to every runs/<case>/<arm>-t<n> with a log.json)
"""

from __future__ import annotations

import json
from pathlib import Path
import re
import subprocess
import sys

ROOT = Path(__file__).resolve().parent
SCRATCH = Path("/tmp/grill-eval-scratch")
JUDGE_MODEL = "opus"
INDIFFERENT = re.compile(r"没想过|按你(觉得|的|定)|你(来)?定|都行|无所谓")

JUDGE_SYSTEM = """You grade a requirements-clarification dialogue (质问) between an AI assistant and a requester.
You are given: the requester's hidden ground-truth decisions, the dialogue, and the documents the assistant wrote at the end.
Answer only with a single JSON object, no prose, matching this shape:
{
  "questions": [
    {"turn": <agent turn index>, "question": "<short paraphrase>",
     "decision": <1-based index of the ground-truth decision this question resolves, or null>,
     "label": "decisive" | "indifferent" | "discoverable" | "redundant" | "bundled"}
  ],
  "coverage": [
    {"decision": <1-based index>, "status": "correct" | "wrong" | "missing", "evidence": "<doc quote or why missing>"}
  ],
  "non_goal_violations": ["<non-goal the docs wrongly put in scope>"]
}
Label definitions (apply the first that is true):
- redundant: the requester already stated the answer earlier in the dialogue.
- discoverable: the answer was determinable from the repository code, the opening request, or conventions the assistant itself cited, and the requester added nothing new.
- bundled: the turn asks two or more independent decisions at once.
- decisive: the requester's answer supplies one of the numbered ground-truth decisions (set "decision" to its index), or the requester rejects the assistant's recommendation in favour of a different choice.
- indifferent: anything else, including "没想过, 按你来" and merely agreeing with the assistant's own recommendation on a point outside the ground truth.
Only grade agent turns marked "(still clarifying)". Turns marked "(documents already written)" are not questions. Include every clarifying turn that asks the requester something (a question or a request to choose/confirm).
Coverage: judge only from the final documents, not from the dialogue. "correct" means the documents state the decision consistently with the ground truth; "wrong" means they state something contradictory; "missing" means they do not settle it."""


def judge_run(run_dir: Path, case: dict) -> dict:
    dialogue = json.loads((run_dir / "dialogue.json").read_text())
    docs = []
    for p in sorted((run_dir / "docs").rglob("*.md")):
        docs.append(f"### {p.relative_to(run_dir / 'docs')}\n{p.read_text()}")

    turns = []
    agent_idx = 0
    for t in dialogue:
        if t["role"] == "agent":
            phase = "documents already written" if t.get("wrote_prd") else "still clarifying"
            turns.append(f"[agent turn {agent_idx} ({phase})]\n{t['text']}")
            agent_idx += 1
        else:
            turns.append(f"[requester]\n{t['text']}")

    decisions = "\n".join(f"{i + 1}. {d}" for i, d in enumerate(case["decisions"]))
    prompt = (
        f"# Ground-truth decisions\n{decisions}\n\n# Non-goals\n" + "\n".join(case["non_goals"])
        + "\n\n# Dialogue\n" + "\n\n".join(turns)
        + "\n\n# Final documents\n" + ("\n\n".join(docs) or "(none written)")
    )
    SCRATCH.mkdir(exist_ok=True)
    proc = subprocess.run(
        ["claude", "-p", "--output-format", "json", "--model", JUDGE_MODEL,
         "--setting-sources", "project", "--strict-mcp-config", "--tools", "",
         "--system-prompt", JUDGE_SYSTEM, "--", prompt],
        cwd=SCRATCH, text=True, capture_output=True, timeout=900, check=True,
    )
    text = json.loads(proc.stdout)["result"]
    match = re.search(r"\{.*\}", text, re.S)
    if not match:
        raise RuntimeError(f"judge returned no JSON for {run_dir}: {text[:500]}")
    return json.loads(match.group(0))


def programmatic(dialogue: list[dict]) -> dict:
    replies = [t["text"] for t in dialogue[1:] if t["role"] == "user"]
    flags = [bool(INDIFFERENT.search(r)) for r in replies]
    half = len(flags) // 2
    return {
        "user_replies": len(replies),
        "indifferent_replies": sum(flags),
        "indifferent_first_half": sum(flags[:half]),
        "indifferent_second_half": sum(flags[half:]),
    }


def main() -> None:
    dirs = [Path(p) for p in sys.argv[1:]] or sorted(p.parent for p in (ROOT / "runs").glob("*/*/log.json"))
    for run_dir in dirs:
        log = json.loads((run_dir / "log.json").read_text())
        case = json.loads((ROOT / "cases" / f"{log['case']}.json").read_text())
        dialogue = json.loads((run_dir / "dialogue.json").read_text())
        grade = judge_run(run_dir, case)
        qs = grade["questions"]
        labels = [q["label"] for q in qs]
        cov = [c["status"] for c in grade["coverage"]]
        summary = {
            **{k: log[k] for k in ("case", "arm", "trial", "turns", "end", "agent_cost_usd", "seconds")},
            **programmatic(dialogue),
            "questions": len(qs),
            **{f"q_{l}": labels.count(l) for l in ("decisive", "indifferent", "discoverable", "redundant", "bundled")},
            "wasted_rate": round(1 - labels.count("decisive") / len(qs), 3) if qs else None,
            "wasted_second_half": sum(l != "decisive" for l in labels[len(labels) // 2:]),
            "coverage_correct": cov.count("correct"),
            "coverage_wrong": cov.count("wrong"),
            "coverage_missing": cov.count("missing"),
            "coverage_total": len(case["decisions"]),
            "non_goal_violations": len(grade.get("non_goal_violations", [])),
        }
        (run_dir / "grade.json").write_text(json.dumps(grade, ensure_ascii=False, indent=2))
        (run_dir / "summary.json").write_text(json.dumps(summary, indent=2))
        print(json.dumps(summary))


if __name__ == "__main__":
    main()
