//! Best-effort merges of harness content into files the user already owns.

use serde_json::Value;

pub const BLOCK_BEGIN: &str = "<!-- hnm:begin -->";
pub const BLOCK_END: &str = "<!-- hnm:end -->";

/// Outcome of folding generated content into an existing file.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Merged {
    /// The existing file already carries the generated content.
    Unchanged,
    /// New file content to write.
    Updated(String),
    /// The existing file cannot be merged safely.
    Conflict,
}

/// Text found in harness `AGENTS.md` content, used to recognise files written
/// before managed blocks existed.
const HARNESS_SIGNATURE: &str = "`$feature-dev` before implementation";

/// Wrap `body` in managed-block markers.
pub fn block(body: &str) -> String {
    format!("{BLOCK_BEGIN}\n{}\n{BLOCK_END}\n", body.trim_end())
}

/// Insert or refresh the hnm-managed block in `AGENTS.md`. A file without a
/// block that already carries the harness rules (written by hnm before 0.2.0)
/// is left alone rather than duplicated.
pub fn upsert_agents(existing: &str, body: &str) -> Merged {
    if matches!(block_span(existing), Span::Missing) && existing.contains(HARNESS_SIGNATURE) {
        return Merged::Conflict;
    }
    upsert_block(existing, body)
}

/// Insert or refresh the hnm-managed block in a Markdown document.
pub fn upsert_block(existing: &str, body: &str) -> Merged {
    let block = block(body);
    let block = block.trim_end();
    let next = match block_span(existing) {
        Span::Broken => return Merged::Conflict,
        Span::Found(start, end) => {
            format!("{}{block}{}", &existing[..start], &existing[end..])
        }
        Span::Missing => {
            let mut out = existing.trim_end().to_string();
            if !out.is_empty() {
                out.push_str("\n\n");
            }
            out.push_str(block);
            out.push('\n');
            out
        }
    };
    if next == existing {
        Merged::Unchanged
    } else {
        Merged::Updated(next)
    }
}

/// Ensure a Markdown file imports `target` via Claude's `@path` syntax.
pub fn ensure_import(existing: &str, target: &str) -> Merged {
    let import = format!("@{target}");
    if existing.lines().any(|line| line.trim() == import) {
        return Merged::Unchanged;
    }
    upsert_block(existing, &import)
}

/// Deep-merge `generated` JSON into `existing`: objects merge by key, arrays gain
/// missing items, and the user's scalar values win.
pub fn merge_json(existing: &str, generated: &str) -> Merged {
    if existing.trim().is_empty() {
        return Merged::Updated(generated.to_string());
    }
    let (Ok(original), Ok(add)) = (
        serde_json::from_str::<Value>(existing),
        serde_json::from_str::<Value>(generated),
    ) else {
        return Merged::Conflict;
    };
    if !original.is_object() {
        return Merged::Conflict;
    }
    let mut merged = original.clone();
    if !merge_value(&mut merged, add) {
        return Merged::Conflict;
    }
    if merged == original {
        return Merged::Unchanged;
    }
    match serde_json::to_string_pretty(&merged) {
        Ok(mut out) => {
            out.push('\n');
            Merged::Updated(out)
        }
        Err(_) => Merged::Conflict,
    }
}

enum Span {
    Missing,
    Found(usize, usize),
    /// Markers present but not a single well-formed begin/end pair.
    Broken,
}

fn block_span(text: &str) -> Span {
    let begins = text.matches(BLOCK_BEGIN).count();
    let ends = text.matches(BLOCK_END).count();
    match (begins, ends, text.find(BLOCK_BEGIN), text.find(BLOCK_END)) {
        (0, 0, _, _) => Span::Missing,
        (1, 1, Some(start), Some(end)) if start < end => Span::Found(start, end + BLOCK_END.len()),
        _ => Span::Broken,
    }
}

/// Merge `add` into `base`; returns false when their shapes are incompatible
/// (for example the user has `null` where the harness needs an object).
fn merge_value(base: &mut Value, add: Value) -> bool {
    match (base, add) {
        (Value::Object(base), Value::Object(add)) => add.into_iter().all(|(key, value)| match base
            .get_mut(&key)
        {
            Some(slot) => merge_value(slot, value),
            None => {
                base.insert(key, value);
                true
            }
        }),
        (Value::Array(base), Value::Array(add)) => {
            for item in add {
                if !base.contains(&item) {
                    base.push(item);
                }
            }
            true
        }
        (base, add) => !(base.is_object() || base.is_array() || add.is_object() || add.is_array()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn block_appends_then_refreshes() {
        let Merged::Updated(first) = upsert_block("# Mine\n", "v1") else {
            panic!("expected update");
        };
        assert!(first.starts_with("# Mine\n\n<!-- hnm:begin -->\nv1\n"));
        let Merged::Updated(second) = upsert_block(&first, "v2") else {
            panic!("expected update");
        };
        assert!(second.contains("v2") && !second.contains("v1"));
        assert!(second.starts_with("# Mine\n"));
        assert_eq!(upsert_block(&second, "v2"), Merged::Unchanged);
    }

    #[test]
    fn import_detects_existing_line() {
        assert_eq!(
            ensure_import("x\n@AGENTS.md\n", "AGENTS.md"),
            Merged::Unchanged
        );
        let Merged::Updated(out) = ensure_import("x\n", "AGENTS.md") else {
            panic!("expected update");
        };
        assert!(out.contains("\n@AGENTS.md\n"));
    }

    #[test]
    fn json_merges_hooks_and_keeps_user_values() {
        let existing = r#"{"model":"opus","hooks":{"PreToolUse":[{"matcher":"Edit"}]}}"#;
        let generated = r#"{"model":"x","hooks":{"PreToolUse":[{"matcher":"Bash"}]}}"#;
        let Merged::Updated(out) = merge_json(existing, generated) else {
            panic!("expected update");
        };
        let v: Value = serde_json::from_str(&out).unwrap();
        assert_eq!(v["model"], "opus");
        assert_eq!(v["hooks"]["PreToolUse"].as_array().unwrap().len(), 2);
        assert_eq!(merge_json(&out, generated), Merged::Unchanged);
    }

    #[test]
    fn broken_markers_are_conflicts() {
        let lone = format!("# x\n{BLOCK_BEGIN}\nold\n");
        assert_eq!(upsert_block(&lone, "v"), Merged::Conflict);
        let twice = format!("{}{}", block("a"), block("b"));
        assert_eq!(upsert_block(&twice, "v"), Merged::Conflict);
        let reversed = format!("{BLOCK_END}\n{BLOCK_BEGIN}\n");
        assert_eq!(upsert_block(&reversed, "v"), Merged::Conflict);
    }

    #[test]
    fn legacy_agents_is_not_duplicated() {
        let legacy = "# Project workflows\nuse `$feature-dev` before implementation.\n";
        assert_eq!(upsert_agents(legacy, "new"), Merged::Conflict);
        assert!(matches!(
            upsert_agents("# Mine\n", "new"),
            Merged::Updated(_)
        ));
        let managed = format!("# Mine\n\n{}", block(legacy));
        assert!(matches!(upsert_agents(&managed, "new"), Merged::Updated(_)));
    }

    #[test]
    fn json_shape_mismatch_is_conflict() {
        let generated = r#"{"hooks":{"PreToolUse":[1]}}"#;
        assert_eq!(merge_json(r#"{"hooks":null}"#, generated), Merged::Conflict);
        assert_eq!(
            merge_json(r#"{"hooks":{"PreToolUse":{}}}"#, generated),
            Merged::Conflict
        );
        assert_eq!(
            merge_json(r#"{"description":"mine"}"#, r#"{"description":"x"}"#),
            Merged::Unchanged
        );
    }

    #[test]
    fn json_invalid_is_conflict() {
        assert_eq!(merge_json("{ nope", "{}"), Merged::Conflict);
        assert_eq!(merge_json("[1]", "{}"), Merged::Conflict);
    }
}
