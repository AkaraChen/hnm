#!/usr/bin/env python3
"""Run one feature-dev 质问 against a simulated requester, with or without the check hook.

Usage: run_case.py <case_id> <arm: base|hook> <trial> [--max-turns N]
"""

from __future__ import annotations

import argparse
import json
from pathlib import Path
import shutil
import subprocess
import time
import uuid

ROOT = Path(__file__).resolve().parent
JUST_SRC = Path("/tmp/just-src")
HNM = ROOT.parent / "hnm" / "target" / "release" / "hnm"
HOOK = ROOT / "hooks" / "grill_check.py"
SCRATCH = Path("/tmp/grill-eval-scratch")

AGENT_MODEL = "opus"
SIM_MODEL = "sonnet"

ISOLATION = ["--setting-sources", "project", "--strict-mcp-config"]
AGENT_DENY = ["WebFetch", "WebSearch", "Bash(gh:*)", "Bash(curl:*)", "Bash(wget:*)"]

SIM_SYSTEM = """你在扮演一个软件需求的提出者，正在和一个 AI 编码助手对话。助手会用“质问”的方式逐个问你问题来澄清需求。

你的隐藏背景（只在被问到相关内容时透露，不要主动一次性倒出）：
{brief}

回答规则：
- 用中文，像真人一样简短（通常 1-3 句），只回答被问到的那一个问题。
- 问题命中上面的某条决定时，按那条决定如实回答。
- 问题不在上面的范围内时，回答“没想过，按你觉得合理/符合仓库惯例的来”，可以稍作改写，但不要编造新的硬性要求。
- 如果问的是你已经回答过的内容，简短指出“前面说过了”并复述。
- 如果助手在列假设、请你确认、或说要开始写文档，而内容与你的背景不冲突，就回复“可以，写吧”；有冲突就只指出冲突点。
- 如果助手没有提问也没有请求确认，回复“继续”。
只输出你要说的话本身。"""


def sh(cmd: list[str], cwd: Path | None = None, **kw) -> subprocess.CompletedProcess:
    return subprocess.run(cmd, cwd=cwd, check=True, text=True, capture_output=True, **kw)


def setup_repo(case: dict, arm: str, run_dir: Path) -> Path:
    repo = run_dir / "repo"
    if repo.exists():
        shutil.rmtree(repo)
    repo.mkdir(parents=True)
    # Export a snapshot without history so the merged PR cannot leak through git.
    archive = subprocess.run(
        ["git", "archive", case["base"]], cwd=JUST_SRC, check=True, capture_output=True
    ).stdout
    subprocess.run(["tar", "-x"], cwd=repo, input=archive, check=True)
    sh([str(HNM), "init", str(repo), "--stack", "rust", "--name", "just"])

    if arm == "hook":
        settings_path = repo / ".claude" / "settings.json"
        settings = json.loads(settings_path.read_text())
        settings.setdefault("hooks", {})["UserPromptSubmit"] = [
            {"hooks": [{"type": "command", "command": f"python3 {HOOK}", "timeout": 10}]}
        ]
        settings_path.write_text(json.dumps(settings, indent=2) + "\n")

    sh(["git", "init", "-q"], cwd=repo)
    sh(["git", "add", "-A"], cwd=repo)
    sh(["git", "-c", "user.name=eval", "-c", "user.email=eval@example.com",
        "commit", "-qm", "baseline"], cwd=repo)
    return repo


def claude_json(args: list[str], cwd: Path, timeout: int = 1800) -> dict:
    proc = subprocess.run(
        ["claude", "-p", "--output-format", "json", *args],
        cwd=cwd, text=True, capture_output=True, timeout=timeout,
    )
    if proc.returncode != 0:
        raise RuntimeError(f"claude failed ({proc.returncode}): {proc.stderr[-2000:]}\n{proc.stdout[-2000:]}")
    return json.loads(proc.stdout)


def agent_turn(repo: Path, session: str, message: str, first: bool) -> dict:
    args = [
        "--model", AGENT_MODEL, *ISOLATION,
        "--permission-mode", "bypassPermissions",
        "--disallowedTools", *AGENT_DENY,
    ]
    args += ["--session-id", session] if first else ["--resume", session]
    return claude_json([*args, "--", message], cwd=repo)


def sim_turn(case: dict, dialogue: list[dict]) -> dict:
    SCRATCH.mkdir(exist_ok=True)
    shown = "\n\n".join(
        f"【{'助手' if t['role'] == 'agent' else '你'}】\n{t['text']}" for t in dialogue
    )
    prompt = f"以下是到目前为止的对话：\n\n{shown}\n\n请以“你”的身份回复助手最后一条消息。"
    return claude_json(
        ["--model", SIM_MODEL, *ISOLATION, "--tools", "",
         "--system-prompt", SIM_SYSTEM.format(brief=case["brief"]), "--", prompt],
        cwd=SCRATCH, timeout=600,
    )


def prd_written(repo: Path) -> bool:
    return any((repo / "docs" / "prd").glob("*.md"))


def main() -> None:
    ap = argparse.ArgumentParser()
    ap.add_argument("case")
    ap.add_argument("arm", choices=["base", "hook"])
    ap.add_argument("trial", type=int)
    ap.add_argument("--max-turns", type=int, default=30)
    a = ap.parse_args()

    case = json.loads((ROOT / "cases" / f"{a.case}.json").read_text())
    run_dir = ROOT / "runs" / a.case / f"{a.arm}-t{a.trial}"
    run_dir.mkdir(parents=True, exist_ok=True)
    repo = setup_repo(case, a.arm, run_dir)

    session = str(uuid.uuid4())
    dialogue: list[dict] = [{"role": "user", "text": case["opening"]}]
    log = {"case": a.case, "arm": a.arm, "trial": a.trial, "session": session,
           "agent_cost_usd": 0.0, "sim_cost_usd": 0.0, "turns": 0, "end": None}
    started = time.time()

    message = case["opening"]
    for turn in range(a.max_turns):
        res = agent_turn(repo, session, message, first=(turn == 0))
        log["agent_cost_usd"] += res.get("total_cost_usd", 0.0)
        log["turns"] = turn + 1
        dialogue.append({"role": "agent", "text": res.get("result", ""), "wrote_prd": prd_written(repo)})
        (run_dir / "dialogue.json").write_text(json.dumps(dialogue, ensure_ascii=False, indent=2))
        if prd_written(repo):
            log["end"] = "docs_written"
            break
        sim = sim_turn(case, dialogue)
        log["sim_cost_usd"] += sim.get("total_cost_usd", 0.0)
        message = sim.get("result", "").strip() or "继续"
        dialogue.append({"role": "user", "text": message})
    else:
        log["end"] = "max_turns"

    log["seconds"] = round(time.time() - started)
    (run_dir / "dialogue.json").write_text(json.dumps(dialogue, ensure_ascii=False, indent=2))
    (run_dir / "log.json").write_text(json.dumps(log, indent=2))
    docs_out = run_dir / "docs"
    if docs_out.exists():
        shutil.rmtree(docs_out)
    shutil.copytree(repo / "docs", docs_out)
    print(json.dumps(log))


if __name__ == "__main__":
    main()
