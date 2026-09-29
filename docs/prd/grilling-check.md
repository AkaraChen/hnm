# Grilling check

## Status and scope

Accepted: install the English translation of the modified third version (v3b)
from PR #13 for users of the feature-dev workflow.

## Requirements

On each user turn, remind the agent to apply v3b only while clarifying a feature.
Track settled decisions, repository facts, and decisions requiring user input;
ask only the most consequential unresolved question. Important observable
behavior must not be skipped merely because it is reversible. Once material
ambiguities are resolved, present a short contract and assumptions for confirmation,
then write PRD, ADR, and spec after confirmation. Respect prior delegation.

## Acceptance criteria

- Fresh init installs an English hook and registers UserPromptSubmit in both
  runtime configurations alongside the existing documentation review gate.
- Existing PRDs do not disable clarification for a new feature. Missing transcripts
  do not disable the reminder; activation is scoped in the text, not inferred from files.
- The hook only supplies context; it never blocks submission or writes files.
- Invalid input exits successfully without output. Unrelated hook events are ignored.
- Existing configuration and edited hook files follow the normal best-effort merge
  rules. Repeated init does not duplicate registrations.

## Non-goals

No new model evaluation, transcript classifier, persistent session state, changes
to the archived prompts, or runtime-independent enforcement of model behavior.
