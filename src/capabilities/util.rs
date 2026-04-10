use auto_lsp::{
    anyhow,
    core::ast::AstNode,
    default::db::{BaseDatabase, file::File, tracked::ParsedAst},
    lsp_types::{Position, SemanticTokenType, Url},
};

use crate::capabilities::semantic_tokens::SUPPORTED_TYPES;

// It would be great to have some of these upstream

pub fn get_file_from_db(uri: &Url, db: &impl BaseDatabase) -> Result<File, anyhow::Error> {
    db.get_file(uri)
        .ok_or_else(|| anyhow::format_err!("Unknown file: {}", uri))
}

pub fn get_token_index(type_: SemanticTokenType) -> u32 {
    SUPPORTED_TYPES.iter().position(|x| *x == type_).unwrap() as u32
}

// Note that the given nodes are assumed to be ordered by their starting position

// Finds the rightmost leaf at or before the given position
fn prec_at<'a>(ast: &'a ParsedAst, pos: Position) -> Option<&'a Box<dyn AstNode>> {
    let cutoff = ast.partition_point(|p| {
        let start = p.get_lsp_range().start;
        (start.line, start.character) <= (pos.line, pos.character)
    });

    if cutoff == 0 {
        None
    } else {
        Some(&ast[cutoff - 1])
    }
}

// Finds the leaf at the given position
pub fn leaf_at<'a>(ast: &'a ParsedAst, pos: Position) -> Option<&'a Box<dyn AstNode>> {
    prec_at(ast, pos).and_then(|candidate| {
        let end = candidate.get_lsp_range().end;
        if (end.line, end.character) > (pos.line, pos.character) {
            Some(candidate)
        } else {
            None
        }
    })
}

// Finds the deepest node at the given position
pub fn most_specific_at<'a>(ast: &'a ParsedAst, pos: Position) -> Option<&'a Box<dyn AstNode>> {
    fn walk_up<'a>(
        nodes: &'a [Box<dyn AstNode>],
        node: &'a Box<dyn AstNode>,
        pos: Position,
    ) -> Option<&'a Box<dyn AstNode>> {
        let end = node.get_lsp_range().end;
        if (end.line, end.character) > (pos.line, pos.character) {
            Some(node)
        } else {
            node.get_parent(nodes).and_then(|p| walk_up(nodes, p, pos))
        }
    }

    prec_at(ast, pos).and_then(|candidate| walk_up(ast, candidate, pos))
}

// Finds the deepest node of some type above the given node
pub fn walk_up<'a, T: AstNode>(ast: &'a ParsedAst, node: &'a dyn AstNode) -> Option<&'a T> {
    node.downcast_ref::<T>()
        .or(node.get_parent(ast).and_then(|p| walk_up(ast, p.lower())))
}

// Finds the deepest node of some type at the given position
pub fn capture_at<'a, T: AstNode>(ast: &'a ParsedAst, pos: Position) -> Option<&'a T> {
    most_specific_at(ast, pos).and_then(|n| walk_up::<T>(ast, n.lower()))
}

// TODO: It would be preferable to go through tree-sitter
pub fn is_missing(node: &dyn AstNode) -> bool {
    let span = node.get_span();
    span.start_byte == span.end_byte
}
