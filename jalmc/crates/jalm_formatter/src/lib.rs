use jalm_parser::{parse, ParseError};
use jalm_syntax::{SyntaxElement, SyntaxKind, SyntaxNode};

#[derive(Debug)]
pub enum FormatError {
    ParseErrors(Vec<ParseError>),
}

pub fn format_source(source: &str) -> Result<String, FormatError> {
    let parsed = parse(source);
    if !parsed.errors.is_empty() {
        return Err(FormatError::ParseErrors(parsed.errors));
    }
    let root = parsed.syntax();
    let mut fmt = Formatter::new();
    fmt.root(&root);
    Ok(fmt.finish())
}

struct Formatter {
    out: String,
    indent: usize,
}

impl Formatter {
    fn new() -> Self {
        Self { out: String::new(), indent: 0 }
    }

    fn finish(self) -> String {
        self.out
    }

    fn push(&mut self, s: &str) {
        self.out.push_str(s);
    }

    fn newline(&mut self) {
        self.out.push('\n');
        for _ in 0..self.indent {
            self.out.push_str("  ");
        }
    }

    fn root(&mut self, node: &SyntaxNode) {
        let mut first = true;
        for child in node.children() {
            match child.kind() {
                SyntaxKind::ModuleDecl
                | SyntaxKind::UseDecl
                | SyntaxKind::FnDecl
                | SyntaxKind::StructDecl
                | SyntaxKind::EnumDecl => {
                    if !first {
                        self.newline();
                        self.newline();
                    }
                    self.item(&child);
                    first = false;
                }
                _ => {}
            }
        }
    }

    fn item(&mut self, node: &SyntaxNode) {
        match node.kind() {
            SyntaxKind::ModuleDecl => self.module_decl(node),
            SyntaxKind::UseDecl => self.use_decl(node),
            SyntaxKind::FnDecl => self.fn_decl(node),
            SyntaxKind::StructDecl => self.struct_decl(node),
            SyntaxKind::EnumDecl => self.enum_decl(node),
            _ => {}
        }
    }

    fn module_decl(&mut self, node: &SyntaxNode) {
        self.push("mod ");
        if let Some(name) = node
            .children()
            .find(|n| n.kind() == SyntaxKind::IdentNode)
            .and_then(|n| first_ident_child_text(&n))
        {
            self.push(&name);
        }
        self.push(";");
    }

    fn use_decl(&mut self, node: &SyntaxNode) {
        self.push("use ");
        if let Some(path) = format_use_path(node) {
            self.push(&path);
        }
        if let Some(alias) = find_kw_as_alias(node) {
            self.push(" as ");
            self.push(&alias);
        }
        self.push(";");
    }

    fn fn_decl(&mut self, node: &SyntaxNode) {
        let mut tokens = node.children_with_tokens();
        let has_pub = tokens.clone().any(|e| matches!(e, SyntaxElement::Token(t) if t.kind() == SyntaxKind::KwPub));
        let has_async = tokens.any(|e| matches!(e, SyntaxElement::Token(t) if t.kind() == SyntaxKind::KwAsync));
        if has_pub {
            self.push("pub ");
        }
        if has_async {
            self.push("async ");
        }
        self.push("fn ");
        if let Some(name) = node
            .children()
            .find(|n| n.kind() == SyntaxKind::IdentNode)
            .and_then(|n| first_ident_child_text(&n))
        {
            self.push(&name);
        }
        if let Some(params) = node.children().find(|n| n.kind() == SyntaxKind::ParamList) {
            self.push("(");
            self.param_list(&params);
            self.push(")");
        } else {
            self.push("()");
        }
        if let Some(ret) = find_return_type(node) {
            self.push(" -> ");
            self.type_node(&ret);
        }
        if let Some(effects) = node.children().find(|n| n.kind() == SyntaxKind::EffectSet) {
            self.push(" ");
            self.effect_set(&effects);
        }
        if let Some(block) = node.children().find(|n| n.kind() == SyntaxKind::Block) {
            self.push(" ");
            self.block(&block);
        }
    }

    fn param_list(&mut self, node: &SyntaxNode) {
        let mut first = true;
        for param in node.children().filter(|n| n.kind() == SyntaxKind::Param) {
            if !first {
                self.push(", ");
            }
            self.param(&param);
            first = false;
        }
    }

    fn param(&mut self, node: &SyntaxNode) {
        if node.children_with_tokens().any(|e| matches!(e, SyntaxElement::Token(t) if t.kind() == SyntaxKind::KwMut)) {
            self.push("mut ");
        }
        if let Some(name) = node
            .children()
            .find(|n| n.kind() == SyntaxKind::IdentNode)
            .and_then(|n| first_ident_child_text(&n))
        {
            self.push(&name);
        }
        if let Some(ty) = node.children().find(|n| n.kind() == SyntaxKind::Type) {
            self.push(": ");
            self.type_node(&ty);
        }
    }

    fn type_node(&mut self, node: &SyntaxNode) {
        let text = node.text().to_string();
        self.push(text.trim());
    }

    fn effect_set(&mut self, node: &SyntaxNode) {
        self.push("!{");
        let mut first = true;
        for ident in node.children().filter(|n| n.kind() == SyntaxKind::IdentNode) {
            if !first {
                self.push(", ");
            }
            if let Some(name) = first_ident_child_text(&ident) {
                self.push(&name);
            }
            first = false;
        }
        self.push("}");
    }

    fn struct_decl(&mut self, node: &SyntaxNode) {
        let has_pub = node.children_with_tokens().any(|e| matches!(e, SyntaxElement::Token(t) if t.kind() == SyntaxKind::KwPub));
        if has_pub {
            self.push("pub ");
        }
        self.push("struct ");
        if let Some(name) = node
            .children()
            .find(|n| n.kind() == SyntaxKind::IdentNode)
            .and_then(|n| first_ident_child_text(&n))
        {
            self.push(&name);
        }
        self.push(" {");
        self.indent += 1;
        for field in node.children().filter(|n| n.kind() == SyntaxKind::StructField) {
            self.newline();
            if let Some(fname) = field
                .children()
                .find(|n| n.kind() == SyntaxKind::IdentNode)
                .and_then(|n| first_ident_child_text(&n))
            {
                self.push(&fname);
            }
            if let Some(ty) = field.children().find(|n| n.kind() == SyntaxKind::Type) {
                self.push(": ");
                self.type_node(&ty);
            }
            self.push(";");
        }
        self.indent -= 1;
        self.newline();
        self.push("}");
    }

    fn enum_decl(&mut self, node: &SyntaxNode) {
        let has_pub = node.children_with_tokens().any(|e| matches!(e, SyntaxElement::Token(t) if t.kind() == SyntaxKind::KwPub));
        if has_pub {
            self.push("pub ");
        }
        self.push("enum ");
        if let Some(name) = node
            .children()
            .find(|n| n.kind() == SyntaxKind::IdentNode)
            .and_then(|n| first_ident_child_text(&n))
        {
            self.push(&name);
        }
        self.push(" {");
        self.indent += 1;
        for variant in node.children().filter(|n| n.kind() == SyntaxKind::EnumVariant) {
            self.newline();
            if let Some(vname) = variant
                .children()
                .find(|n| n.kind() == SyntaxKind::IdentNode)
                .and_then(|n| first_ident_child_text(&n))
            {
                self.push(&vname);
            }
            let types: Vec<_> = variant.children().filter(|n| n.kind() == SyntaxKind::Type).collect();
            if !types.is_empty() {
                self.push("(");
                let mut first = true;
                for ty in types {
                    if !first {
                        self.push(", ");
                    }
                    self.type_node(&ty);
                    first = false;
                }
                self.push(")");
            }
            self.push(";");
        }
        self.indent -= 1;
        self.newline();
        self.push("}");
    }

    fn block(&mut self, node: &SyntaxNode) {
        self.push("{");
        self.indent += 1;
        let mut any_stmt = false;
        if let Some(stmts) = node.children().find(|n| n.kind() == SyntaxKind::StmtList) {
            let items: Vec<_> = stmts.children().collect();
            let len = items.len();
            for (idx, stmt) in items.into_iter().enumerate() {
                if matches!(
                    stmt.kind(),
                    SyntaxKind::LetStmt
                        | SyntaxKind::ReturnStmt
                        | SyntaxKind::ExprStmt
                        | SyntaxKind::IfExpr
                        | SyntaxKind::MatchExpr
                        | SyntaxKind::Block
                        | SyntaxKind::BinExpr
                        | SyntaxKind::CallExpr
                        | SyntaxKind::MemberExpr
                        | SyntaxKind::IdentNode
                        | SyntaxKind::LiteralNode
                        | SyntaxKind::ParenExpr
                        | SyntaxKind::Error
                ) {
                    self.newline();
                    if idx == len - 1 && is_expr_kind(stmt.kind()) && stmt.kind() != SyntaxKind::ExprStmt {
                        self.expr(&stmt, 0);
                    } else {
                        self.stmt(&stmt);
                    }
                    any_stmt = true;
                }
            }
        }
        self.indent -= 1;
        if any_stmt {
            self.newline();
        }
        self.push("}");
    }

    fn stmt(&mut self, node: &SyntaxNode) {
        match node.kind() {
            SyntaxKind::LetStmt => self.let_stmt(node),
            SyntaxKind::ReturnStmt => self.return_stmt(node),
            SyntaxKind::ExprStmt => self.expr_stmt(node),
            SyntaxKind::IfExpr => self.if_expr(node),
            SyntaxKind::MatchExpr => self.match_expr(node),
            SyntaxKind::Block => self.block(node),
            _ => self.expr(node, 0),
        }
    }

    fn let_stmt(&mut self, node: &SyntaxNode) {
        self.push("let ");
        if node.children_with_tokens().any(|e| matches!(e, SyntaxElement::Token(t) if t.kind() == SyntaxKind::KwMut)) {
            self.push("mut ");
        }
        if let Some(pattern) = node.children().find(|n| n.kind() == SyntaxKind::Pattern) {
            self.pattern(&pattern);
        }
        if let Some(ty) = node.children().find(|n| n.kind() == SyntaxKind::Type) {
            self.push(": ");
            self.type_node(&ty);
        }
        self.push(" = ");
        if let Some(expr) = find_expr_after_token(node, SyntaxKind::Eq) {
            self.expr(&expr, 0);
        }
        self.push(";");
    }

    fn return_stmt(&mut self, node: &SyntaxNode) {
        self.push("return");
        if let Some(expr) = node.children().find(|n| is_expr_kind(n.kind())) {
            self.push(" ");
            self.expr(&expr, 0);
        }
        self.push(";");
    }

    fn expr_stmt(&mut self, node: &SyntaxNode) {
        if let Some(expr) = node.children().find(|n| is_expr_kind(n.kind())) {
            self.expr(&expr, 0);
        }
        self.push(";");
    }

    fn if_expr(&mut self, node: &SyntaxNode) {
        self.push("if ");
        let mut kids = node.children();
        if let Some(cond) = kids.next() {
            self.expr(&cond, 0);
        }
        if let Some(then_block) = kids.next() {
            self.push(" ");
            self.block(&then_block);
        }
        if let Some(else_node) = kids.next() {
            self.push(" else ");
            if else_node.kind() == SyntaxKind::IfExpr {
                self.if_expr(&else_node);
            } else {
                self.block(&else_node);
            }
        }
    }

    fn match_expr(&mut self, node: &SyntaxNode) {
        self.push("match ");
        let mut kids = node.children();
        if let Some(scrutinee) = kids.next() {
            self.expr(&scrutinee, 0);
        }
        self.push(" {");
        self.indent += 1;
        for arm in kids.filter(|n| n.kind() == SyntaxKind::MatchArm) {
            self.newline();
            self.match_arm(&arm);
        }
        self.indent -= 1;
        self.newline();
        self.push("}");
    }

    fn match_arm(&mut self, node: &SyntaxNode) {
        if let Some(pat) = node.children().find(|n| n.kind() == SyntaxKind::Pattern) {
            self.pattern(&pat);
        }
        self.push(" => ");
        if let Some(expr) = node.children().find(|n| n.kind() != SyntaxKind::Pattern) {
            self.expr(&expr, 0);
        }
        self.push(",");
    }

    fn pattern(&mut self, node: &SyntaxNode) {
        if let Some(token) = node.children_with_tokens().find_map(|e| match e {
            SyntaxElement::Token(t) if t.kind() == SyntaxKind::Underscore => Some(t.text().to_string()),
            _ => None,
        }) {
            self.push(&token);
            return;
        }
        if let Some(lit) = node.children().find(|n| n.kind() == SyntaxKind::LiteralNode) {
            if let Some(text) = literal_text(&lit) {
                self.push(&text);
                return;
            }
        }
        if let Some(ident) = node.children().find(|n| n.kind() == SyntaxKind::IdentNode) {
            if let Some(name) = first_ident_child_text(&ident) {
                self.push(&name);
                return;
            }
        }
    }

    fn expr(&mut self, node: &SyntaxNode, min_bp: u8) {
        match node.kind() {
            SyntaxKind::BinExpr => self.bin_expr(node, min_bp),
            SyntaxKind::CallExpr => self.call_expr(node),
            SyntaxKind::MemberExpr => self.member_expr(node),
            SyntaxKind::IfExpr => self.if_expr(node),
            SyntaxKind::MatchExpr => self.match_expr(node),
            SyntaxKind::Block => self.block(node),
            SyntaxKind::ParenExpr => self.paren_expr(node),
            SyntaxKind::IdentNode => {
                if let Some(name) = first_ident_child_text(node) {
                    self.push(&name);
                }
            }
            SyntaxKind::LiteralNode => {
                if let Some(lit) = literal_text(node) {
                    self.push(&lit);
                }
            }
            _ => {}
        }
    }

    fn bin_expr(&mut self, node: &SyntaxNode, min_bp: u8) {
        let (op_kind, op_text, left, right) = match bin_parts(node) {
            Some(parts) => parts,
            None => return,
        };
        let (l_bp, r_bp) = infix_binding_power(op_kind);
        let needs_paren = l_bp < min_bp;
        if needs_paren {
            self.push("(");
        }
        self.expr(&left, l_bp);
        self.push(" ");
        self.push(&op_text);
        self.push(" ");
        self.expr(&right, r_bp);
        if needs_paren {
            self.push(")");
        }
    }

    fn call_expr(&mut self, node: &SyntaxNode) {
        let mut kids = node.children();
        if let Some(callee) = kids.next() {
            self.expr(&callee, 0);
        }
        self.push("(");
        let mut first = true;
        for arg in kids {
            if !first {
                self.push(", ");
            }
            self.expr(&arg, 0);
            first = false;
        }
        self.push(")");
    }

    fn member_expr(&mut self, node: &SyntaxNode) {
        let mut kids = node.children();
        if let Some(base) = kids.next() {
            self.expr(&base, 0);
        }
        if let Some(field) = kids.next() {
            self.push(".");
            if let Some(name) = first_ident_child_text(&field) {
                self.push(&name);
            }
        }
    }

    fn paren_expr(&mut self, node: &SyntaxNode) {
        self.push("(");
        if let Some(inner) = node.children().next() {
            self.expr(&inner, 0);
        }
        self.push(")");
    }
}

fn first_ident_child_text(node: &SyntaxNode) -> Option<String> {
    node.children_with_tokens().find_map(|e| match e {
        SyntaxElement::Token(t) if t.kind() == SyntaxKind::Ident => Some(t.text().to_string()),
        _ => None,
    })
}

fn literal_text(node: &SyntaxNode) -> Option<String> {
    node.children_with_tokens().find_map(|e| match e {
        SyntaxElement::Token(t) if t.kind().is_literal() => Some(t.text().to_string()),
        _ => None,
    })
}

fn find_kw_as_alias(node: &SyntaxNode) -> Option<String> {
    let mut seen_as = false;
    for el in node.children_with_tokens() {
        match el {
            SyntaxElement::Token(t) if t.kind() == SyntaxKind::KwAs => seen_as = true,
            SyntaxElement::Token(t) if seen_as && t.kind() == SyntaxKind::Ident => return Some(t.text().to_string()),
            _ => {}
        }
    }
    None
}

fn format_use_path(node: &SyntaxNode) -> Option<String> {
    let mut parts = Vec::new();
    for child in node.children() {
        if child.kind() == SyntaxKind::UsePath {
            for el in child.children_with_tokens() {
                match el {
                    SyntaxElement::Token(t) if t.kind() == SyntaxKind::Ident => parts.push(t.text().to_string()),
                    SyntaxElement::Token(t) if t.kind() == SyntaxKind::ColonColon => parts.push("::".to_string()),
                    _ => {}
                }
            }
            break;
        }
    }
    if parts.is_empty() {
        None
    } else {
        Some(parts.concat())
    }
}

fn find_return_type(node: &SyntaxNode) -> Option<SyntaxNode> {
    let mut seen_arrow = false;
    for el in node.children_with_tokens() {
        if let SyntaxElement::Token(t) = &el {
            if t.kind() == SyntaxKind::Arrow {
                seen_arrow = true;
                continue;
            }
        }
        if seen_arrow {
            if let SyntaxElement::Node(n) = el {
                if n.kind() == SyntaxKind::Type {
                    return Some(n);
                }
            }
        }
    }
    None
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

fn bin_parts(node: &SyntaxNode) -> Option<(SyntaxKind, String, SyntaxNode, SyntaxNode)> {
    let mut children = node.children();
    let left = children.next()?;
    let right = children.nth(0)?;
    let mut op_kind = None;
    let mut op_text = None;
    for el in node.children_with_tokens() {
        if let SyntaxElement::Token(t) = el {
            if matches!(t.kind(),
                SyntaxKind::Plus | SyntaxKind::Minus | SyntaxKind::Star | SyntaxKind::Slash | SyntaxKind::Percent |
                SyntaxKind::EqEq | SyntaxKind::Neq | SyntaxKind::Lt | SyntaxKind::Lte | SyntaxKind::Gt | SyntaxKind::Gte |
                SyntaxKind::AndAnd | SyntaxKind::OrOr
            ) {
                op_kind = Some(t.kind());
                op_text = Some(t.text().to_string());
                break;
            }
        }
    }
    Some((op_kind?, op_text?, left, right))
}

fn infix_binding_power(kind: SyntaxKind) -> (u8, u8) {
    match kind {
        SyntaxKind::OrOr => (1, 2),
        SyntaxKind::AndAnd => (3, 4),
        SyntaxKind::EqEq | SyntaxKind::Neq => (5, 6),
        SyntaxKind::Lt | SyntaxKind::Lte | SyntaxKind::Gt | SyntaxKind::Gte => (7, 8),
        SyntaxKind::Plus | SyntaxKind::Minus => (9, 10),
        SyntaxKind::Star | SyntaxKind::Slash | SyntaxKind::Percent => (11, 12),
        _ => (0, 0),
    }
}
