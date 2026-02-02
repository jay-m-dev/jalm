use jalm_parser::parse;
use jalm_syntax::{SyntaxElement, SyntaxKind, SyntaxNode};
use rowan::TextRange;
use serde::Serialize;
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Diagnostic {
    pub code: String,
    pub message: String,
    pub span: Span,
    pub expected: Option<String>,
    pub actual: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Span {
    pub start: usize,
    pub end: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum Type {
    I64,
    I32,
    F64,
    Bool,
    String,
    Bytes,
    Unit,
    Named(String),
    Unknown,
    Error,
}

impl Type {
    fn name(&self) -> String {
        match self {
            Type::I64 => "i64".to_string(),
            Type::I32 => "i32".to_string(),
            Type::F64 => "f64".to_string(),
            Type::Bool => "bool".to_string(),
            Type::String => "string".to_string(),
            Type::Bytes => "bytes".to_string(),
            Type::Unit => "()".to_string(),
            Type::Named(name) => name.clone(),
            Type::Unknown => "<unknown>".to_string(),
            Type::Error => "<error>".to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct CheckResult {
    pub diagnostics: Vec<Diagnostic>,
}

pub fn check(source: &str) -> CheckResult {
    let parsed = parse(source);
    let root = parsed.syntax();
    let mut checker = Checker::new();
    checker.check_root(&root);
    CheckResult {
        diagnostics: checker.diagnostics,
    }
}

struct Checker {
    scopes: Vec<HashMap<String, Type>>,
    current_return: Type,
    diagnostics: Vec<Diagnostic>,
}

impl Checker {
    fn new() -> Self {
        Self {
            scopes: vec![HashMap::new()],
            current_return: Type::Unit,
            diagnostics: Vec::new(),
        }
    }

    fn check_root(&mut self, node: &SyntaxNode) {
        for item in node.children() {
            if item.kind() == SyntaxKind::FnDecl {
                self.check_fn(&item);
            }
        }
    }

    fn check_fn(&mut self, node: &SyntaxNode) {
        let ret = find_return_type(node).unwrap_or(Type::Unit);
        let saved_return = self.current_return.clone();
        self.current_return = ret;
        self.enter_scope();
        if let Some(params) = node.children().find(|n| n.kind() == SyntaxKind::ParamList) {
            for param in params.children().filter(|n| n.kind() == SyntaxKind::Param) {
                if let (Some(name), Some(ty)) = (find_ident_in(&param), find_type_in(&param)) {
                    self.insert_var(&name, ty);
                }
            }
        }
        if let Some(block) = node.children().find(|n| n.kind() == SyntaxKind::Block) {
            let body_ty = self.check_block(&block);
            let expected = self.current_return.clone();
            if body_ty != Type::Error && !type_compatible(&expected, &body_ty) {
                self.type_mismatch(&block, &expected, &body_ty, "E0004");
            }
        }
        self.exit_scope();
        self.current_return = saved_return;
    }

    fn check_block(&mut self, node: &SyntaxNode) -> Type {
        let mut last = Type::Unit;
        if let Some(stmts) = node.children().find(|n| n.kind() == SyntaxKind::StmtList) {
            let items: Vec<_> = stmts.children().collect();
            let len = items.len();
            for (idx, stmt) in items.into_iter().enumerate() {
                if idx + 1 == len && is_expr_kind(stmt.kind()) && stmt.kind() != SyntaxKind::ExprStmt {
                    last = self.check_expr(&stmt);
                } else {
                    self.check_stmt(&stmt);
                }
            }
        }
        last
    }

    fn check_stmt(&mut self, node: &SyntaxNode) {
        match node.kind() {
            SyntaxKind::LetStmt => self.check_let(node),
            SyntaxKind::ReturnStmt => self.check_return(node),
            SyntaxKind::ExprStmt => {
                if let Some(expr) = node.children().find(|n| is_expr_kind(n.kind())) {
                    self.check_expr(&expr);
                }
            }
            _ => {
                if is_expr_kind(node.kind()) {
                    self.check_expr(node);
                }
            }
        }
    }

    fn check_let(&mut self, node: &SyntaxNode) {
        let name = node
            .children()
            .find(|n| n.kind() == SyntaxKind::Pattern)
            .and_then(|n| find_ident_in(&n));
        let ty_annot = node
            .children()
            .find(|n| n.kind() == SyntaxKind::Type)
            .map(|n| type_from_node(&n));
        let expr = find_expr_after_token(node, SyntaxKind::Eq);
        let expr_ty = expr.map(|e| self.check_expr(&e)).unwrap_or(Type::Unknown);
        if let Some(name) = name {
            if let Some(annot) = ty_annot.clone() {
                if !type_compatible(&annot, &expr_ty) {
                    self.type_mismatch(node, &annot, &expr_ty, "E0003");
                }
                self.insert_var(&name, annot);
            } else {
                self.insert_var(&name, expr_ty);
            }
        }
    }

    fn check_return(&mut self, node: &SyntaxNode) {
        let expr = node.children().find(|n| is_expr_kind(n.kind()));
        let expr_ty = expr.map(|e| self.check_expr(&e)).unwrap_or(Type::Unit);
        let expected = self.current_return.clone();
        if !type_compatible(&expected, &expr_ty) {
            self.type_mismatch(node, &expected, &expr_ty, "E0004");
        }
    }

    fn check_expr(&mut self, node: &SyntaxNode) -> Type {
        match node.kind() {
            SyntaxKind::IdentNode => {
                if let Some(name) = find_ident_in(node) {
                    self.lookup_var(&name).unwrap_or_else(|| {
                        self.report(node, "E0001", "undefined variable", None, Some(name));
                        Type::Error
                    })
                } else {
                    Type::Unknown
                }
            }
            SyntaxKind::LiteralNode => literal_type(node),
            SyntaxKind::BinExpr => self.check_bin_expr(node),
            SyntaxKind::CallExpr => Type::Unknown,
            SyntaxKind::MemberExpr => Type::Unknown,
            SyntaxKind::IfExpr => self.check_if_expr(node),
            SyntaxKind::MatchExpr => self.check_match_expr(node),
            SyntaxKind::Block => self.check_block(node),
            SyntaxKind::ParenExpr => node.children().find(|n| is_expr_kind(n.kind())).map(|e| self.check_expr(&e)).unwrap_or(Type::Unknown),
            _ => Type::Unknown,
        }
    }

    fn check_if_expr(&mut self, node: &SyntaxNode) -> Type {
        let mut kids = node.children();
        let cond = kids.next();
        let then_block = kids.next();
        let else_block = kids.next();
        if let Some(cond) = cond {
            let cond_ty = self.check_expr(&cond);
            if cond_ty != Type::Bool && cond_ty != Type::Error {
                self.type_mismatch(&cond, &Type::Bool, &cond_ty, "E0005");
            }
        }
        let then_ty = then_block.map(|b| self.check_expr(&b)).unwrap_or(Type::Unit);
        let else_ty = else_block.map(|b| self.check_expr(&b)).unwrap_or(Type::Unit);
        if !type_compatible(&then_ty, &else_ty) {
            self.type_mismatch(node, &then_ty, &else_ty, "E0006");
            Type::Error
        } else {
            then_ty
        }
    }

    fn check_match_expr(&mut self, node: &SyntaxNode) -> Type {
        let mut kids = node.children();
        let _scrutinee = kids.next().map(|e| self.check_expr(&e));
        let mut arm_type: Option<Type> = None;
        for arm in kids.filter(|n| n.kind() == SyntaxKind::MatchArm) {
            if let Some(expr) = arm.children().find(|n| is_expr_kind(n.kind())) {
                let ty = self.check_expr(&expr);
                if let Some(existing) = &arm_type {
                    if !type_compatible(existing, &ty) {
                        self.type_mismatch(&arm, existing, &ty, "E0007");
                        return Type::Error;
                    }
                } else {
                    arm_type = Some(ty);
                }
            }
        }
        arm_type.unwrap_or(Type::Unit)
    }

    fn check_bin_expr(&mut self, node: &SyntaxNode) -> Type {
        let (op_kind, left, right) = match bin_parts(node) {
            Some(parts) => parts,
            None => return Type::Unknown,
        };
        let l = self.check_expr(&left);
        let r = self.check_expr(&right);
        if l == Type::Error || r == Type::Error {
            return Type::Error;
        }
        match op_kind {
            SyntaxKind::Plus | SyntaxKind::Minus | SyntaxKind::Star | SyntaxKind::Slash | SyntaxKind::Percent => {
                if is_numeric(&l) && type_compatible(&l, &r) {
                    l
                } else {
                    self.type_mismatch(node, &l, &r, "E0003");
                    Type::Error
                }
            }
            SyntaxKind::EqEq | SyntaxKind::Neq => {
                if type_compatible(&l, &r) {
                    Type::Bool
                } else {
                    self.type_mismatch(node, &l, &r, "E0003");
                    Type::Error
                }
            }
            SyntaxKind::Lt | SyntaxKind::Lte | SyntaxKind::Gt | SyntaxKind::Gte => {
                if is_numeric(&l) && type_compatible(&l, &r) {
                    Type::Bool
                } else {
                    self.type_mismatch(node, &l, &r, "E0003");
                    Type::Error
                }
            }
            SyntaxKind::AndAnd | SyntaxKind::OrOr => {
                if l == Type::Bool && r == Type::Bool {
                    Type::Bool
                } else {
                    self.type_mismatch(node, &Type::Bool, &l, "E0003");
                    Type::Error
                }
            }
            _ => Type::Unknown,
        }
    }

    fn report(&mut self, node: &SyntaxNode, code: &str, message: &str, expected: Option<String>, actual: Option<String>) {
        let span = span_of(node.text_range());
        self.diagnostics.push(Diagnostic {
            code: code.to_string(),
            message: message.to_string(),
            span,
            expected,
            actual,
        });
    }

    fn type_mismatch(&mut self, node: &SyntaxNode, expected: &Type, actual: &Type, code: &str) {
        self.report(node, code, "type mismatch", Some(expected.name()), Some(actual.name()));
    }

    fn enter_scope(&mut self) {
        self.scopes.push(HashMap::new());
    }

    fn exit_scope(&mut self) {
        self.scopes.pop();
    }

    fn insert_var(&mut self, name: &str, ty: Type) {
        if let Some(scope) = self.scopes.last_mut() {
            scope.insert(name.to_string(), ty);
        }
    }

    fn lookup_var(&self, name: &str) -> Option<Type> {
        for scope in self.scopes.iter().rev() {
            if let Some(ty) = scope.get(name) {
                return Some(ty.clone());
            }
        }
        None
    }
}

fn find_return_type(node: &SyntaxNode) -> Option<Type> {
    let mut seen_arrow = false;
    for el in node.children_with_tokens() {
        match el {
            SyntaxElement::Token(t) if t.kind() == SyntaxKind::Arrow => {
                seen_arrow = true;
            }
            SyntaxElement::Node(n) if seen_arrow && n.kind() == SyntaxKind::Type => {
                return Some(type_from_node(&n));
            }
            _ => {}
        }
    }
    None
}

fn find_ident_in(node: &SyntaxNode) -> Option<String> {
    if let Some(name) = node.children_with_tokens().find_map(|e| match e {
        SyntaxElement::Token(t) if t.kind() == SyntaxKind::Ident => Some(t.text().to_string()),
        _ => None,
    }) {
        return Some(name);
    }
    for child in node.children() {
        if let Some(name) = find_ident_in(&child) {
            return Some(name);
        }
    }
    None
}

fn find_type_in(node: &SyntaxNode) -> Option<Type> {
    node.children()
        .find(|n| n.kind() == SyntaxKind::Type)
        .map(|n| type_from_node(&n))
}

fn type_from_node(node: &SyntaxNode) -> Type {
    let text = node.text().to_string();
    match text.trim() {
        "i64" => Type::I64,
        "i32" => Type::I32,
        "f64" => Type::F64,
        "bool" => Type::Bool,
        "string" => Type::String,
        "bytes" => Type::Bytes,
        "()" => Type::Unit,
        other => Type::Named(other.to_string()),
    }
}

fn literal_type(node: &SyntaxNode) -> Type {
    for el in node.children_with_tokens() {
        if let SyntaxElement::Token(t) = el {
            return match t.kind() {
                SyntaxKind::Int => Type::I64,
                SyntaxKind::Float => Type::F64,
                SyntaxKind::String => Type::String,
                SyntaxKind::Bytes => Type::Bytes,
                SyntaxKind::KwTrue | SyntaxKind::KwFalse => Type::Bool,
                _ => Type::Unknown,
            };
        }
    }
    Type::Unknown
}

fn is_numeric(ty: &Type) -> bool {
    matches!(ty, Type::I64 | Type::I32 | Type::F64)
}

fn type_compatible(a: &Type, b: &Type) -> bool {
    match (a, b) {
        (Type::Unknown, _) | (_, Type::Unknown) => true,
        _ => a == b,
    }
}

fn bin_parts(node: &SyntaxNode) -> Option<(SyntaxKind, SyntaxNode, SyntaxNode)> {
    let mut children = node.children();
    let left = children.next()?;
    let right = children.nth(0)?;
    let mut op_kind = None;
    for el in node.children_with_tokens() {
        if let SyntaxElement::Token(t) = el {
            if matches!(t.kind(),
                SyntaxKind::Plus | SyntaxKind::Minus | SyntaxKind::Star | SyntaxKind::Slash | SyntaxKind::Percent |
                SyntaxKind::EqEq | SyntaxKind::Neq | SyntaxKind::Lt | SyntaxKind::Lte | SyntaxKind::Gt | SyntaxKind::Gte |
                SyntaxKind::AndAnd | SyntaxKind::OrOr
            ) {
                op_kind = Some(t.kind());
                break;
            }
        }
    }
    Some((op_kind?, left, right))
}

fn find_expr_after_token(node: &SyntaxNode, token_kind: SyntaxKind) -> Option<SyntaxNode> {
    let mut seen = false;
    for el in node.children_with_tokens() {
        match el {
            SyntaxElement::Token(t) if t.kind() == token_kind => seen = true,
            SyntaxElement::Node(n) if seen && is_expr_kind(n.kind()) => return Some(n),
            _ => {}
        }
    }
    None
}

fn is_expr_kind(kind: SyntaxKind) -> bool {
    matches!(
        kind,
        SyntaxKind::BinExpr
            | SyntaxKind::CallExpr
            | SyntaxKind::MemberExpr
            | SyntaxKind::IfExpr
            | SyntaxKind::MatchExpr
            | SyntaxKind::IdentNode
            | SyntaxKind::LiteralNode
            | SyntaxKind::ParenExpr
            | SyntaxKind::Block
    )
}

fn span_of(range: TextRange) -> Span {
    Span {
        start: range.start().into(),
        end: range.end().into(),
    }
}
