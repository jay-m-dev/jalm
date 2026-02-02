use jalm_parser::parse;
use jalm_syntax::{SyntaxElement, SyntaxKind, SyntaxNode};
use serde::Serialize;
use std::collections::HashSet;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Diagnostic {
    pub code: String,
    pub message: String,
    pub span: Span,
    pub required: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Span {
    pub start: usize,
    pub end: usize,
}

#[derive(Debug, Clone, Serialize)]
pub struct CheckResult {
    pub diagnostics: Vec<Diagnostic>,
}

pub fn check(source: &str) -> CheckResult {
    let parsed = parse(source);
    let root = parsed.syntax();
    let mut diagnostics = Vec::new();
    for item in root.children() {
        if item.kind() == SyntaxKind::FnDecl {
            check_fn(&item, &mut diagnostics);
        }
    }
    CheckResult { diagnostics }
}

fn check_fn(node: &SyntaxNode, diagnostics: &mut Vec<Diagnostic>) {
    let declared = declared_effects(node);
    if let Some(block) = node.children().find(|n| n.kind() == SyntaxKind::Block) {
        for (effect, span) in effects_used_in(&block) {
            if !declared.contains(effect) {
                diagnostics.push(Diagnostic {
                    code: "E1001".to_string(),
                    message: "undeclared effect".to_string(),
                    span,
                    required: effect.to_string(),
                });
            }
        }
    }
}

fn declared_effects(node: &SyntaxNode) -> HashSet<String> {
    let mut effects = HashSet::new();
    if let Some(effect_set) = node.children().find(|n| n.kind() == SyntaxKind::EffectSet) {
        for ident in effect_set.children().filter(|n| n.kind() == SyntaxKind::IdentNode) {
            if let Some(name) = find_ident_text(&ident) {
                match name.as_str() {
                    "io" | "net" | "fs" | "time" | "rand" | "ffi" => {
                        effects.insert(name);
                    }
                    _ => {}
                }
            }
        }
    }
    effects
}

fn effects_used_in(node: &SyntaxNode) -> Vec<(&'static str, Span)> {
    let mut effects = Vec::new();
    let text = node.text().to_string();
    let base: usize = node.text_range().start().into();
    for (prefix, effect) in [
        ("fs::", "fs"),
        ("net::", "net"),
        ("http::", "net"),
        ("time::", "time"),
        ("rand::", "rand"),
        ("log::", "io"),
        ("ffi::", "ffi"),
    ] {
        let mut offset = 0;
        while let Some(pos) = text[offset..].find(prefix) {
            let start = base + offset + pos;
            let end = start + prefix.len();
            effects.push((effect, Span { start, end }));
            offset = offset + pos + prefix.len();
        }
    }
    effects
}

fn find_ident_text(node: &SyntaxNode) -> Option<String> {
    node.children_with_tokens().find_map(|e| match e {
        SyntaxElement::Token(t) if t.kind() == SyntaxKind::Ident => Some(t.text().to_string()),
        _ => None,
    })
}
