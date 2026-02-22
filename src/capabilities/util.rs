use std::ops::Range;

use auto_lsp::{anyhow, core::ast::AstNode, default::db::tracked::ParsedAst};

// NOTE: It would be great to have these upstream

pub fn get_file_from_db(
    uri: &auto_lsp::lsp_types::Url,
    db: &impl auto_lsp::default::db::BaseDatabase,
) -> Result<auto_lsp::default::db::file::File, anyhow::Error> {
    db.get_file(uri)
        .ok_or_else(|| anyhow::format_err!("File not found in database: {}", uri))
}

// TODO: Check whether this is even useful
pub fn subtree_at(ast: &ParsedAst, line: u32, character: u32) -> Range<usize> {
    let start = ast.nodes.partition_point(|p| {
        let end = p.get_lsp_range().end;
        (end.line, end.character) < (line, character)
    });
    let end = start
        + ast.nodes[start..].partition_point(|p| {
            let start = p.get_lsp_range().start;
            (start.line, start.character) <= (line, character)
        });
    start..end
}

pub fn leaf_at<'a>(
    nodes: &'a [Box<dyn AstNode>],
    line: u32,
    character: u32,
) -> Option<&'a Box<dyn AstNode>> {
    let start = nodes.partition_point(|p| {
        let end = p.get_lsp_range().end;
        (end.line, end.character) < (line, character)
    });

    let end = start
        + nodes[start..].partition_point(|p| {
            let start = p.get_lsp_range().start;
            (start.line, start.character) <= (line, character)
        });

    if start < end {
        Some(&nodes[end - 1])
    } else {
        None
    }
}
