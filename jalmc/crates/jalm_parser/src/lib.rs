use jalm_syntax::parser_events::Event;
use jalm_syntax::{build_green, lex, SyntaxKind, SyntaxNode, Token};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Span {
    pub start: usize,
    pub end: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParseError {
    pub message: String,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct Parse {
    green: rowan::GreenNode,
    pub errors: Vec<ParseError>,
}

impl Parse {
    pub fn syntax(&self) -> SyntaxNode {
        SyntaxNode::new_root(self.green.clone())
    }
}

pub fn parse(source: &str) -> Parse {
    let mut tokens = lex(source);
    let end = source.len();
    tokens.push(Token {
        kind: SyntaxKind::Eof,
        text: String::new(),
        span: end..end,
    });
    let mut p = Parser::new(tokens);
    p.parse_root();
    let green = build_green(p.events);
    Parse { green, errors: p.errors }
}

struct Parser {
    tokens: Vec<Token>,
    pos: usize,
    events: Vec<Event>,
    errors: Vec<ParseError>,
}

#[derive(Debug, Clone, Copy)]
struct Marker {
    pos: usize,
}

#[derive(Debug, Clone, Copy)]
struct CompletedMarker {
    pos: usize,
}

impl Parser {
    fn new(tokens: Vec<Token>) -> Self {
        Self {
            tokens,
            pos: 0,
            events: Vec::new(),
            errors: Vec::new(),
        }
    }

    fn parse_root(&mut self) {
        let m = self.start();
        self.eat_trivia();
        while !self.at(SyntaxKind::Eof) {
            if self.at(SyntaxKind::KwMod) {
                self.parse_module_decl();
            } else if self.at(SyntaxKind::KwUse) {
                self.parse_use_decl();
            } else if self.at(SyntaxKind::KwPub) {
                match self.nth(1) {
                    SyntaxKind::KwFn | SyntaxKind::KwAsync => self.parse_fn_decl(),
                    SyntaxKind::KwStruct | SyntaxKind::KwEnum => self.parse_struct_or_enum(),
                    _ => {
                        self.error_here("expected 'fn', 'struct', or 'enum' after 'pub'");
                        self.bump_any();
                    }
                }
            } else if self.at(SyntaxKind::KwAsync) || self.at(SyntaxKind::KwFn) {
                self.parse_fn_decl();
            } else if self.at(SyntaxKind::KwStruct) || self.at(SyntaxKind::KwEnum) {
                self.parse_struct_or_enum();
            } else {
                let m = self.start();
                self.error_here("expected item");
                if !self.at(SyntaxKind::Eof) {
                    self.bump_any();
                }
                self.complete(m, SyntaxKind::Error);
            }
            self.eat_trivia();
        }
        self.complete(m, SyntaxKind::Root);
    }

    fn parse_module_decl(&mut self) {
        let m = self.start();
        self.expect(SyntaxKind::KwMod);
        self.parse_ident();
        self.expect(SyntaxKind::Semi);
        self.complete(m, SyntaxKind::ModuleDecl);
    }

    fn parse_use_decl(&mut self) {
        let m = self.start();
        self.expect(SyntaxKind::KwUse);
        self.parse_use_path();
        self.eat_trivia();
        if self.at(SyntaxKind::KwAs) {
            self.bump_any();
            self.parse_ident();
        }
        self.expect(SyntaxKind::Semi);
        self.complete(m, SyntaxKind::UseDecl);
    }

    fn parse_use_path(&mut self) {
        self.eat_trivia();
        let m = self.start();
        self.parse_ident();
        while self.at(SyntaxKind::ColonColon) {
            self.bump_any();
            self.parse_ident();
        }
        self.complete(m, SyntaxKind::UsePath);
    }

    fn parse_fn_decl(&mut self) {
        let m = self.start();
        if self.at(SyntaxKind::KwPub) {
            self.bump_any();
        }
        if self.at(SyntaxKind::KwAsync) {
            self.bump_any();
        }
        self.expect(SyntaxKind::KwFn);
        self.parse_ident();
        self.expect(SyntaxKind::LParen);
        let params = self.start();
        self.eat_trivia();
        if !self.at(SyntaxKind::RParen) {
            loop {
                self.parse_param();
                self.eat_trivia();
                if self.at(SyntaxKind::Comma) {
                    self.bump_any();
                    self.eat_trivia();
                    if self.at(SyntaxKind::RParen) {
                        break;
                    }
                } else {
                    break;
                }
            }
        }
        self.expect(SyntaxKind::RParen);
        self.complete(params, SyntaxKind::ParamList);
        self.eat_trivia();
        if self.at(SyntaxKind::Arrow) {
            self.bump_any();
            self.parse_type();
        }
        self.eat_trivia();
        if self.at(SyntaxKind::Bang) {
            self.parse_effect_set();
        }
        self.eat_trivia();
        self.parse_block();
        self.complete(m, SyntaxKind::FnDecl);
    }

    fn parse_param(&mut self) {
        let m = self.start();
        if self.at(SyntaxKind::KwMut) {
            self.bump_any();
        }
        self.parse_ident();
        self.expect(SyntaxKind::Colon);
        self.parse_type();
        self.complete(m, SyntaxKind::Param);
    }

    fn parse_type(&mut self) {
        self.eat_trivia();
        let m = self.start();
        if self.at(SyntaxKind::Ident) {
            self.parse_ident();
            while self.at(SyntaxKind::ColonColon) {
                self.bump_any();
                self.parse_ident();
            }
        } else {
            self.error_here("expected type");
            self.bump_any();
        }
        self.complete(m, SyntaxKind::Type);
    }

    fn parse_effect_set(&mut self) {
        let m = self.start();
        if self.at(SyntaxKind::Bang) {
            self.bump_any();
        } else {
            self.error_here("expected '!'");
        }
        self.expect(SyntaxKind::LBrace);
        self.eat_trivia();
        if !self.at(SyntaxKind::RBrace) {
            loop {
                self.parse_ident();
                self.eat_trivia();
                if self.at(SyntaxKind::Comma) {
                    self.bump_any();
                    self.eat_trivia();
                    if self.at(SyntaxKind::RBrace) {
                        break;
                    }
                } else {
                    break;
                }
            }
        }
        self.expect(SyntaxKind::RBrace);
        self.complete(m, SyntaxKind::EffectSet);
    }

    fn parse_struct_or_enum(&mut self) {
        if self.at(SyntaxKind::KwPub) {
            if self.nth(1) == SyntaxKind::KwStruct {
                self.parse_struct_decl();
            } else if self.nth(1) == SyntaxKind::KwEnum {
                self.parse_enum_decl();
            } else {
                self.error_here("expected 'struct' or 'enum' after 'pub'");
                self.bump_any();
            }
            return;
        }
        if self.at(SyntaxKind::KwStruct) {
            self.parse_struct_decl();
            return;
        }
        if self.at(SyntaxKind::KwEnum) {
            self.parse_enum_decl();
            return;
        }
        self.error_here("expected 'struct' or 'enum'");
        self.bump_any();
    }

    fn parse_struct_decl(&mut self) {
        let m = self.start();
        if self.at(SyntaxKind::KwPub) {
            self.bump_any();
        }
        self.expect(SyntaxKind::KwStruct);
        self.parse_ident();
        self.expect(SyntaxKind::LBrace);
        self.eat_trivia();
        while !self.at(SyntaxKind::RBrace) && !self.at(SyntaxKind::Eof) {
            let f = self.start();
            self.parse_ident();
            self.expect(SyntaxKind::Colon);
            self.parse_type();
            self.expect(SyntaxKind::Semi);
            self.complete(f, SyntaxKind::StructField);
            self.eat_trivia();
        }
        self.expect(SyntaxKind::RBrace);
        self.complete(m, SyntaxKind::StructDecl);
    }

    fn parse_enum_decl(&mut self) {
        let m = self.start();
        if self.at(SyntaxKind::KwPub) {
            self.bump_any();
        }
        self.expect(SyntaxKind::KwEnum);
        self.parse_ident();
        self.expect(SyntaxKind::LBrace);
        self.eat_trivia();
        while !self.at(SyntaxKind::RBrace) && !self.at(SyntaxKind::Eof) {
            let v = self.start();
            self.parse_ident();
            self.eat_trivia();
            if self.at(SyntaxKind::LParen) {
                self.bump_any();
                self.eat_trivia();
                if !self.at(SyntaxKind::RParen) {
                    loop {
                        self.parse_type();
                        self.eat_trivia();
                        if self.at(SyntaxKind::Comma) {
                            self.bump_any();
                            self.eat_trivia();
                            if self.at(SyntaxKind::RParen) {
                                break;
                            }
                        } else {
                            break;
                        }
                    }
                }
                self.expect(SyntaxKind::RParen);
            }
            self.expect(SyntaxKind::Semi);
            self.complete(v, SyntaxKind::EnumVariant);
            self.eat_trivia();
        }
        self.expect(SyntaxKind::RBrace);
        self.complete(m, SyntaxKind::EnumDecl);
    }

    fn parse_block(&mut self) -> CompletedMarker {
        let m = self.start();
        self.expect(SyntaxKind::LBrace);
        let stmts = self.start();
        self.eat_trivia();
        while !self.at(SyntaxKind::RBrace) && !self.at(SyntaxKind::Eof) {
            if self.at(SyntaxKind::KwLet) {
                self.parse_let_stmt();
                self.eat_trivia();
                continue;
            }
            if self.at(SyntaxKind::KwReturn) {
                self.parse_return_stmt();
                self.eat_trivia();
                continue;
            }

            let expr = self.parse_expr_bp(0);
            self.eat_trivia();
            if self.at(SyntaxKind::Semi) {
                let s = expr.precede(self);
                self.bump_any();
                self.complete(s, SyntaxKind::ExprStmt);
                self.eat_trivia();
                continue;
            }

            // tail expression
            break;
        }
        self.complete(stmts, SyntaxKind::StmtList);
        self.expect(SyntaxKind::RBrace);
        self.complete(m, SyntaxKind::Block)
    }

    fn parse_let_stmt(&mut self) {
        let m = self.start();
        self.expect(SyntaxKind::KwLet);
        if self.at(SyntaxKind::KwMut) {
            self.bump_any();
        }
        self.parse_pattern();
        if self.at(SyntaxKind::Colon) {
            self.bump_any();
            self.parse_type();
        }
        self.expect(SyntaxKind::Eq);
        self.parse_expr_bp(0);
        self.expect(SyntaxKind::Semi);
        self.complete(m, SyntaxKind::LetStmt);
    }

    fn parse_return_stmt(&mut self) {
        let m = self.start();
        self.expect(SyntaxKind::KwReturn);
        if !self.at(SyntaxKind::Semi) {
            self.parse_expr_bp(0);
        }
        self.expect(SyntaxKind::Semi);
        self.complete(m, SyntaxKind::ReturnStmt);
    }

    fn parse_pattern(&mut self) {
        self.eat_trivia();
        let m = self.start();
        if self.at(SyntaxKind::Ident) {
            self.parse_ident();
        } else if self.current().is_literal() {
            self.parse_literal();
        } else if self.at(SyntaxKind::Underscore) {
            self.bump_any();
        } else {
            self.error_here("expected pattern");
            self.bump_any();
        }
        self.complete(m, SyntaxKind::Pattern);
    }

    fn parse_expr_bp(&mut self, min_bp: u8) -> CompletedMarker {
        self.eat_trivia();
        let mut lhs = self.parse_postfix();

        loop {
            self.eat_trivia();
            let op = self.current();
            let (l_bp, r_bp) = match infix_binding_power(op) {
                Some((l, r)) => (l, r),
                None => break,
            };
            if l_bp < min_bp {
                break;
            }
            let m = lhs.precede(self);
            self.bump_any();
            self.parse_expr_bp(r_bp);
            lhs = self.complete(m, SyntaxKind::BinExpr);
        }
        lhs
    }

    fn parse_postfix(&mut self) -> CompletedMarker {
        let mut lhs = self.parse_primary();
        loop {
            self.eat_trivia();
            if self.at(SyntaxKind::LParen) {
                let m = lhs.precede(self);
                self.bump_any();
                self.eat_trivia();
                if !self.at(SyntaxKind::RParen) {
                    loop {
                        self.parse_expr_bp(0);
                        self.eat_trivia();
                        if self.at(SyntaxKind::Comma) {
                            self.bump_any();
                            self.eat_trivia();
                            if self.at(SyntaxKind::RParen) {
                                break;
                            }
                        } else {
                            break;
                        }
                    }
                }
                self.expect(SyntaxKind::RParen);
                lhs = self.complete(m, SyntaxKind::CallExpr);
                continue;
            }
            if self.at(SyntaxKind::Dot) {
                let m = lhs.precede(self);
                self.bump_any();
                self.parse_ident();
                lhs = self.complete(m, SyntaxKind::MemberExpr);
                continue;
            }
            break;
        }
        lhs
    }

    fn parse_primary(&mut self) -> CompletedMarker {
        self.eat_trivia();
        if self.at(SyntaxKind::LBrace) {
            return self.parse_block();
        }
        if self.at(SyntaxKind::KwIf) {
            return self.parse_if_expr();
        }
        if self.at(SyntaxKind::KwMatch) {
            return self.parse_match_expr();
        }
        if self.at(SyntaxKind::Ident) {
            return self.parse_ident();
        }
        if self.current().is_literal() {
            return self.parse_literal();
        }
        if self.at(SyntaxKind::LParen) {
            let m = self.start();
            self.bump_any();
            self.parse_expr_bp(0);
            self.expect(SyntaxKind::RParen);
            return self.complete(m, SyntaxKind::ParenExpr);
        }
        let m = self.start();
        self.error_here("expected expression");
        if !self.at(SyntaxKind::Eof) {
            self.bump_any();
        }
        self.complete(m, SyntaxKind::Error)
    }

    fn parse_if_expr(&mut self) -> CompletedMarker {
        let m = self.start();
        self.expect(SyntaxKind::KwIf);
        self.parse_expr_bp(0);
        self.parse_block();
        if self.at(SyntaxKind::KwElse) {
            self.bump_any();
            if self.at(SyntaxKind::KwIf) {
                self.parse_if_expr();
            } else {
                self.parse_block();
            }
        }
        self.complete(m, SyntaxKind::IfExpr)
    }

    fn parse_match_expr(&mut self) -> CompletedMarker {
        let m = self.start();
        self.expect(SyntaxKind::KwMatch);
        self.parse_expr_bp(0);
        self.expect(SyntaxKind::LBrace);
        self.eat_trivia();
        while !self.at(SyntaxKind::RBrace) && !self.at(SyntaxKind::Eof) {
            let arm = self.start();
            self.parse_pattern();
            self.expect(SyntaxKind::FatArrow);
            self.parse_expr_bp(0);
            if self.at(SyntaxKind::Comma) {
                self.bump_any();
            } else {
                self.error_here("expected ',' after match arm");
            }
            self.complete(arm, SyntaxKind::MatchArm);
            self.eat_trivia();
        }
        self.expect(SyntaxKind::RBrace);
        self.complete(m, SyntaxKind::MatchExpr)
    }

    fn parse_ident(&mut self) -> CompletedMarker {
        self.eat_trivia();
        let m = self.start();
        if self.at(SyntaxKind::Ident) {
            self.bump_any();
        } else {
            self.error_here("expected identifier");
            if !self.at(SyntaxKind::Eof) {
                self.bump_any();
            }
        }
        self.complete(m, SyntaxKind::IdentNode)
    }

    fn parse_literal(&mut self) -> CompletedMarker {
        let m = self.start();
        if self.current().is_literal() {
            self.bump_any();
        } else {
            self.error_here("expected literal");
            if !self.at(SyntaxKind::Eof) {
                self.bump_any();
            }
        }
        self.complete(m, SyntaxKind::LiteralNode)
    }

    fn start(&mut self) -> Marker {
        let pos = self.events.len();
        self.events.push(Event::StartNode(SyntaxKind::Tomestone));
        Marker { pos }
    }

    fn complete(&mut self, marker: Marker, kind: SyntaxKind) -> CompletedMarker {
        self.events[marker.pos] = Event::StartNode(kind);
        self.events.push(Event::FinishNode);
        CompletedMarker { pos: marker.pos }
    }

    fn expect(&mut self, kind: SyntaxKind) {
        self.eat_trivia();
        if self.at(kind) {
            self.bump_any();
        } else {
            self.error_here(&format!("expected {:?}", kind));
            let m = self.start();
            if !self.at(SyntaxKind::Eof) {
                self.bump_any();
            }
            self.complete(m, SyntaxKind::Error);
        }
    }

    fn eat_trivia(&mut self) {
        while self.current().is_trivia() {
            self.bump_any();
        }
    }

    fn current(&self) -> SyntaxKind {
        self.tokens.get(self.pos).map(|t| t.kind).unwrap_or(SyntaxKind::Eof)
    }

    fn at(&self, kind: SyntaxKind) -> bool {
        self.current() == kind
    }

    fn nth(&self, n: usize) -> SyntaxKind {
        self.tokens.get(self.pos + n).map(|t| t.kind).unwrap_or(SyntaxKind::Eof)
    }

    fn bump_any(&mut self) {
        let token = self.tokens.get(self.pos).cloned();
        if let Some(token) = token {
            if token.kind != SyntaxKind::Eof {
                self.events.push(Event::Token(token.kind, token.text));
            }
            self.pos += 1;
        }
    }

    fn error_here(&mut self, message: &str) {
        let span = self.tokens.get(self.pos).map(|t| t.span.clone()).unwrap_or(0..0);
        self.errors.push(ParseError {
            message: message.to_string(),
            span: Span { start: span.start, end: span.end },
        });
    }
}

impl CompletedMarker {
    fn precede(self, p: &mut Parser) -> Marker {
        p.events.insert(self.pos, Event::StartNode(SyntaxKind::Tomestone));
        Marker { pos: self.pos }
    }
}

fn infix_binding_power(kind: SyntaxKind) -> Option<(u8, u8)> {
    let (l, r) = match kind {
        SyntaxKind::OrOr => (1, 2),
        SyntaxKind::AndAnd => (3, 4),
        SyntaxKind::EqEq | SyntaxKind::Neq => (5, 6),
        SyntaxKind::Lt | SyntaxKind::Lte | SyntaxKind::Gt | SyntaxKind::Gte => (7, 8),
        SyntaxKind::Plus | SyntaxKind::Minus => (9, 10),
        SyntaxKind::Star | SyntaxKind::Slash | SyntaxKind::Percent => (11, 12),
        _ => return None,
    };
    Some((l, r))
}
