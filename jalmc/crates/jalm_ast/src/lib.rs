use jalm_syntax::{SyntaxKind, SyntaxNode};

pub trait AstNode: Sized {
    fn can_cast(kind: SyntaxKind) -> bool;
    fn cast(node: SyntaxNode) -> Option<Self>;
    fn syntax(&self) -> &SyntaxNode;
}

macro_rules! impl_ast_node {
    ($name:ident, $kind:path) => {
        #[derive(Debug, Clone, PartialEq, Eq)]
        pub struct $name {
            syntax: SyntaxNode,
        }

        impl AstNode for $name {
            fn can_cast(kind: SyntaxKind) -> bool {
                kind == $kind
            }

            fn cast(node: SyntaxNode) -> Option<Self> {
                if Self::can_cast(node.kind()) {
                    Some(Self { syntax: node })
                } else {
                    None
                }
            }

            fn syntax(&self) -> &SyntaxNode {
                &self.syntax
            }
        }

    };
}

impl_ast_node!(Module, SyntaxKind::ModuleDecl);
impl_ast_node!(Import, SyntaxKind::UseDecl);
impl_ast_node!(FnDecl, SyntaxKind::FnDecl);
impl_ast_node!(Param, SyntaxKind::Param);
impl_ast_node!(Block, SyntaxKind::Block);
impl_ast_node!(Let, SyntaxKind::LetStmt);
impl_ast_node!(Struct, SyntaxKind::StructDecl);
impl_ast_node!(Enum, SyntaxKind::EnumDecl);
impl_ast_node!(IfExpr, SyntaxKind::IfExpr);
impl_ast_node!(MatchExpr, SyntaxKind::MatchExpr);
impl_ast_node!(CallExpr, SyntaxKind::CallExpr);
impl_ast_node!(Ident, SyntaxKind::IdentNode);
impl_ast_node!(Literal, SyntaxKind::LiteralNode);

pub fn children<T: AstNode>(node: &SyntaxNode) -> impl Iterator<Item = T> + '_ {
    node.children().filter_map(T::cast)
}
