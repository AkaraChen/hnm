use minijinja::{context, Environment};

use crate::error::{HnmError, Result};
use crate::stack::Stack;

const AGENTS_TEMPLATE: &str = include_str!("../templates/AGENTS.md.j2");
const SPEC_TEMPLATE: &str = include_str!("../templates/docs/spec.md.j2");

pub struct TemplateContext<'a> {
    pub project_name: &'a str,
    pub stack: Stack,
}

pub fn render_agents(ctx: &TemplateContext<'_>) -> Result<String> {
    render_named(
        "AGENTS.md.j2",
        AGENTS_TEMPLATE,
        ctx.project_name,
        ctx.stack,
    )
}

pub fn render_spec(ctx: &TemplateContext<'_>) -> Result<String> {
    render_named(
        "docs/spec.md.j2",
        SPEC_TEMPLATE,
        ctx.project_name,
        ctx.stack,
    )
}

fn render_named(
    name: &str,
    source: &str,
    project_name: &str,
    stack: Stack,
) -> Result<String> {
    let mut env = Environment::new();
    env.add_template(name, source)?;
    let tmpl = env
        .get_template(name)
        .map_err(|_| HnmError::MissingTemplate(name.to_string()))?;
    let rendered = tmpl.render(context! {
        project_name => project_name,
        commands => stack.commands_block(),
        stack => stack.to_string(),
    })?;
    Ok(rendered)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn agents_includes_name_and_rust_commands() {
        let out = render_agents(&TemplateContext {
            project_name: "demo",
            stack: Stack::Rust,
        })
        .unwrap();
        assert!(out.contains("`demo`"));
        assert!(out.contains("cargo test"));
        assert!(out.contains("$feature-dev"));
        assert!(out.contains("$git-commit"));
    }

    #[test]
    fn spec_includes_project_name() {
        let out = render_spec(&TemplateContext {
            project_name: "acme",
            stack: Stack::Generic,
        })
        .unwrap();
        assert!(out.contains("`acme`"));
        assert!(out.contains("docs/prd/"));
    }
}
