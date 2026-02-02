use jalm_parser::parse;
use jalm_syntax::{SyntaxElement, SyntaxKind, SyntaxNode};
use wasm_encoder::{CodeSection, ExportKind, ExportSection, Function, FunctionSection, Instruction, Module, TypeSection, ValType};

#[derive(Debug, Clone)]
pub struct Diagnostic {
    pub code: String,
    pub message: String,
}

pub fn compile_to_wasm(source: &str) -> Result<Vec<u8>, Vec<Diagnostic>> {
    let parsed = parse(source);
    if !parsed.errors.is_empty() {
        return Err(parsed
            .errors
            .into_iter()
            .map(|e| Diagnostic { code: "E2000".to_string(), message: e.message })
            .collect());
    }
    let root = parsed.syntax();
    let functions = collect_functions(&root);
    let mut diags = Vec::new();
    if functions.is_empty() {
        diags.push(Diagnostic { code: "E2001".to_string(), message: "no functions found".to_string() });
        return Err(diags);
    }

    let mut types = TypeSection::new();
    let mut funcs = FunctionSection::new();
    let mut code = CodeSection::new();
    let mut exports = ExportSection::new();

    let mut func_indices = std::collections::HashMap::new();

    for (idx, f) in functions.iter().enumerate() {
        func_indices.insert(f.name.clone(), idx as u32);
    }

    for f in &functions {
        let (params, result) = signature_from_fn(f, &mut diags);
        let type_index = types.len();
        types.function(params.clone(), result.clone());
        funcs.function(type_index);

        let mut locals = Vec::new();
        for (_, ty) in &f.locals {
            locals.push((1, *ty));
        }
        let mut body = Function::new(locals);
        let mut ctx = EmitCtx { func_indices: &func_indices, locals: &f.locals, params: &f.params, diagnostics: &mut diags };
        for stmt in &f.body {
            emit_stmt(&mut body, &mut ctx, stmt);
        }
        if !matches!(f.ret, Some(ValType::I64)) {
            // default return 0 for now
            body.instruction(&Instruction::I64Const(0));
        }
        body.instruction(&Instruction::End);
        code.function(&body);

        if f.name == "main" {
            exports.export("main", ExportKind::Func, func_indices["main"]);
        }
    }

    if diags.is_empty() {
        let mut module = Module::new();
        module.section(&types);
        module.section(&funcs);
        module.section(&exports);
        module.section(&code);
        Ok(module.finish())
    } else {
        Err(diags)
    }
}

#[derive(Debug, Clone)]
struct FnDef {
    name: String,
    params: Vec<(String, ValType)>,
    locals: Vec<(String, ValType)>,
    body: Vec<Stmt>,
    ret: Option<ValType>,
}

#[derive(Debug, Clone)]
enum Stmt {
    Let { name: String, expr: Expr },
    Return(Expr),
    Expr(Expr),
    If { cond: Expr, then_body: Vec<Stmt>, else_body: Vec<Stmt> },
}

#[derive(Debug, Clone)]
enum Expr {
    Int(i64),
    Bool(bool),
    Ident(String),
    Bin { op: SyntaxKind, lhs: Box<Expr>, rhs: Box<Expr> },
    Call { name: String, args: Vec<Expr> },
}

fn collect_functions(root: &SyntaxNode) -> Vec<FnDef> {
    let mut out = Vec::new();
    for node in root.children().filter(|n| n.kind() == SyntaxKind::FnDecl) {
        if let Some(f) = lower_fn(&node) {
            out.push(f);
        }
    }
    out
}

fn lower_fn(node: &SyntaxNode) -> Option<FnDef> {
    let name = node
        .children()
        .find(|n| n.kind() == SyntaxKind::IdentNode)
        .and_then(|n| find_ident_text(n))?;

    let params = node
        .children()
        .find(|n| n.kind() == SyntaxKind::ParamList)
        .map(lower_params)
        .unwrap_or_default();

    let ret = find_return_type(node).and_then(map_type);

    let mut locals = Vec::new();
    let mut body = Vec::new();
    if let Some(block) = node.children().find(|n| n.kind() == SyntaxKind::Block) {
        lower_block(block, &mut locals, &mut body);
    }

    Some(FnDef { name, params, locals, body, ret })
}

fn lower_params(node: SyntaxNode) -> Vec<(String, ValType)> {
    let mut out = Vec::new();
    for param in node.children().filter(|n| n.kind() == SyntaxKind::Param) {
        if let (Some(name), Some(ty)) = (
            param.children().find(|n| n.kind() == SyntaxKind::IdentNode).and_then(|n| find_ident_text(n)),
            param.children().find(|n| n.kind() == SyntaxKind::Type).and_then(|n| map_type(n.text().to_string())),
        ) {
            out.push((name, ty));
        }
    }
    out
}

fn lower_block(node: SyntaxNode, locals: &mut Vec<(String, ValType)>, out: &mut Vec<Stmt>) {
    if let Some(stmts) = node.children().find(|n| n.kind() == SyntaxKind::StmtList) {
        for stmt in stmts.children() {
            match stmt.kind() {
                SyntaxKind::LetStmt => {
                    if let (Some(name), Some(expr)) = (
                        stmt.children().find(|n| n.kind() == SyntaxKind::Pattern).and_then(|n| find_ident_text(n)),
                        stmt.children().find(|n| is_expr_kind(n.kind())).and_then(lower_expr),
                    ) {
                        let ty = stmt
                            .children()
                            .find(|n| n.kind() == SyntaxKind::Type)
                            .and_then(|n| map_type(n.text().to_string()))
                            .unwrap_or(ValType::I64);
                        locals.push((name.clone(), ty));
                        out.push(Stmt::Let { name, expr });
                    }
                }
                SyntaxKind::ReturnStmt => {
                    if let Some(expr) = stmt.children().find(|n| is_expr_kind(n.kind())).and_then(lower_expr) {
                        out.push(Stmt::Return(expr));
                    }
                }
                SyntaxKind::IfExpr => {
                    if let Some(stmt_if) = lower_if(stmt) {
                        out.push(stmt_if);
                    }
                }
                SyntaxKind::ExprStmt => {
                    if let Some(expr) = stmt.children().find(|n| is_expr_kind(n.kind())).and_then(lower_expr) {
                        out.push(Stmt::Expr(expr));
                    }
                }
                _ => {}
            }
        }
    }
}

fn lower_if(node: SyntaxNode) -> Option<Stmt> {
    let mut kids = node.children();
    let cond = kids.next().and_then(lower_expr)?;
    let then_block = kids.next()?;
    let else_block = kids.next();
    let mut then_body = Vec::new();
    let mut else_body = Vec::new();
    lower_block(then_block, &mut Vec::new(), &mut then_body);
    if let Some(else_node) = else_block {
        if else_node.kind() == SyntaxKind::IfExpr {
            if let Some(nested) = lower_if(else_node) {
                else_body.push(nested);
            }
        } else {
            lower_block(else_node, &mut Vec::new(), &mut else_body);
        }
    }
    Some(Stmt::If { cond, then_body, else_body })
}

fn lower_expr(node: SyntaxNode) -> Option<Expr> {
    match node.kind() {
        SyntaxKind::LiteralNode => {
            for el in node.children_with_tokens() {
                if let SyntaxElement::Token(t) = el {
                    return match t.kind() {
                        SyntaxKind::Int => t.text().parse::<i64>().ok().map(Expr::Int),
                        SyntaxKind::KwTrue => Some(Expr::Bool(true)),
                        SyntaxKind::KwFalse => Some(Expr::Bool(false)),
                        _ => None,
                    };
                }
            }
            None
        }
        SyntaxKind::IdentNode => find_ident_text(node).map(Expr::Ident),
        SyntaxKind::BinExpr => {
            let mut children = node.children();
            let lhs = children.next().and_then(lower_expr)?;
            let rhs = children.next().and_then(lower_expr)?;
            let op = node.children_with_tokens().find_map(|e| match e {
                SyntaxElement::Token(t) if is_bin_op(t.kind()) => Some(t.kind()),
                _ => None,
            })?;
            Some(Expr::Bin { op, lhs: Box::new(lhs), rhs: Box::new(rhs) })
        }
        SyntaxKind::CallExpr => {
            let mut kids = node.children();
            let name = kids.next().and_then(|n| find_ident_text(n))?;
            let mut args = Vec::new();
            for arg in kids {
                if let Some(expr) = lower_expr(arg) {
                    args.push(expr);
                }
            }
            Some(Expr::Call { name, args })
        }
        SyntaxKind::ParenExpr => node.children().find(|n| is_expr_kind(n.kind())).and_then(lower_expr),
        _ => None,
    }
}

fn signature_from_fn(f: &FnDef, diags: &mut Vec<Diagnostic>) -> (Vec<ValType>, Vec<ValType>) {
    for (_, ty) in &f.params {
        if *ty != ValType::I64 {
            diags.push(Diagnostic { code: "E2002".to_string(), message: "only i64 params supported".to_string() });
        }
    }
    if let Some(ret) = f.ret {
        if ret != ValType::I64 {
            diags.push(Diagnostic { code: "E2003".to_string(), message: "only i64 return supported".to_string() });
        }
    }
    (
        f.params.iter().map(|(_, t)| *t).collect(),
        vec![ValType::I64],
    )
}

fn emit_stmt(body: &mut Function, ctx: &mut EmitCtx, stmt: &Stmt) {
    match stmt {
        Stmt::Let { name, expr } => {
            emit_expr(body, ctx, expr);
            if let Some(idx) = ctx.local_index(name) {
                body.instruction(&Instruction::LocalSet(idx));
            }
        }
        Stmt::Return(expr) => {
            emit_expr(body, ctx, expr);
            body.instruction(&Instruction::Return);
        }
        Stmt::Expr(expr) => {
            emit_expr(body, ctx, expr);
            body.instruction(&Instruction::Drop);
        }
        Stmt::If { cond, then_body, else_body } => {
            emit_expr(body, ctx, cond);
            body.instruction(&Instruction::If(wasm_encoder::BlockType::Empty));
            for stmt in then_body {
                emit_stmt(body, ctx, stmt);
            }
            if !else_body.is_empty() {
                body.instruction(&Instruction::Else);
                for stmt in else_body {
                    emit_stmt(body, ctx, stmt);
                }
            }
            body.instruction(&Instruction::End);
        }
    }
}

fn emit_expr(body: &mut Function, ctx: &mut EmitCtx, expr: &Expr) {
    match expr {
        Expr::Int(v) => {
            body.instruction(&Instruction::I64Const(*v));
        }
        Expr::Bool(v) => {
            body.instruction(&Instruction::I32Const(if *v { 1 } else { 0 }));
        }
        Expr::Ident(name) => {
            if let Some(idx) = ctx.local_index(name) {
                body.instruction(&Instruction::LocalGet(idx));
            } else {
                ctx.diagnostics.push(Diagnostic { code: "E2004".to_string(), message: format!("unknown local {name}") });
                body.instruction(&Instruction::I64Const(0));
            }
        }
        Expr::Bin { op, lhs, rhs } => {
            emit_expr(body, ctx, lhs);
            emit_expr(body, ctx, rhs);
            match op {
                SyntaxKind::Plus => body.instruction(&Instruction::I64Add),
                SyntaxKind::Minus => body.instruction(&Instruction::I64Sub),
                SyntaxKind::Star => body.instruction(&Instruction::I64Mul),
                SyntaxKind::Slash => body.instruction(&Instruction::I64DivS),
                SyntaxKind::EqEq => body.instruction(&Instruction::I64Eq),
                SyntaxKind::Neq => body.instruction(&Instruction::I64Ne),
                SyntaxKind::Lt => body.instruction(&Instruction::I64LtS),
                SyntaxKind::Lte => body.instruction(&Instruction::I64LeS),
                SyntaxKind::Gt => body.instruction(&Instruction::I64GtS),
                SyntaxKind::Gte => body.instruction(&Instruction::I64GeS),
                _ => return,
            };
        }
        Expr::Call { name, args } => {
            for arg in args {
                emit_expr(body, ctx, arg);
            }
            if let Some(idx) = ctx.func_indices.get(name) {
                body.instruction(&Instruction::Call(*idx));
            } else {
                ctx.diagnostics.push(Diagnostic { code: "E2005".to_string(), message: format!("unknown function {name}") });
                body.instruction(&Instruction::I64Const(0));
            }
        }
    }
}

struct EmitCtx<'a> {
    func_indices: &'a std::collections::HashMap<String, u32>,
    locals: &'a [(String, ValType)],
    params: &'a [(String, ValType)],
    diagnostics: &'a mut Vec<Diagnostic>,
}

impl<'a> EmitCtx<'a> {
    fn local_index(&self, name: &str) -> Option<u32> {
        for (i, (n, _)) in self.params.iter().enumerate() {
            if n == name {
                return Some(i as u32);
            }
        }
        let base = self.params.len() as u32;
        for (i, (n, _)) in self.locals.iter().enumerate() {
            let idx = base + i as u32;
            if n == name {
                return Some(idx);
            }
        }
        None
    }
}

fn find_return_type(node: &SyntaxNode) -> Option<String> {
    let mut seen_arrow = false;
    for el in node.children_with_tokens() {
        match el {
            SyntaxElement::Token(t) if t.kind() == SyntaxKind::Arrow => seen_arrow = true,
            SyntaxElement::Node(n) if seen_arrow && n.kind() == SyntaxKind::Type => {
                return Some(n.text().to_string());
            }
            _ => {}
        }
    }
    None
}

fn map_type(text: String) -> Option<ValType> {
    match text.trim() {
        "i64" => Some(ValType::I64),
        "i32" => Some(ValType::I32),
        "bool" => Some(ValType::I32),
        _ => None,
    }
}

fn find_ident_text(node: SyntaxNode) -> Option<String> {
    node.descendants_with_tokens().find_map(|e| match e {
        SyntaxElement::Token(t) if t.kind() == SyntaxKind::Ident => Some(t.text().to_string()),
        _ => None,
    })
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

fn is_bin_op(kind: SyntaxKind) -> bool {
    matches!(
        kind,
        SyntaxKind::Plus
            | SyntaxKind::Minus
            | SyntaxKind::Star
            | SyntaxKind::Slash
            | SyntaxKind::EqEq
            | SyntaxKind::Neq
            | SyntaxKind::Lt
            | SyntaxKind::Lte
            | SyntaxKind::Gt
            | SyntaxKind::Gte
    )
}
