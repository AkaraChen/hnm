# Automatic agent hooks

Status: Accepted (KIT-928)

The generated-CLI migration removed the legacy documentation-review gate and replaced useful root instructions with empty Markdown. Restore useful AGENTS.md/spec defaults and automatic Claude/Codex hook installation on project `hnm init`.

The first direct git commit attempt in a session at a given HEAD returns a documentation-review reminder and denial. Retrying at the same HEAD is allowed; a new HEAD or session prompts again. This is a reminder, not proof that review happened and not a shell security boundary.

Init merges one PreToolUse Bash hook into `.claude/settings.json` and `.codex/hooks.json`, preserving unrelated settings and hooks. Both invoke the bundled Python 3 review script under `.codex/hooks/spec_doc_review.py`. Repeated init does not duplicate hooks. Existing scripts are preserved unless force is selected. Force resets harness defaults and the owned script but preserves unrelated settings. Invalid settings fail without overwriting them, including with force. Filesystem errors fail nonzero; no cross-file transaction is promised.

The generated package owns entities and bundled skills; hnm owns all runtime-specific installation. Hook installation does not change user/global runtime settings or approve hook trust. Python 3 and Git are runtime prerequisites. Codex requires project and hook trust; Claude may require a session restart after settings changes.

Acceptance: build/install entrypoint unchanged; init, show/write, repeat init, force, custom settings preservation, invalid settings, and first/retry/new-session review behavior pass runnable tests. Regeneration leaves the custom entrypoint intact. No stack presets, template engine, general hook framework, or upstream hnm policy.

Project init installs the complete bundled `feature-dev` and `git-commit` directories into `.agents/skills/` and creates per-skill links under `.claude/skills/`. Existing files/links are preserved by default; force refreshes bundled files and replaces conflicting links or files, but never deletes nonempty user directories. Other skills remain untouched. Existing legacy `.claude/skills -> ../.agents/skills` links remain supported. Skill content is obtained from the generated CLI package, not duplicated in a separate embed.
