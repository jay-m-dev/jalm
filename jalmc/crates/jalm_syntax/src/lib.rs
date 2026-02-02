use logos::Logos;
use rowan::{GreenNode, Language};
use std::ops::Range;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(u16)]
pub enum SyntaxKind {
    Tomestone = 0,
    Eof,
    Whitespace,
    Comment,
    ErrorToken,

    Ident,
    Int,
    Float,
    String,
    Bytes,
    Underscore,

    KwMod,
    KwUse,
    KwFn,
    KwAsync,
    KwStruct,
    KwEnum,
    KwMatch,
    KwIf,
    KwElse,
    KwFor,
    KwIn,
    KwReturn,
    KwLet,
    KwMut,
    KwTrue,
    KwFalse,
    KwScope,
    KwSpawn,
    KwJoin,
    KwAwait,
    KwAs,
    KwPub,

    LParen,
    RParen,
    LBrace,
    RBrace,
    LBracket,
    RBracket,
    Comma,
    Semi,
    Colon,
    Dot,
    ColonColon,
    Arrow,
    FatArrow,

    Question,
    QuestionQuestion,

    Bang,
    Plus,
    Minus,
    Star,
    Slash,
    Percent,

    Eq,
    EqEq,
    Neq,
    Lt,
    Lte,
    Gt,
    Gte,

    AndAnd,
    OrOr,
    Amp,
    Pipe,
    Caret,
    Tilde,

    Shl,
    Shr,
    ShlEq,
    ShrEq,

    PlusEq,
    MinusEq,
    StarEq,
    SlashEq,
    PercentEq,
    AmpEq,
    PipeEq,
    CaretEq,

    Range,
    RangeEq,

    Root,
    ModuleDecl,
    UseDecl,
    UsePath,
    FnDecl,
    ParamList,
    Param,
    Type,
    EffectSet,
    StructDecl,
    StructField,
    EnumDecl,
    EnumVariant,
    Block,
    StmtList,
    LetStmt,
    ReturnStmt,
    ExprStmt,
    IfExpr,
    MatchExpr,
    MatchArm,
    CallExpr,
    MemberExpr,
    BinExpr,
    ParenExpr,
    IdentNode,
    LiteralNode,
    Pattern,
    Error,
}

impl SyntaxKind {
    pub fn is_trivia(self) -> bool {
        matches!(self, SyntaxKind::Whitespace | SyntaxKind::Comment)
    }

    pub fn is_literal(self) -> bool {
        matches!(self, SyntaxKind::Int | SyntaxKind::Float | SyntaxKind::String | SyntaxKind::Bytes | SyntaxKind::KwTrue | SyntaxKind::KwFalse)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct JalmLanguage;

impl Language for JalmLanguage {
    type Kind = SyntaxKind;

    fn kind_from_raw(raw: rowan::SyntaxKind) -> SyntaxKind {
        unsafe { std::mem::transmute(raw.0) }
    }

    fn kind_to_raw(kind: SyntaxKind) -> rowan::SyntaxKind {
        rowan::SyntaxKind(kind as u16)
    }
}

pub type SyntaxNode = rowan::SyntaxNode<JalmLanguage>;
pub type SyntaxToken = rowan::SyntaxToken<JalmLanguage>;
pub type SyntaxElement = rowan::SyntaxElement<JalmLanguage>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Token {
    pub kind: SyntaxKind,
    pub text: String,
    pub span: Range<usize>,
}

#[derive(Logos, Debug, Clone, Copy, PartialEq, Eq)]
enum LexKind {
    #[regex(r"[ \t\r\n]+")]
    Whitespace,

    #[regex(r"//[^\n]*")]
    LineComment,

    #[regex(r"/\*([^*]|\*+[^*/])*\*+/")]
    BlockComment,

    #[token("mod")]
    KwMod,
    #[token("use")]
    KwUse,
    #[token("fn")]
    KwFn,
    #[token("async")]
    KwAsync,
    #[token("struct")]
    KwStruct,
    #[token("enum")]
    KwEnum,
    #[token("match")]
    KwMatch,
    #[token("if")]
    KwIf,
    #[token("else")]
    KwElse,
    #[token("for")]
    KwFor,
    #[token("in")]
    KwIn,
    #[token("return")]
    KwReturn,
    #[token("let")]
    KwLet,
    #[token("mut")]
    KwMut,
    #[token("true")]
    KwTrue,
    #[token("false")]
    KwFalse,
    #[token("scope")]
    KwScope,
    #[token("spawn")]
    KwSpawn,
    #[token("join")]
    KwJoin,
    #[token("await")]
    KwAwait,
    #[token("as")]
    KwAs,
    #[token("pub")]
    KwPub,

    #[token("(")]
    LParen,
    #[token(")")]
    RParen,
    #[token("{")]
    LBrace,
    #[token("}")]
    RBrace,
    #[token("[")]
    LBracket,
    #[token("]")]
    RBracket,
    #[token(",")]
    Comma,
    #[token(";")]
    Semi,
    #[token(":")]
    Colon,
    #[token(".")]
    Dot,
    #[token("::")]
    ColonColon,
    #[token("->")]
    Arrow,
    #[token("=>")]
    FatArrow,

    #[token("??")]
    QuestionQuestion,
    #[token("?")]
    Question,

    #[token("!")]
    Bang,

    #[token("+")]
    Plus,
    #[token("-")]
    Minus,
    #[token("*")]
    Star,
    #[token("/")]
    Slash,
    #[token("%")]
    Percent,

    #[token("=")]
    Eq,
    #[token("==")]
    EqEq,
    #[token("!=")]
    Neq,
    #[token("<=")]
    Lte,
    #[token("<")]
    Lt,
    #[token(">=")]
    Gte,
    #[token(">")]
    Gt,

    #[token("&&")]
    AndAnd,
    #[token("||")]
    OrOr,
    #[token("&")]
    Amp,
    #[token("|")]
    Pipe,
    #[token("^")]
    Caret,
    #[token("~")]
    Tilde,

    #[token("<<=")]
    ShlEq,
    #[token(">>=")]
    ShrEq,
    #[token("<<")]
    Shl,
    #[token(">>")]
    Shr,

    #[token("+=")]
    PlusEq,
    #[token("-=")]
    MinusEq,
    #[token("*=")]
    StarEq,
    #[token("/=")]
    SlashEq,
    #[token("%=")]
    PercentEq,
    #[token("&=")]
    AmpEq,
    #[token("|=")]
    PipeEq,
    #[token("^=")]
    CaretEq,

    #[token("..=")]
    RangeEq,
    #[token("..")]
    Range,

    #[token("_", priority = 3)]
    Underscore,

    #[regex(r"[0-9]([0-9_])*\.[0-9]([0-9_])*")]
    Float,
    #[regex(r"[0-9]([0-9_])*")]
    Int,

    #[regex(r#"b\"([^\"\\]|\\.)*\""#)]
    Bytes,
    #[regex(r#"\"([^\"\\]|\\.)*\""#)]
    String,

    #[regex(r"[A-Za-z_][A-Za-z0-9_]*", priority = 1)]
    Ident,
}

fn lex_kind_to_syntax(kind: LexKind) -> SyntaxKind {
    match kind {
        LexKind::Whitespace => SyntaxKind::Whitespace,
        LexKind::LineComment | LexKind::BlockComment => SyntaxKind::Comment,

        LexKind::KwMod => SyntaxKind::KwMod,
        LexKind::KwUse => SyntaxKind::KwUse,
        LexKind::KwFn => SyntaxKind::KwFn,
        LexKind::KwAsync => SyntaxKind::KwAsync,
        LexKind::KwStruct => SyntaxKind::KwStruct,
        LexKind::KwEnum => SyntaxKind::KwEnum,
        LexKind::KwMatch => SyntaxKind::KwMatch,
        LexKind::KwIf => SyntaxKind::KwIf,
        LexKind::KwElse => SyntaxKind::KwElse,
        LexKind::KwFor => SyntaxKind::KwFor,
        LexKind::KwIn => SyntaxKind::KwIn,
        LexKind::KwReturn => SyntaxKind::KwReturn,
        LexKind::KwLet => SyntaxKind::KwLet,
        LexKind::KwMut => SyntaxKind::KwMut,
        LexKind::KwTrue => SyntaxKind::KwTrue,
        LexKind::KwFalse => SyntaxKind::KwFalse,
        LexKind::KwScope => SyntaxKind::KwScope,
        LexKind::KwSpawn => SyntaxKind::KwSpawn,
        LexKind::KwJoin => SyntaxKind::KwJoin,
        LexKind::KwAwait => SyntaxKind::KwAwait,
        LexKind::KwAs => SyntaxKind::KwAs,
        LexKind::KwPub => SyntaxKind::KwPub,

        LexKind::LParen => SyntaxKind::LParen,
        LexKind::RParen => SyntaxKind::RParen,
        LexKind::LBrace => SyntaxKind::LBrace,
        LexKind::RBrace => SyntaxKind::RBrace,
        LexKind::LBracket => SyntaxKind::LBracket,
        LexKind::RBracket => SyntaxKind::RBracket,
        LexKind::Comma => SyntaxKind::Comma,
        LexKind::Semi => SyntaxKind::Semi,
        LexKind::Colon => SyntaxKind::Colon,
        LexKind::Dot => SyntaxKind::Dot,
        LexKind::ColonColon => SyntaxKind::ColonColon,
        LexKind::Arrow => SyntaxKind::Arrow,
        LexKind::FatArrow => SyntaxKind::FatArrow,

        LexKind::Question => SyntaxKind::Question,
        LexKind::QuestionQuestion => SyntaxKind::QuestionQuestion,
        LexKind::Bang => SyntaxKind::Bang,

        LexKind::Plus => SyntaxKind::Plus,
        LexKind::Minus => SyntaxKind::Minus,
        LexKind::Star => SyntaxKind::Star,
        LexKind::Slash => SyntaxKind::Slash,
        LexKind::Percent => SyntaxKind::Percent,

        LexKind::Eq => SyntaxKind::Eq,
        LexKind::EqEq => SyntaxKind::EqEq,
        LexKind::Neq => SyntaxKind::Neq,
        LexKind::Lt => SyntaxKind::Lt,
        LexKind::Lte => SyntaxKind::Lte,
        LexKind::Gt => SyntaxKind::Gt,
        LexKind::Gte => SyntaxKind::Gte,

        LexKind::AndAnd => SyntaxKind::AndAnd,
        LexKind::OrOr => SyntaxKind::OrOr,
        LexKind::Amp => SyntaxKind::Amp,
        LexKind::Pipe => SyntaxKind::Pipe,
        LexKind::Caret => SyntaxKind::Caret,
        LexKind::Tilde => SyntaxKind::Tilde,

        LexKind::Shl => SyntaxKind::Shl,
        LexKind::Shr => SyntaxKind::Shr,
        LexKind::ShlEq => SyntaxKind::ShlEq,
        LexKind::ShrEq => SyntaxKind::ShrEq,

        LexKind::PlusEq => SyntaxKind::PlusEq,
        LexKind::MinusEq => SyntaxKind::MinusEq,
        LexKind::StarEq => SyntaxKind::StarEq,
        LexKind::SlashEq => SyntaxKind::SlashEq,
        LexKind::PercentEq => SyntaxKind::PercentEq,
        LexKind::AmpEq => SyntaxKind::AmpEq,
        LexKind::PipeEq => SyntaxKind::PipeEq,
        LexKind::CaretEq => SyntaxKind::CaretEq,

        LexKind::Range => SyntaxKind::Range,
        LexKind::RangeEq => SyntaxKind::RangeEq,

        LexKind::Underscore => SyntaxKind::Underscore,

        LexKind::Float => SyntaxKind::Float,
        LexKind::Int => SyntaxKind::Int,
        LexKind::String => SyntaxKind::String,
        LexKind::Bytes => SyntaxKind::Bytes,
        LexKind::Ident => SyntaxKind::Ident,
    }
}

pub fn lex(source: &str) -> Vec<Token> {
    let mut lexer = LexKind::lexer(source);
    let mut tokens = Vec::new();
    while let Some(result) = lexer.next() {
        let span = lexer.span();
        let text = source[span.clone()].to_string();
        let kind = match result {
            Ok(kind) => lex_kind_to_syntax(kind),
            Err(()) => SyntaxKind::ErrorToken,
        };
        tokens.push(Token { kind, text, span });
    }
    tokens
}

pub fn to_string_lossless(node: &SyntaxNode) -> String {
    node.text().to_string()
}

pub fn dump_tree(node: &SyntaxNode) -> String {
    let mut out = String::new();
    dump_tree_impl(node, 0, &mut out);
    out
}

fn dump_tree_impl(node: &SyntaxNode, depth: usize, out: &mut String) {
    let indent = "  ".repeat(depth);
    out.push_str(&format!("{}{:?}\n", indent, node.kind()));
    for child in node.children_with_tokens() {
        match child {
            rowan::NodeOrToken::Node(n) => dump_tree_impl(&n, depth + 1, out),
            rowan::NodeOrToken::Token(t) => {
                let text = t.text().replace('\n', "\\n");
                out.push_str(&format!("{}  {:?} '{}'\n", indent, t.kind(), text));
            }
        }
    }
}

pub fn build_green(events: Vec<crate::parser_events::Event>) -> GreenNode {
    use crate::parser_events::Event;
    let mut builder = rowan::GreenNodeBuilder::new();
    for event in events {
        match event {
            Event::StartNode(kind) => builder.start_node(kind.into()),
            Event::FinishNode => builder.finish_node(),
            Event::Token(kind, text) => builder.token(kind.into(), &text),
        }
    }
    builder.finish()
}

impl From<SyntaxKind> for rowan::SyntaxKind {
    fn from(kind: SyntaxKind) -> Self {
        rowan::SyntaxKind(kind as u16)
    }
}

pub mod parser_events {
    use super::SyntaxKind;

    #[derive(Debug, Clone)]
    pub enum Event {
        StartNode(SyntaxKind),
        FinishNode,
        Token(SyntaxKind, String),
    }
}
