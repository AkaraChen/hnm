# ADR: Stateless v3b clarification reminder

## Status

Accepted.

## Decision

Embed a shared Python command hook in the existing .codex/hooks directory and
register it for UserPromptSubmit in both runtime configurations. Emit the English
v3b translation as additionalContext, prefaced with an explicit feature-dev
clarification scope. Keep the repository's installed copy in sync with the template.

## Alternatives and trade-offs

The archived experiment checks transcript text and the absence of any PRD. That
heuristic fails for existing projects and later features. A session state machine
would require reliable skill lifecycle signals that the current harness does not
provide. A scoped reminder avoids those false negatives and transcript reads,
at the cost of adding context on every user turn and relying on the agent to
recognize the current phase. It is advisory, not an enforced workflow gate.

Use the existing Python runtime and JSON protocol, with no dependencies, network
access, filesystem writes, or permission decisions. Command paths follow existing
hook conventions and require a Git project and Python 3; Codex uses /usr/bin/python3.
Malformed input is a no-op. New scripts use the existing Keep policy, while hook
registrations use JsonMerge. No archived experiment data is changed.

## Validation

Run hook protocol tests, Cargo tests, formatting and lint checks; smoke-test fresh
init and migration from existing hook settings, including repeated init.
