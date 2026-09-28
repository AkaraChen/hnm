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

/// Insert or refresh the hnm-managed block in a Markdown document.
pub fn upsert_block(existing: &str, body: &str) -> Merged {
    let block = format!("{BLOCK_BEGIN}\n{}\n{BLOCK_END}", body.trim_end());
    let next = match block_span(existing) {
        Some((start, end)) => format!("{}{block}{}", &existing[..start], &existing[end..]),
        None => {
            let mut out = existing.trim_end().to_string();
            if !out.is_empty() {
                out.push_str("\n\n");
            }
            out.push_str(&block);
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
    merge_value(&mut merged, add);
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

fn block_span(text: &str) -> Option<(usize, usize)> {
    let start = text.find(BLOCK_BEGIN)?;
    let end = start + text[start..].find(BLOCK_END)? + BLOCK_END.len();
    Some((start, end))
}

fn merge_value(base: &mut Value, add: Value) {
    match (base, add) {
        (Value::Object(base), Value::Object(add)) => {
            for (key, value) in add {
                match base.get_mut(&key) {
                    Some(slot) => merge_value(slot, value),
                    None => {
                        base.insert(key, value);
                    }
                }
            }
        }
        (Value::Array(base), Value::Array(add)) => {
            for item in add {
                if !base.contains(&item) {
                    base.push(item);
                }
            }
        }
        _ => {}
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
    fn json_invalid_is_conflict() {
        assert_eq!(merge_json("{ nope", "{}"), Merged::Conflict);
        assert_eq!(merge_json("[1]", "{}"), Merged::Conflict);
    }
}
